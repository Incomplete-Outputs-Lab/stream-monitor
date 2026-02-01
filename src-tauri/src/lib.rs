// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod collectors;
mod commands;
mod config;
mod database;
mod logger;
mod oauth;
mod websocket;

use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use collectors::{poller::ChannelPoller, twitch::TwitchCollector, youtube::YouTubeCollector};
use commands::{
    analytics::{
        get_broadcaster_analytics, get_channel_daily_stats, get_data_availability,
        get_game_analytics, get_game_daily_stats, list_game_categories,
    },
    channels::{add_channel, list_channels, remove_channel, toggle_channel, update_channel},
    chat::{get_chat_messages, get_chat_rate, get_chat_stats},
    config::{
        delete_oauth_config, delete_token, get_build_info, get_database_init_status,
        get_oauth_config, get_token, has_oauth_config, has_token, initialize_database,
        recreate_database, save_oauth_config, save_token, verify_token,
    },
    database::{create_database_backup, get_database_info},
    export::{export_to_csv, export_to_json},
    logs::get_logs,
    oauth::{poll_twitch_device_token, start_twitch_device_auth},
    sql::{
        delete_sql_template, execute_sql, get_sql_template, list_database_tables,
        list_sql_templates, save_sql_template,
    },
    stats::{get_channel_stats, get_collector_status, get_live_channels, get_stream_stats},
    twitch::{get_twitch_rate_limit_status, validate_twitch_channel},
};
use config::settings::SettingsManager;
use database::DatabaseManager;
use logger::AppLogger;
use std::sync::Arc;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Helper function to start polling for existing enabled channels
fn start_existing_channels_polling(
    db_manager: &tauri::State<'_, DatabaseManager>,
    poller: &mut ChannelPoller,
    app_handle: &tauri::AppHandle,
) -> Result<usize, Box<dyn std::error::Error>> {
    let conn = db_manager.get_connection()?;

    // Get all enabled channels
    let mut stmt = conn.prepare(
        "SELECT id, platform, channel_id, channel_name, display_name, profile_image_url, enabled, poll_interval, follower_count, broadcaster_type, view_count, \
         CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at \
         FROM channels WHERE enabled = true"
    )?;

    let channels: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(database::models::Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: row.get(4)?,
                profile_image_url: row.get(5)?,
                enabled: row.get(6)?,
                poll_interval: row.get(7)?,
                follower_count: row.get(8).ok(),
                broadcaster_type: row.get(9).ok(),
                view_count: row.get(10).ok(),
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })?
        .collect();

    let channels = channels?;
    let count = channels.len();

    // Start polling for each enabled channel
    for channel in channels {
        if let Err(e) = poller.start_polling(channel.clone(), db_manager, app_handle.clone()) {
            eprintln!(
                "Failed to start polling for channel {:?}: {}",
                channel.id, e
            );
            // Note: ここではloggerを渡していないため、eprintln!のままにする
            // start_pollingメソッド内でloggerを使用するように変更する
        }
    }

    Ok(count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize Keyring plugin
            app.handle()
                .plugin(tauri_plugin_keyring::init())
                .expect("failed to initialize keyring plugin");

            // Initialize AppLogger
            let log_path = app_handle
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory")
                .join("logs.txt");
            let logger = AppLogger::new(log_path).expect("Failed to create AppLogger");
            logger.info("Application starting...");
            app.manage(logger.clone());

            // DatabaseManagerを初期化して管理
            let db_manager = DatabaseManager::new(&app_handle)
                .expect("Failed to create DatabaseManager");
            app.manage(db_manager);

            // Initialize ChannelPoller and manage it as app state (before spawning threads)
            let poller = ChannelPoller::new();
            let poller_arc = Arc::new(Mutex::new(poller));
            app.manage(poller_arc.clone());

            // データベース初期化を起動時に実行
            logger.info("Starting database initialization on startup...");
            let poller_for_init = poller_arc.clone();
            let logger_for_init = logger.clone();
            std::thread::Builder::new()
                .stack_size(512 * 1024 * 1024) // 512MB stack for DuckDB initialization
                .spawn(move || {
                    // DatabaseManagerを取得
                    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();

                    // まずデータベース接続を取得
                    let conn = match db_manager.get_connection() {
                        Ok(conn) => conn,
                        Err(e) => {
                            logger_for_init.error(&format!("Database connection failed: {}", e));
                            // フロントエンドにDB初期化失敗を通知
                            let _ = app_handle.emit("database-init-error", e.to_string());
                            return; // DB接続失敗時は以降の処理を中止
                        }
                    };

                    // 次にスキーマを初期化（初回起動時のみ）
                    match crate::database::schema::init_database(&conn) {
                        Ok(_) => {
                            logger_for_init.info("Database schema initialization successful, notifying frontend...");
                            // フロントエンドのイベントリスナーが準備されるまで少し待つ
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            // フロントエンドにDB初期化成功を通知
                            let _ = app_handle.emit("database-init-success", ());
                        }
                        Err(e) => {
                            logger_for_init.error(&format!("Database schema initialization failed: {}", e));
                            // フロントエンドにDB初期化失敗を通知
                            let _ = app_handle.emit("database-init-error", format!("Schema initialization failed: {}", e));
                            return; // DBスキーマ初期化失敗時は以降の処理を中止
                        }
                    }

                    // DB初期化成功時のみ、コレクターとチャンネルポーリングを初期化
                    logger_for_init.info("Initializing application daemons...");

                    // Initialize collectors from settings
                    tauri::async_runtime::block_on(async {
                        // Load settings
                        let settings = match SettingsManager::load_settings(&app_handle) {
                            Ok(settings) => settings,
                            Err(e) => {
                                logger_for_init.error(&format!("Failed to load settings: {}", e));
                                return;
                            }
                        };

                        let mut poller = poller_for_init.lock().await;

                        // Initialize Twitch collector if credentials are available
                        // Device Code Flow uses only client_id (no client_secret required)
                        if let Some(client_id) = &settings.twitch.client_id {
                            let collector = Arc::new(TwitchCollector::new_with_app(client_id.clone(), None, app_handle.clone()));
                            poller.register_twitch_collector(collector);
                            logger_for_init.info("Twitch collector initialized successfully");
                        } else {
                            logger_for_init.info("Twitch credentials not configured, skipping collector initialization");
                        }

                        // Initialize YouTube collector if credentials are available
                        if let (Some(client_id), Some(client_secret)) = (&settings.youtube.client_id, &settings.youtube.client_secret) {
                            // YouTubeCollector用に新しい接続を作成
                            match db_manager.get_connection() {
                                Ok(yt_conn) => {
                                    let db_conn = Arc::new(Mutex::new(yt_conn));
                                    match YouTubeCollector::new(client_id.clone(), client_secret.clone(), "http://localhost:8081/callback".to_string(), Arc::clone(&db_conn)).await {
                                        Ok(collector) => {
                                            poller.register_collector("youtube".to_string(), Arc::new(collector));
                                            logger_for_init.info("YouTube collector initialized successfully");
                                        }
                                        Err(e) => {
                                            logger_for_init.error(&format!("Failed to initialize YouTube collector: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    logger_for_init.error(&format!("Failed to get database connection for YouTube collector: {}", e));
                                }
                            }
                        } else {
                            logger_for_init.info("YouTube credentials not configured, skipping collector initialization");
                        }

                        // Start polling for existing enabled channels
                        logger_for_init.info("Starting polling for existing enabled channels...");
                        match start_existing_channels_polling(&db_manager, &mut poller, &app_handle) {
                            Ok(count) => {
                                logger_for_init.info(&format!("Started polling for {} existing enabled channel(s)", count));
                            }
                            Err(e) => {
                                logger_for_init.error(&format!("Failed to start polling for existing channels: {}", e));
                            }
                        }
                    });

                    logger_for_init.info("Application daemon initialization completed");
                })
                .expect("Failed to spawn thread for collector initialization");

            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::WindowEvent;
            if let WindowEvent::CloseRequested { .. } = event {
                let logger = window.state::<AppLogger>();
                logger.info("Window close requested");
                // Note: DatabaseManager now uses file-based DuckDB with WAL
                // No explicit shutdown needed - DuckDB handles persistence automatically
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // Analytics commands
            get_broadcaster_analytics,
            get_game_analytics,
            list_game_categories,
            get_data_availability,
            get_game_daily_stats,
            get_channel_daily_stats,
            // Channel commands
            add_channel,
            remove_channel,
            update_channel,
            list_channels,
            toggle_channel,
            // Chat commands
            get_chat_messages,
            get_chat_stats,
            get_chat_rate,
            // Config commands
            save_token,
            get_token,
            delete_token,
            has_token,
            verify_token,
            get_build_info,
            get_database_init_status,
            initialize_database,
            recreate_database,
            get_oauth_config,
            save_oauth_config,
            delete_oauth_config,
            has_oauth_config,
            // Database commands
            create_database_backup,
            get_database_info,
            // SQL commands
            execute_sql,
            list_sql_templates,
            get_sql_template,
            save_sql_template,
            delete_sql_template,
            list_database_tables,
            // OAuth commands
            start_twitch_device_auth,
            poll_twitch_device_token,
            // Stats commands
            get_stream_stats,
            get_live_channels,
            get_channel_stats,
            get_collector_status,
            // Export commands
            export_to_csv,
            export_to_json,
            // Logs commands
            get_logs,
            // Twitch commands
            validate_twitch_channel,
            get_twitch_rate_limit_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
