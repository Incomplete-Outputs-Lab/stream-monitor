// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod collectors;
mod commands;
mod config;
mod database;
mod oauth;
mod websocket;

use tauri::Manager;
use tokio::sync::Mutex;

use collectors::{poller::ChannelPoller, twitch::TwitchCollector, youtube::YouTubeCollector};
use commands::{
    channels::{add_channel, list_channels, remove_channel, toggle_channel, update_channel},
    chat::{get_chat_messages, get_chat_rate, get_chat_stats},
    config::{delete_token, get_token, has_token, save_token, verify_token},
    export::{export_to_csv, export_to_json},
    oauth::{login_with_twitch, login_with_youtube},
    stats::{get_channel_stats, get_live_channels, get_stream_stats},
};
use config::settings::SettingsManager;
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
            // Initialize ChannelPoller and manage it as app state
            let poller = ChannelPoller::new(app.handle().clone());
            let poller_arc = Arc::new(Mutex::new(poller));
            app.manage(poller_arc.clone());

            // Initialize collectors from settings
            let app_handle = app.handle().clone();
            let poller_for_collectors = poller_arc.clone();
            std::thread::Builder::new()
                .stack_size(512 * 1024 * 1024) // 512MB stack for DuckDB initialization
                .spawn(move || {
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
                        if let (Some(client_id), Some(client_secret)) = (&settings.twitch.client_id, &settings.twitch.client_secret) {
                            let collector = TwitchCollector::new(client_id.clone(), client_secret.clone());
                            poller.register_collector("twitch".to_string(), Arc::new(collector));
                            println!("Twitch collector initialized successfully");
                        } else {
                            println!("Twitch credentials not configured, skipping collector initialization");
                        }

                        // Initialize YouTube collector if credentials are available
                        if let (Some(client_id), Some(client_secret)) = (&settings.youtube.client_id, &settings.youtube.client_secret) {
                            match YouTubeCollector::new(client_id.clone(), client_secret.clone(), "http://localhost:8081/callback".to_string()).await {
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

            // Start polling for existing enabled channels in a separate task
            // DISABLED: Database initialization moved to lazy initialization
            // Channels will be loaded when first accessed via Tauri commands
            /*
            let app_handle = app.handle().clone();
            let poller_clone = poller_arc.clone();
            std::thread::Builder::new()
                .stack_size(512 * 1024 * 1024) // 512MB stack for DuckDB operations
                .spawn(move || {
                    // Delay to let app fully initialize
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    tauri::async_runtime::block_on(async {
                        eprintln!("Starting database connection...");
                        let conn = match get_connection(&app_handle) {
                            Ok(conn) => {
                                eprintln!("Database connection successful");
                                conn
                            }
                            Err(e) => {
                                eprintln!("Failed to get database connection: {}", e);
                                return;
                            }
                        };
                        
                        // Get all enabled channels
                        let mut stmt = match conn.prepare(
                            "SELECT id, platform, channel_id, channel_name, enabled, poll_interval, created_at, updated_at
                             FROM channels WHERE enabled = 1"
                        ) {
                            Ok(stmt) => stmt,
                            Err(e) => {
                                eprintln!("Failed to prepare channels query: {}", e);
                                return;
                            }
                        };

                        let channels: Vec<Channel> = match stmt.query_map([], |row| {
                            Ok(Channel {
                                id: Some(row.get(0)?),
                                platform: row.get(1)?,
                                channel_id: row.get(2)?,
                                channel_name: row.get(3)?,
                                display_name: None,
                                enabled: row.get(4)?,
                                poll_interval: row.get(5)?,
                                created_at: Some(row.get(6)?),
                                updated_at: Some(row.get(7)?),
                            })
                        }) {
                            Ok(channels_iter) => match channels_iter.collect::<Result<Vec<_>, _>>() {
                                Ok(channels) => channels,
                                Err(e) => {
                                    eprintln!("Failed to collect channels: {}", e);
                                    return;
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to query channels: {}", e);
                                return;
                            }
                        };

                        // Start polling for each enabled channel
                        let mut poller = poller_clone.lock().await;
                        for channel in channels {
                            if let Err(e) = poller.start_polling(channel) {
                                eprintln!("Failed to start polling for existing channel: {}", e);
                                // Continue with other channels even if one fails
                            }
                        }
                    });
                })
                .expect("Failed to spawn thread for channel polling");
            */
            eprintln!("Database initialization skipped during startup - will be initialized on first access");

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
            // OAuth commands
            login_with_twitch,
            login_with_youtube,
            // Stats commands
            get_stream_stats,
            get_live_channels,
            get_channel_stats,
            // Export commands
            export_to_csv,
            export_to_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
