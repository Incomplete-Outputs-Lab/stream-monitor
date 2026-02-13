// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod api;
mod collectors;
mod commands;
mod config;
mod constants;
pub mod database;
mod error;
mod logger;
mod oauth;
mod websocket;

use std::sync::atomic::AtomicBool;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tokio::sync::Mutex;

use collectors::{
    auto_discovery::AutoDiscoveryPoller, poller::ChannelPoller, twitch::TwitchCollector,
    youtube::YouTubeCollector,
};
use commands::{
    analytics::{
        detect_chat_spikes, get_broadcaster_analytics, get_channel_daily_stats,
        get_chat_engagement_timeline, get_chatter_behavior_stats, get_data_availability,
        get_game_analytics, get_game_daily_stats, get_time_pattern_stats, get_top_chatters,
        get_user_segment_stats, list_game_categories,
    },
    channels::{
        add_channel, list_channels, list_channels_basic, remove_channel, toggle_channel,
        update_channel,
    },
    chat::{get_chat_messages, get_chat_messages_around_timestamp},
    config::{
        delete_oauth_config, delete_token, get_build_info, get_database_init_status,
        get_oauth_config, has_oauth_config, recreate_database, save_oauth_config, save_token,
        verify_token,
    },
    data_science::{
        detect_anomalies, get_category_change_impact, get_chatter_activity_scores,
        get_emote_analysis, get_message_length_stats, get_viewer_chat_correlation,
        get_word_frequency_analysis,
    },
    database::get_database_info,
    discovery::{
        get_auto_discovery_settings, get_discovered_streams, get_games_by_ids,
        promote_discovered_channel, promote_discovered_channels, save_auto_discovery_settings,
        search_twitch_games, toggle_auto_discovery, DiscoveredStreamInfo,
    },
    export::{export_to_delimited, preview_export_data},
    game_categories::{
        delete_game_category, get_game_categories, get_game_category, search_game_categories,
        upsert_game_category,
    },
    logs::get_logs,
    multiview::get_multiview_realtime_stats,
    oauth::{poll_twitch_device_token, reinitialize_twitch_collector, start_twitch_device_auth},
    sql::{
        delete_sql_template, execute_sql, list_database_tables, list_sql_templates,
        save_sql_template,
    },
    stats::{get_realtime_chat_rate, get_stream_stats},
    system::is_backend_ready,
    timeline::{
        get_channel_streams, get_stream_timeline, get_streams_by_date_range,
        get_suggested_streams_for_comparison,
    },
    twitch::{get_twitch_rate_limit_status, validate_twitch_channel},
    window::show_main_window,
};
use config::settings::SettingsManager;
use database::DatabaseManager;
use logger::AppLogger;
use std::sync::Arc;

/// メモリキャッシュ: 自動発見された配信の最新結果
pub struct DiscoveredStreamsCache {
    pub streams: Mutex<Vec<DiscoveredStreamInfo>>,
    pub initialized: AtomicBool,
}

/// アップデート確認関数
async fn check_for_updates(app: tauri::AppHandle) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    use tauri_plugin_updater::UpdaterExt;

    // updaterが設定されているか確認
    let updater = match app.updater_builder().build() {
        Ok(u) => u,
        Err(_) => {
            // 署名鍵が未設定の場合
            app.dialog()
                .message(
                    "アップデート機能は未設定です。\n署名鍵とエンドポイントを設定してください。",
                )
                .kind(MessageDialogKind::Warning)
                .blocking_show();
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            // アップデートダイアログを表示
            let confirmed = app
                .dialog()
                .message(format!(
                    "新しいバージョン {} が利用可能です。アップデートしますか?",
                    update.version
                ))
                .kind(MessageDialogKind::Info)
                .title("アップデート利用可能")
                .blocking_show();

            if confirmed {
                let _ = update.download_and_install(|_, _| {}, || {}).await;
            }
        }
        Ok(None) => {
            app.dialog()
                .message("最新バージョンを使用中です。")
                .kind(MessageDialogKind::Info)
                .title("アップデート")
                .blocking_show();
        }
        Err(e) => {
            app.dialog()
                .message(format!("アップデート確認に失敗しました: {}", e))
                .kind(MessageDialogKind::Error)
                .title("エラー")
                .blocking_show();
        }
    }
}

