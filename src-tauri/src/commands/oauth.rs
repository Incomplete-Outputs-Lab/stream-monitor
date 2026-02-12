use crate::config::settings::SettingsManager;
use crate::error::ResultExt;
use crate::oauth::twitch::{DeviceAuthStatus, TwitchOAuth};
use tauri::{AppHandle, Manager};

/// Twitch Device Code Grant Flow を開始
#[tauri::command]
pub async fn start_twitch_device_auth(app_handle: AppHandle) -> Result<DeviceAuthStatus, String> {
    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let client_id = settings.twitch.client_id.ok_or_else(|| {
        "Twitch Client ID is not configured. Please set it in settings first.".to_string()
    })?;

    eprintln!("[Twitch Device Auth] Starting device authorization flow");
    eprintln!("  - Client ID configured (length: {})", client_id.len());

    // Device Code Flow では Client Secret 不要、redirect_uri も不要
    let oauth = TwitchOAuth::new(client_id, String::new());

    // デバイスフローを開始（スコープを指定）
    let scopes = vec!["user:read:email", "channel:read:stream_key"];

    oauth
        .start_device_flow(scopes)
        .await
        .map_err(|e| format!("Device flow initialization failed: {}", e))
}

/// Twitch Device Code でトークンをポーリング取得
#[tauri::command]
pub async fn poll_twitch_device_token(
    app_handle: AppHandle,
    device_code: String,
    interval: u64,
    client_id: String,
) -> Result<String, String> {
    eprintln!("[Twitch Device Auth] Starting token polling");
    eprintln!("  - Device code length: {}", device_code.len());
    eprintln!("  - Polling interval: {} seconds", interval);

    // TwitchOAuthインスタンスを作成（Client Secret不要）
    let oauth = TwitchOAuth::new(client_id, String::new()).with_app_handle(app_handle.clone());

    oauth
        .poll_for_device_token(&device_code, interval, Some(app_handle))
        .await
        .map_err(|e| format!("Token polling failed: {}", e))
}

/// Twitch Collector を再初期化（トークン設定後に呼び出す）
#[tauri::command]
pub async fn reinitialize_twitch_collector(
    app_handle: AppHandle,
    db_manager: tauri::State<'_, crate::database::DatabaseManager>,
    poller: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::collectors::poller::ChannelPoller>>,
    >,
) -> Result<(), String> {
    use crate::collectors::twitch::TwitchCollector;
    use crate::logger::AppLogger;
    use std::sync::Arc;

    eprintln!("[Reinit] Reinitializing Twitch collector after authentication");

    // 設定を再読み込み
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let client_id = settings
        .twitch
        .client_id
        .ok_or_else(|| "Twitch Client ID not configured".to_string())?;

    eprintln!("[Reinit] Client ID loaded: {}", client_id);

    // 新しいTwitchCollectorを作成
    let logger = app_handle.state::<AppLogger>();
    let collector = Arc::new(TwitchCollector::new_with_app(
        client_id.clone(),
        None,
        app_handle.clone(),
        Arc::new(db_manager.inner().clone()),
        Arc::new(logger.inner().clone()),
    ));

    eprintln!("[Reinit] TwitchCollector created, initializing IRC...");

    // IRC初期化
    collector.initialize_irc().await;

    eprintln!("[Reinit] IRC initialized, registering collector...");

    // ChannelPollerに登録（既存を上書き）
    let mut poller_guard = poller.lock().await;
    poller_guard.register_twitch_collector(collector);

    eprintln!("[Reinit] Twitch collector reinitialized successfully");

    Ok(())
}
