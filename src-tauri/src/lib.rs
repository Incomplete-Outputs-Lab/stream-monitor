// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod collectors;
mod commands;
mod config;
mod database;
mod oauth;
mod websocket;

use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use collectors::{poller::ChannelPoller, twitch::TwitchCollector, youtube::YouTubeCollector};
use commands::{
    channels::{add_channel, list_channels, remove_channel, toggle_channel, update_channel},
    chat::{get_chat_messages, get_chat_rate, get_chat_stats},
    config::{
        delete_oauth_config, delete_token, get_build_info, get_database_init_status,
        get_oauth_config, get_token, has_oauth_config, has_token, initialize_database,
        recreate_database, save_oauth_config, save_token, verify_token,
    },
    export::{export_to_csv, export_to_json},
    logs::get_logs,
    oauth::{start_twitch_device_auth, poll_twitch_device_token},
    stats::{get_channel_stats, get_live_channels, get_stream_stats},
    stronghold::{check_vault_initialized, initialize_vault},
};
use config::settings::SettingsManager;
use database::DatabaseManager;
use std::sync::Arc;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize Stronghold plugin with argon2 KDF
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");
            app.handle()
                .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())
                .expect("failed to initialize stronghold plugin");

            // DatabaseManagerを初期化して管理
            let db_manager = DatabaseManager::new(&app_handle)
                .expect("Failed to create DatabaseManager");
            app.manage(db_manager);

            // データベース初期化を起動時に実行
            eprintln!("Starting database initialization on startup...");
            std::thread::Builder::new()
                .stack_size(512 * 1024 * 1024) // 512MB stack for DuckDB initialization
                .spawn(move || {
                    // DatabaseManagerを取得
                    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();

                    // まずデータベース接続を取得
                    let conn = match db_manager.get_connection() {
                        Ok(conn) => conn,
                        Err(e) => {
                            eprintln!("Database connection failed: {}", e);
                            // フロントエンドにDB初期化失敗を通知
                            let _ = app_handle.emit("database-init-error", e.to_string());
                            return; // DB接続失敗時は以降の処理を中止
                        }
                    };

                    // 次にスキーマを初期化（初回起動時のみ）
                    match crate::database::schema::init_database(&conn) {
                        Ok(_) => {
                            eprintln!("Database schema initialization successful, notifying frontend...");
                            // フロントエンドのイベントリスナーが準備されるまで少し待つ
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            // フロントエンドにDB初期化成功を通知
                            let _ = app_handle.emit("database-init-success", ());
                        }
                        Err(e) => {
                            eprintln!("Database schema initialization failed: {}", e);
                            // フロントエンドにDB初期化失敗を通知
                            let _ = app_handle.emit("database-init-error", format!("Schema initialization failed: {}", e));
                            return; // DBスキーマ初期化失敗時は以降の処理を中止
                        }
                    }

                    // DB初期化成功時のみ、ChannelPoller等のデーモンを初期化
                    eprintln!("Initializing application daemons...");

                    // Initialize ChannelPoller and manage it as app state
                    let poller = ChannelPoller::new();
                    let poller_arc = Arc::new(Mutex::new(poller));
                    // Note: app.manage() cannot be called from a spawned thread, so we handle this differently
                    // The poller will be managed per command invocation instead

                    // Initialize collectors from settings
                    let poller_for_collectors = poller_arc.clone();
                    tauri::async_runtime::block_on(async {
                        // Load settings
                        let settings = match SettingsManager::load_settings(&app_handle) {
                            Ok(settings) => settings,
                            Err(e) => {
                                eprintln!("Failed to load settings: {}", e);
                                return;
                            }
                        };

                        let mut poller = poller_for_collectors.lock().await;

                        // Initialize Twitch collector if credentials are available
                        // Device Code Flow uses only client_id (no client_secret required)
                        if let Some(client_id) = &settings.twitch.client_id {
                            let collector = TwitchCollector::new_with_app(client_id.clone(), None, app_handle.clone());
                            poller.register_collector("twitch".to_string(), Arc::new(collector));
                            println!("Twitch collector initialized successfully");
                        } else {
                            println!("Twitch credentials not configured, skipping collector initialization");
                        }

                        // Initialize YouTube collector if credentials are available
                        if let (Some(client_id), Some(client_secret)) = (&settings.youtube.client_id, &settings.youtube.client_secret) {
                            let db_conn = Arc::new(Mutex::new(conn));
                            match YouTubeCollector::new(client_id.clone(), client_secret.clone(), "http://localhost:8081/callback".to_string(), Arc::clone(&db_conn)).await {
                                Ok(collector) => {
                                    poller.register_collector("youtube".to_string(), Arc::new(collector));
                                    println!("YouTube collector initialized successfully");
                                }
                                Err(e) => {
                                    eprintln!("Failed to initialize YouTube collector: {}", e);
                                }
                            }
                        } else {
                            println!("YouTube credentials not configured, skipping collector initialization");
                        }
                    });
                })
                .expect("Failed to spawn thread for collector initialization");

            // DB初期化が成功した場合のみ、既存チャンネルのポーリングを開始
            eprintln!("Application daemon initialization completed");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
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
            // OAuth commands
            start_twitch_device_auth,
            poll_twitch_device_token,
            // Stronghold commands
            check_vault_initialized,
            initialize_vault,
            // Stats commands
            get_stream_stats,
            get_live_channels,
            get_channel_stats,
            // Export commands
            export_to_csv,
            export_to_json,
            // Logs commands
            get_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
