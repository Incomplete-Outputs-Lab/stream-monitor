use crate::api::twitch_api::TwitchApiClient;
use crate::commands::discovery::DiscoveredStreamInfo;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::database::repositories::game_category_repository::GameCategoryRepository;
use crate::database::repositories::stream_stats_repository::StreamStatsRepository;
use crate::database::repositories::ChannelRepository;
use crate::database::DatabaseManager;
use crate::error::ResultExt;
use crate::DiscoveredStreamsCache;
use chrono::Local;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
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

        // Twitch Client IDの確認（事前チェック）
        if settings.twitch.client_id.is_none() && self.twitch_client.is_none() {
            let error_msg =
                "Twitch Client IDが設定されていません。設定画面でClient IDを設定してください。";
            eprintln!("[AutoDiscovery] {}", error_msg);

            // フロントエンドにエラー通知イベントを発行
            let _ = self.app_handle.emit("auto-discovery-error", error_msg);

            return Err(error_msg.to_string());
        }

        // Twitch クライアントを取得（既存のものか、新規作成）
        let twitch_client = match &self.twitch_client {
            Some(client) => client.clone(),
            None => {
                // twitch_clientがNoneの場合、設定からClient IDを取得して新規作成
                let client_id = settings.twitch.client_id.as_ref().ok_or(
                    "Twitch Client IDが設定されていません。設定画面でClient IDを設定してください。",
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

            eprintln!("[AutoDiscovery] ===== AUTO DISCOVERY STARTED =====");
            eprintln!(
                "[AutoDiscovery] Poll interval: {} seconds",
                poll_interval_secs
            );
            eprintln!(
                "[AutoDiscovery] Max streams: {}",
                auto_discovery_settings.max_streams
            );
            eprintln!(
                "[AutoDiscovery] Game IDs filter: {:?}",
                auto_discovery_settings.filters.game_ids
            );
            eprintln!("[AutoDiscovery] First run: IMMEDIATE");

            // 初回は即座に実行
            let mut is_first_run = true;

            loop {
                if !is_first_run {
                    eprintln!("[AutoDiscovery] Waiting for next poll cycle...");
                    ticker.tick().await;
                    eprintln!("[AutoDiscovery] Starting new poll cycle...");
                } else {
                    eprintln!("[AutoDiscovery] Running FIRST discovery check now...");
                }
                is_first_run = false;

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

                // 初回ポーリング完了マーク（成功・失敗・0件すべての場合で設定）
                {
                    let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> =
                        app_handle.state();
                    if !cache.initialized.load(Ordering::SeqCst) {
                        cache.initialized.store(true, Ordering::SeqCst);
                        eprintln!("[AutoDiscovery] First poll cycle completed, cache initialized");
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
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        eprintln!("[AutoDiscovery] ===== DISCOVER STREAMS CALLED =====");

        // フィルター条件を準備
        let game_ids = if settings.filters.game_ids.is_empty() {
            eprintln!(
                "[AutoDiscovery] No game ID filter - fetching top streams from all categories"
            );
            None
        } else {
            eprintln!(
                "[AutoDiscovery] Game ID filter: {:?}",
                settings.filters.game_ids
            );
            Some(settings.filters.game_ids.clone())
        };

        let languages = if settings.filters.languages.is_empty() {
            eprintln!("[AutoDiscovery] No language filter");
            None
        } else {
            eprintln!(
                "[AutoDiscovery] Language filter: {:?}",
                settings.filters.languages
            );
            Some(settings.filters.languages.clone())
        };

        // 配信を取得
        eprintln!(
            "[AutoDiscovery] Calling Twitch API to get top {} streams...",
            settings.max_streams
        );
        let streams = twitch_client
            .get_top_streams(game_ids, languages, Some(settings.max_streams as usize))
            .await?;

        eprintln!(
            "[AutoDiscovery] Twitch API returned {} streams",
            streams.len()
        );

        // 最小視聴者数フィルターを適用
        let filtered_streams: Vec<_> = streams
            .into_iter()
            .filter(|stream| {
                if settings.filters.min_viewers > 0 {
                    (stream.viewer_count as u32) >= settings.filters.min_viewers
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
        let mut categories_to_upsert: HashMap<String, String> = HashMap::new();
        let mut stats_to_insert: Vec<(String, i32, String, String, String)> = Vec::new();
        let now = Local::now().to_rfc3339();

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
            let follower_count: i32 = 0; // フォロワー数は別APIが必要なため0固定

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

            // stream_statsデータを収集（後でバッチINSERT）
            stats_to_insert.push((
                now.clone(),
                stream.viewer_count as i32,
                user_id.clone(),
                user_login.clone(),
                stream.game_name.to_string(),
            ));

            // カテゴリ情報を収集（ループ後にバッチ処理）
            let game_id_str = stream.game_id.to_string();
            let game_name_str = stream.game_name.to_string();
            if !game_id_str.is_empty() && !game_name_str.is_empty() {
                categories_to_upsert.insert(game_id_str, game_name_str);
            }

            eprintln!(
                "[AutoDiscovery] Discovered stream: {} ({}) - {} viewers, category: {}",
                user_login, user_id, stream.viewer_count, stream.game_name
            );
        }

        let discovered_count = discovered_streams_info.len();

        // 単一の接続とトランザクションで全てのDB操作を実行（デッドロック防止）
        match db_manager.get_connection().await {
            Ok(conn) => {
                // トランザクション開始
                if let Err(e) = conn.execute("BEGIN TRANSACTION", []) {
                    eprintln!("[AutoDiscovery] Failed to begin transaction: {}", e);
                } else {
                    let mut transaction_successful = true;

                    // stream_statsをバッチINSERT（Repositoryメソッド使用）
                    for (collected_at, viewer_count, user_id, user_login, game_name) in
                        &stats_to_insert
                    {
                        if let Err(e) = StreamStatsRepository::insert_auto_discovery_stats(
                            &conn,
                            collected_at,
                            *viewer_count,
                            user_id,
                            user_login,
                            game_name,
                        ) {
                            eprintln!(
                                "[AutoDiscovery] Failed to insert stream stats for {}: {}",
                                user_login, e
                            );
                            transaction_successful = false;
                            break;
                        }
                    }

                    // game_categoriesをバッチUPSERT（Repositoryメソッド使用）
                    if transaction_successful {
                        for (game_id, game_name) in &categories_to_upsert {
                            if let Err(e) = GameCategoryRepository::upsert_category(
                                &conn, game_id, game_name, None,
                            ) {
                                eprintln!(
                                    "[AutoDiscovery] Failed to upsert category {}: {}",
                                    game_id, e
                                );
                                transaction_successful = false;
                                break;
                            }
                        }
                    }

                    // トランザクションをコミットまたはロールバック
                    if transaction_successful {
                        if let Err(e) = conn.execute("COMMIT", []) {
                            eprintln!("[AutoDiscovery] Failed to commit transaction: {}", e);
                        }
                    } else if let Err(e) = conn.execute("ROLLBACK", []) {
                        eprintln!("[AutoDiscovery] Failed to rollback transaction: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[AutoDiscovery] Failed to get connection for database operations: {}",
                    e
                );
            }
        }

        // メモリキャッシュに保存
        let cache: tauri::State<'_, Arc<DiscoveredStreamsCache>> = app_handle.state();
        let mut streams_lock = cache.streams.lock().await;
        *streams_lock = discovered_streams_info;
        drop(streams_lock);

        // フロントエンドにイベントを発行（キャッシュ無効化のトリガー）
        if let Err(e) = app_handle.emit("discovered-streams-updated", ()) {
            eprintln!(
                "[AutoDiscovery] Failed to emit discovered-streams-updated event: {}",
                e
            );
        } else {
            eprintln!("[AutoDiscovery] Event 'discovered-streams-updated' emitted successfully");
        }

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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // トランザクションで処理して競合状態を防ぐ
        let conn = db_manager.get_connection().await?;

        // トランザクション開始
        conn.execute("BEGIN TRANSACTION", [])?;

        let cleanup_result: Result<Vec<(i64, String)>, Box<dyn std::error::Error + Send + Sync>> =
            (|| {
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

                // データベースから削除（ポーリングは自動的に停止される）
                for (channel_id, channel_name) in &channels {
                    // 削除前に再度ライブ状態をチェック（競合回避）
                    let mut check_stmt = conn.prepare(
                    "SELECT EXISTS(SELECT 1 FROM streams WHERE channel_id = ? AND ended_at IS NULL)"
                )?;
                    let is_live: bool = check_stmt.query_row([channel_id], |row| row.get(0))?;
                    drop(check_stmt);

                    if is_live {
                        eprintln!(
                        "[AutoDiscovery] Skip cleanup for {} (id: {}) - channel went live again",
                        channel_name, channel_id
                    );
                        continue;
                    }

                    ChannelRepository::delete(&conn, *channel_id)?;
                    eprintln!(
                        "[AutoDiscovery] Cleaned up offline channel: {} (id: {})",
                        channel_name, channel_id
                    );
                }

                Ok(channels)
            })();

        match cleanup_result {
            Ok(channels) => {
                // コミット
                conn.execute("COMMIT", [])?;
                drop(conn);

                // イベント発行
                for (channel_id, _) in channels {
                    let _ = app_handle.emit("channel-removed", channel_id);
                }

                Ok(())
            }
            Err(e) => {
                // ロールバック
                let _ = conn.execute("ROLLBACK", []);
                drop(conn);
                Err(e)
            }
        }
    }
}