/// Helper function to start polling for existing enabled channels
async fn start_existing_channels_polling(
    db_manager: &tauri::State<'_, DatabaseManager>,
    poller: &mut ChannelPoller,
    app_handle: &tauri::AppHandle,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    use crate::database::repositories::ChannelRepository;

    let channels = db_manager
        .with_connection(ChannelRepository::list_enabled)
        .await?;
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
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 2回目以降の起動時に既存のウィンドウを表示してフォーカス
            let logger = app.state::<AppLogger>();
            logger.info("Second instance detected - showing existing window");

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
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

            // DatabaseManagerを初期化して管理（失敗時はパニックせずログして終了）
            let db_manager = match DatabaseManager::new(&app_handle) {
                Ok(m) => m,
                Err(e) => {
                    let msg = format!("Failed to create DatabaseManager: {}", e);
                    logger.error(&msg);
                    eprintln!("{}", msg);
                    std::process::exit(1);
                }
            };
            app.manage(db_manager.clone());

            // Ctrl+C / SIGTERMシグナルハンドラを設定（ホットリロード対策）
            let db_manager_for_signal = db_manager.clone();
            let logger_for_signal = logger.clone();
            std::thread::spawn(move || {
                if let Err(e) = ctrlc::set_handler(move || {
                    eprintln!("[Signal] Received termination signal, performing cleanup...");
                    logger_for_signal.info("Received termination signal, performing cleanup...");
                    if let Err(e) = tauri::async_runtime::block_on(async {
                        db_manager_for_signal.shutdown().await
                    }) {
                        eprintln!("[Signal] Cleanup failed: {}", e);
                        logger_for_signal.error(&format!("Cleanup failed: {}", e));
                    }
                    std::process::exit(0);
                }) {
                    eprintln!("[Signal] Failed to set signal handler: {}", e);
                }
            });

            // Initialize ChannelPoller and manage it as app state (before spawning threads)
            let poller = ChannelPoller::new();
            let poller_arc = Arc::new(Mutex::new(poller));
            app.manage(poller_arc.clone());

            // Initialize AutoDiscoveryPoller state (will be populated later)
            let auto_discovery_poller: Arc<tokio::sync::Mutex<Option<AutoDiscoveryPoller>>> =
                Arc::new(tokio::sync::Mutex::new(None));
            app.manage(auto_discovery_poller.clone());

            // Initialize DiscoveredStreamsCache
            let discovered_streams_cache = Arc::new(DiscoveredStreamsCache {
                streams: Mutex::new(Vec::new()),
                initialized: AtomicBool::new(false),
            });
            app.manage(discovered_streams_cache);

            // データベース初期化を起動時に実行
            logger.info("Starting database initialization on startup...");
            let poller_for_init = poller_arc.clone();
            let logger_for_init = logger.clone();
            let app_handle_for_init = app_handle.clone();
            std::thread::Builder::new()
                .stack_size(2 * 1024 * 1024 * 1024) // 2GB stack for DuckDB initialization
                .spawn(move || {
                    // DatabaseManagerを取得
                    let db_manager: tauri::State<'_, DatabaseManager> = app_handle_for_init.state();

                    // データベース接続を取得してスキーマを初期化
                    let schema_result = tauri::async_runtime::block_on(async {
                        db_manager.with_connection(|conn| {
                            crate::database::schema::init_database(conn)
                                .map_err(|e| e.to_string())
                        }).await
                    });

                    match schema_result {
                        Ok(()) => {
                            logger_for_init.info("Database schema initialization successful, notifying frontend...");
                            // フロントエンドにDB初期化成功を通知
                            let _ = app_handle_for_init.emit("database-init-success", ());
                        }
                        Err(e) => {
                            logger_for_init.error(&format!("Database schema initialization failed: {}", e));
                            // フロントエンドにDB初期化失敗を通知
                            let _ = app_handle_for_init.emit("database-init-error", format!("Schema initialization failed: {}", e));
                            return; // DBスキーマ初期化失敗時は以降の処理を中止
                        }
                    }

                    // DB初期化成功時のみ、コレクターとチャンネルポーリングを初期化
                    logger_for_init.info("Initializing application daemons...");

                    // Initialize collectors from settings
                    tauri::async_runtime::block_on(async {
                        // Load settings
                        let settings = match SettingsManager::load_settings(&app_handle_for_init) {
                            Ok(settings) => settings,
                            Err(e) => {
                                logger_for_init.error(&format!("Failed to load settings: {}", e));
                                return;
                            }
                        };

                        // Initialize Twitch collector if credentials are available
                        // Device Code Flow uses only client_id (no client_secret required)
                        if let Some(client_id) = &settings.twitch.client_id {
                            let collector = Arc::new(TwitchCollector::new_with_app(
                                client_id.clone(),
                                None,
                                app_handle_for_init.clone(),
                                Arc::new(db_manager.inner().clone()),
                                Arc::new(logger_for_init.clone()),
                            ));
                            // IRC DB ハンドラーを初期化
                            collector.initialize_irc().await;

                            // Register collector - lock only for registration
                            {
                                let mut poller = poller_for_init.lock().await;
                                poller.register_twitch_collector(collector);
                            }
                            logger_for_init.info("Twitch collector initialized successfully with IRC support");
                        } else {
                            logger_for_init.info("Twitch credentials not configured, skipping collector initialization");
                        }

                        // Initialize YouTube collector if credentials are available
                        if let (Some(client_id), Some(client_secret)) = (&settings.youtube.client_id, &settings.youtube.client_secret) {
                            match YouTubeCollector::new(
                                client_id.clone(),
                                client_secret.clone(),
                                "http://localhost:8081/callback".to_string(),
                                Arc::new(db_manager.inner().clone()),
                            )
                            .await
                            {
                                Ok(collector) => {
                                    // Register collector - lock only for registration
                                    {
                                        let mut poller = poller_for_init.lock().await;
                                        poller.register_collector(
                                            crate::constants::database::PLATFORM_YOUTUBE.to_string(),
                                            Arc::new(collector),
                                        );
                                    }
                                    logger_for_init
                                        .info("YouTube collector initialized successfully");
                                }
                                Err(e) => {
                                    logger_for_init
                                        .error(&format!("Failed to initialize YouTube collector: {}", e));
                                }
                            }
                        } else {
                            logger_for_init
                                .info("YouTube credentials not configured, skipping collector initialization");
                        }

                        // Start polling for existing enabled channels
                        logger_for_init.info("Starting polling for existing enabled channels...");
                        {
                            let mut poller = poller_for_init.lock().await;
                            match start_existing_channels_polling(&db_manager, &mut poller, &app_handle_for_init).await {
                                Ok(count) => {
                                    logger_for_init.info(&format!("Started polling for {} existing enabled channel(s)", count));
                                }
                                Err(e) => {
                                    logger_for_init.error(&format!("Failed to start polling for existing channels: {}", e));
                                }
                            }
                        } // ロックを解放

                        // Initialize AutoDiscoveryPoller
                        logger_for_init.info("Initializing AutoDiscoveryPoller...");
                        let twitch_api_client = if settings.twitch.client_id.is_some() {
                            // Get Twitch API client from the registered collector
                            let poller = poller_for_init.lock().await;
                            poller
                                .get_twitch_collector()
                                .map(|tc| Arc::clone(tc.get_api_client()))
                        } else {
                            None
                        };

                        // Use DatabaseManager for AutoDiscoveryPoller
                        let discovery_poller = AutoDiscoveryPoller::new(
                            twitch_api_client,
                            Arc::new(db_manager.inner().clone()),
                            app_handle_for_init.clone(),
                        );

                        // Start AutoDiscoveryPoller if enabled
                        if let Some(auto_discovery_settings) = &settings.auto_discovery {
                            logger_for_init.info(&format!(
                                "AutoDiscovery settings found - enabled: {}, poll_interval: {}s, max_streams: {}, game_ids: {:?}",
                                auto_discovery_settings.enabled,
                                auto_discovery_settings.poll_interval,
                                auto_discovery_settings.max_streams,
                                auto_discovery_settings.filters.game_ids
                            ));

                            if auto_discovery_settings.enabled {
                                logger_for_init.info("Starting AutoDiscoveryPoller...");
                                match discovery_poller.start().await {
                                    Ok(_) => {
                                        logger_for_init.info("AutoDiscoveryPoller started successfully");
                                    }
                                    Err(e) => {
                                        logger_for_init.error(&format!(
                                            "Failed to start AutoDiscoveryPoller: {}",
                                            e
                                        ));
                                    }
                                }
                            } else {
                                logger_for_init.info("AutoDiscovery is disabled in settings - enable it in Settings > Twitch自動発見");
                            }
                        } else {
                            logger_for_init.info("AutoDiscovery settings not configured - configure it in Settings > Twitch自動発見");
                        }

                        // Store the AutoDiscoveryPoller in app state
                        let auto_discovery_state: tauri::State<'_, Arc<tokio::sync::Mutex<Option<AutoDiscoveryPoller>>>> =
                            app_handle_for_init.state();
                        let mut state = auto_discovery_state.lock().await;
                        *state = Some(discovery_poller);

                        // Emit backend-ready event to notify frontend that all collectors and pollers are initialized
                        let _ = app_handle_for_init.emit("backend-ready", ());
                        logger_for_init.info("Backend fully initialized, frontend queries are now safe");
                    });

                    logger_for_init.info("Application daemon initialization completed");
                })
                .expect("Failed to spawn thread for collector initialization");

            // Initialize system tray icon and menu
            let show_i = MenuItem::with_id(app, "show", "表示", true, None::<&str>)?;
            let update_i = MenuItem::with_id(app, "check_update", "アップデートを確認", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &update_i, &quit_i])?;

            // Clone app_handle for use in tray event handlers before it's moved
            let app_handle_for_menu = app_handle.clone();
            let app_handle_for_tray = app_handle.clone();
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |_app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app_handle_for_menu.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "check_update" => {
                            let app_handle_clone = app_handle_for_menu.clone();
                            tauri::async_runtime::spawn(async move {
                                check_for_updates(app_handle_clone).await;
                            });
                        }
                        "quit" => {
                            // グレースフルシャットダウン
                            if let Some(db_manager) = _app.try_state::<DatabaseManager>() {
                                eprintln!("[App Exit] Performing graceful shutdown...");
                                let _ = tauri::async_runtime::block_on(async {
                                    db_manager.shutdown().await
                                });
                            }
                            _app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = app_handle_for_tray.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            logger.info("System tray initialized successfully");

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let logger = window.state::<AppLogger>();
                logger.info("Window close requested - hiding window instead of exiting");
                // アプリを終了せず、ウィンドウを非表示にする
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Analytics commands
            get_broadcaster_analytics,
            get_game_analytics,
            list_game_categories,
            get_data_availability,
            get_game_daily_stats,
            get_channel_daily_stats,
            // Chat Analytics commands
            get_chat_engagement_timeline,
            detect_chat_spikes,
            get_user_segment_stats,
            get_top_chatters,
            get_time_pattern_stats,
            get_chatter_behavior_stats,
            // Data Science commands
            get_word_frequency_analysis,
            get_emote_analysis,
            get_message_length_stats,
            get_viewer_chat_correlation,
            get_category_change_impact,
            get_chatter_activity_scores,
            detect_anomalies,
            // Channel commands
            add_channel,
            remove_channel,
            update_channel,
            list_channels,
            list_channels_basic,
            toggle_channel,
            // System commands
            is_backend_ready,
            // Chat commands
            get_chat_messages,
            get_chat_messages_around_timestamp,
            // Config commands
            save_token,
            delete_token,
            verify_token,
            get_build_info,
            get_database_init_status,
            recreate_database,
            get_oauth_config,
            save_oauth_config,
            delete_oauth_config,
            has_oauth_config,
            // Database commands
            get_database_info,
            // Discovery commands
            get_auto_discovery_settings,
            save_auto_discovery_settings,
            toggle_auto_discovery,
            get_discovered_streams,
            search_twitch_games,
            get_games_by_ids,
            promote_discovered_channel,
            promote_discovered_channels,
            // Game Category commands
            get_game_categories,
            get_game_category,
            upsert_game_category,
            delete_game_category,
            search_game_categories,
            // SQL commands
            execute_sql,
            list_sql_templates,
            save_sql_template,
            delete_sql_template,
            list_database_tables,
            // OAuth commands
            start_twitch_device_auth,
            poll_twitch_device_token,
            reinitialize_twitch_collector,
            // Stats commands
            get_stream_stats,
            get_realtime_chat_rate,
            get_multiview_realtime_stats,
            // Timeline commands
            get_channel_streams,
            get_stream_timeline,
            get_streams_by_date_range,
            get_suggested_streams_for_comparison,
            // Export commands
            export_to_delimited,
            preview_export_data,
            // Logs commands
            get_logs,
            // Twitch commands
            validate_twitch_channel,
            get_twitch_rate_limit_status,
            // Window commands
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
