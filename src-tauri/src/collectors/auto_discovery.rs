use crate::api::twitch_api::TwitchApiClient;
use crate::commands::discovery::DiscoveredStreamInfo;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::database::DatabaseManager;
use crate::error::ResultExt;
use crate::DiscoveredStreamsCache;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
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
            .config_context("load settings")
            .map_err(|e| e.to_string())?;

        let auto_discovery_settings = match &settings.auto_discovery {
            Some(s) if s.enabled => s.clone(),
            _ => {
                eprintln!("[AutoDiscovery] Auto-discovery is disabled");
                return Ok(());
            }
        };

        // Twitch クライアントを取得（既存のものか、新規作成）
        let twitch_client = match &self.twitch_client {
            Some(client) => client.clone(),
            None => {
                // twitch_clientがNoneの場合、設定からClient IDを取得して新規作成
                let client_id = settings.twitch.client_id.as_ref().ok_or(
                    "Twitch Client ID is not configured. Please configure it in Settings.",
                )?;

                eprintln!("[AutoDiscovery] Creating new TwitchApiClient with client_id");
                Arc::new(
                    TwitchApiClient::new(client_id.clone(), None)
                        .with_app_handle(self.app_handle.clone()),
                )
            }
        };

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
                match Self::discover_streams(
                    &twitch_client,
                    current_auto_discovery,
                    &db_manager,
                    &app_handle,
                )
                .await
                {
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

    /// 配信を発見してメモリキャッシュに保存し、統計データをDBに記録
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

        // 最小視聴者数フィルターを適用
        let filtered_streams: Vec<_> = streams
            .into_iter()
            .filter(|stream| {
                if let Some(min_viewers) = settings.filters.min_viewers {
                    (stream.viewer_count as u32) >= min_viewers
                } else {
                    true
                }
            })
            .collect();

        if filtered_streams.is_empty() {
            return Ok(0);
        }

        // User IDを収集
        let user_ids: Vec<String> = filtered_streams
            .iter()
            .map(|s| s.user_id.to_string())
            .collect();

        // ユーザー情報をバッチ取得
        let user_id_refs: Vec<&str> = user_ids.iter().map(|s| s.as_str()).collect();
        let users = twitch_client.get_users_by_ids(&user_id_refs).await?;

        // User情報をHashMapに格納
        let user_map: HashMap<String, _> =
            users.into_iter().map(|u| (u.id.to_string(), u)).collect();

        // メモリキャッシュに保存するための配信情報を構築
        let mut discovered_streams_info = Vec::new();
        let now = Utc::now().to_rfc3339();

        for stream in filtered_streams {
            let user_id = stream.user_id.to_string();
            let user_login = stream.user_login.to_string();

            // User情報を取得
            let user = user_map.get(&user_id);

            // User情報から各フィールドを取得
            let profile_image_url = user
                .and_then(|u| u.profile_image_url.as_deref())
                .map(|s| s.to_string());
            let display_name = Some(stream.user_name.to_string());
            let broadcaster_type =
                user.and_then(|u| u.broadcaster_type.as_ref())
                    .map(|bt| match bt {
                        twitch_api::types::BroadcasterType::Partner => "partner".to_string(),
                        twitch_api::types::BroadcasterType::Affiliate => "affiliate".to_string(),
                        _ => "".to_string(),
                    });
            let follower_count: Option<i32> = None; // view_count is deprecated

            // user_idをi64に変換
            let twitch_user_id: i64 = user_id
                .parse()
                .map_err(|e| format!("Failed to parse Twitch user ID '{}': {}", user_id, e))?;

            // DiscoveredStreamInfoを構築（メモリキャッシュ用）
            let stream_info = DiscoveredStreamInfo {
                id: 0,                          // メモリキャッシュでは使用しない
                twitch_user_id,                 // 不変なuser ID
                channel_id: user_login.clone(), // login（表示用）
                channel_name: user_login.clone(),
                display_name: display_name.clone(),
                profile_image_url: profile_image_url.clone(),
                discovered_at: Some(now.clone()),
                title: Some(stream.title.to_string()),
                category: Some(stream.game_name.to_string()),
                viewer_count: Some(stream.viewer_count as i32),
                follower_count,
                broadcaster_type: broadcaster_type.clone(),
            };
            discovered_streams_info.push(stream_info);

            // stream_statsテーブルに統計データを記録
            let conn = db_manager.get_connection()?;
            conn.execute(
                r#"
                INSERT INTO stream_stats (
                    stream_id, collected_at, viewer_count, chat_rate_1min,
                    twitch_user_id, channel_name, category
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                duckdb::params![
                    None::<i64>, // stream_id = NULL
                    now.as_str(),
                    stream.viewer_count as i32,
                    0, // chat_rate_1min（AutoDiscoveryでは収集しない）
                    user_id.as_str(),
                    user_login.as_str(),
                    stream.game_name.as_str(),
                ],
            )?;
            drop(conn);

            eprintln!(
                "[AutoDiscovery] Discovered stream: {} ({}) - {} viewers, category: {}",
                user_login, user_id, stream.viewer_count, stream.game_name
            );
        }

        let discovered_count = discovered_streams_info.len();

        // メモリキャッシュに保存
        let cache: tauri::State<'_, Arc<DiscoveredStreamsCache>> = app_handle.state();
        let mut streams_lock = cache.streams.lock().await;
        *streams_lock = discovered_streams_info;
        drop(streams_lock);

        // フロントエンドにイベントを発行（キャッシュ無効化のトリガー）
        let _ = app_handle.emit("discovered-streams-updated", ());

        eprintln!(
            "[AutoDiscovery] Discovered {} streams, saved to cache",
            discovered_count
        );

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
