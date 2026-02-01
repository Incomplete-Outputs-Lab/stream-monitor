use crate::api::twitch_api::TwitchApiClient;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::database::DatabaseManager;
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

/// 自動発見ポーラー
///
/// 設定に基づいてTwitchの上位配信を定期的に取得し、
/// 新しく発見した配信を自動的に監視対象に追加する
pub struct AutoDiscoveryPoller {
    twitch_client: Option<Arc<TwitchApiClient>>,
    db_manager: Arc<DatabaseManager>,
    app_handle: AppHandle,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl AutoDiscoveryPoller {
    pub fn new(
        twitch_client: Option<Arc<TwitchApiClient>>,
        db_manager: Arc<DatabaseManager>,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            twitch_client,
            db_manager,
            app_handle,
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 自動発見を開始
    pub async fn start(&self) -> Result<(), String> {
        // 設定をロード
        let settings = SettingsManager::load_settings(&self.app_handle)
            .map_err(|e| format!("Failed to load settings: {}", e))?;

        let auto_discovery_settings = match &settings.auto_discovery {
            Some(s) if s.enabled => s.clone(),
            _ => {
                eprintln!("[AutoDiscovery] Auto-discovery is disabled");
                return Ok(());
            }
        };

        // Twitch クライアントがない場合はエラー
        let twitch_client = self
            .twitch_client
            .as_ref()
            .ok_or("Twitch client not available")?
            .clone();

        let db_manager = Arc::clone(&self.db_manager);
        let app_handle = self.app_handle.clone();

        // 既存のタスクを停止
        self.stop().await;

        // 新しいタスクを開始
        let task = tokio::spawn(async move {
            let poll_interval_secs = auto_discovery_settings.poll_interval as u64;
            let mut ticker = interval(Duration::from_secs(poll_interval_secs));

            eprintln!(
                "[AutoDiscovery] Started polling every {} seconds",
                poll_interval_secs
            );

            loop {
                ticker.tick().await;

                // 最新の設定を再読み込み
                let current_settings = match SettingsManager::load_settings(&app_handle) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[AutoDiscovery] Failed to reload settings: {}", e);
                        continue;
                    }
                };

                let current_auto_discovery = match &current_settings.auto_discovery {
                    Some(s) if s.enabled => s,
                    _ => {
                        eprintln!("[AutoDiscovery] Auto-discovery disabled, stopping...");
                        break;
                    }
                };

                // 配信を取得
                match Self::discover_streams(&twitch_client, current_auto_discovery, &db_manager, &app_handle).await {
                    Ok(count) => {
                        eprintln!("[AutoDiscovery] Discovered {} streams", count);
                        if count > 0 {
                            // 新しいチャンネルが追加されたことをフロントエンドに通知
                            let _ = app_handle.emit("channels-updated", ());
                        }
                    }
                    Err(e) => {
                        eprintln!("[AutoDiscovery] Error discovering streams: {}", e);
                    }
                }

                // 配信終了したチャンネルをクリーンアップ
                if let Err(e) = Self::cleanup_offline_channels(&db_manager, &app_handle).await {
                    eprintln!("[AutoDiscovery] Error cleaning up offline channels: {}", e);
                }
            }

            eprintln!("[AutoDiscovery] Polling stopped");
        });

        // タスクハンドルを保存
        let mut handle = self.task_handle.lock().await;
        *handle = Some(task);

        Ok(())
    }

    /// 自動発見を停止
    pub async fn stop(&self) {
        let mut handle = self.task_handle.lock().await;
        if let Some(task) = handle.take() {
            task.abort();
            eprintln!("[AutoDiscovery] Stopped");
        }
    }

    /// 配信を発見してデータベースに追加
    async fn discover_streams(
        twitch_client: &TwitchApiClient,
        settings: &AutoDiscoverySettings,
        db_manager: &Arc<DatabaseManager>,
        app_handle: &AppHandle,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // フィルター条件を準備
        let game_ids = if settings.filters.game_ids.is_empty() {
            None
        } else {
            Some(settings.filters.game_ids.clone())
        };

        let languages = if settings.filters.languages.is_empty() {
            None
        } else {
            Some(settings.filters.languages.clone())
        };

        // 配信を取得
        let streams = twitch_client
            .get_top_streams(game_ids, languages, Some(settings.max_streams as usize))
            .await?;

        let mut discovered_count = 0;

        for stream in streams {
            // 最小視聴者数フィルター
            if let Some(min_viewers) = settings.filters.min_viewers {
                if (stream.viewer_count as u32) < min_viewers {
                    continue;
                }
            }

            // user_idとuser_loginを取得
            let user_id = stream.user_id.to_string();
            let user_login = stream.user_login.to_string();

            // すでに登録されているかチェック
            let conn = db_manager.get_connection()?;
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM channels WHERE platform = 'twitch' AND channel_id = ?",
            )?;
            let count: i64 = stmt.query_row([&user_id], |row| row.get(0))?;
            drop(stmt);
            drop(conn);

            if count > 0 {
                // 既に登録済み
                continue;
            }

            // 新しいチャンネルとして登録
            let now = Utc::now().to_rfc3339();

            // データベースに挿入
            let conn = db_manager.get_connection()?;
            conn.execute(
                r#"
                INSERT INTO channels (
                    platform, channel_id, channel_name, display_name,
                    enabled, poll_interval, is_auto_discovered, discovered_at,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                [
                    &"twitch",
                    &user_id.as_str(),
                    &user_login.as_str(),
                    &stream.user_name.as_str(),
                    &"true",
                    &"60",
                    &"true",
                    &now.as_str(),
                    &now.as_str(),
                    &now.as_str(),
                ],
            )?;

            drop(conn);

            eprintln!(
                "[AutoDiscovery] Added channel: {} ({})",
                user_login, user_id
            );
            
            // Emit event to notify that a new channel was added
            let _ = app_handle.emit("channel-added", user_id.clone());
            
            discovered_count += 1;
        }

        Ok(discovered_count)
    }

    /// オフラインになった自動発見チャンネルをクリーンアップ
    async fn cleanup_offline_channels(
        db_manager: &Arc<DatabaseManager>,
        app_handle: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = db_manager.get_connection()?;

        // 自動発見されたチャンネルで、最新の配信が終了しているものを取得
        let mut stmt = conn.prepare(
            r#"
            SELECT c.id, c.channel_name
            FROM channels c
            WHERE c.is_auto_discovered = true
            AND NOT EXISTS (
                SELECT 1 FROM streams s
                WHERE s.channel_id = c.id
                AND s.ended_at IS NULL
            )
            AND EXISTS (
                SELECT 1 FROM streams s
                WHERE s.channel_id = c.id
            )
            "#,
        )?;

        let channels: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        drop(stmt);
        drop(conn);

        // クリーンアップ処理
        for (channel_id, channel_name) in channels {
            // データベースから削除（ポーリングは自動的に停止される）
            let conn = db_manager.get_connection()?;
            conn.execute("DELETE FROM channels WHERE id = ?", [channel_id])?;
            drop(conn);

            // Emit event to notify that a channel was removed
            let _ = app_handle.emit("channel-removed", channel_id);

            eprintln!(
                "[AutoDiscovery] Cleaned up offline channel: {} (id: {})",
                channel_name, channel_id
            );
        }

        Ok(())
    }
}
