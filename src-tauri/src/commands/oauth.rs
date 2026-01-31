use crate::oauth::twitch::{TwitchOAuth, DeviceAuthStatus};
use crate::config::settings::SettingsManager;
use tauri::AppHandle;

/// Twitch Device Code Grant Flow を開始
#[tauri::command]
pub async fn start_twitch_device_auth(app_handle: AppHandle) -> Result<DeviceAuthStatus, String> {
    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    let client_id = settings.twitch.client_id
        .ok_or_else(|| "Twitch Client ID is not configured. Please set it in settings first.".to_string())?;

    eprintln!("[Twitch Device Auth] Starting device authorization flow");
    eprintln!("  - Client ID configured (length: {})", client_id.len());

    // Device Code Flow では Client Secret 不要、redirect_uri も不要
    let oauth = TwitchOAuth::new(client_id, String::new());
    
    // デバイスフローを開始（スコープを指定）
    let scopes = vec!["user:read:email", "channel:read:stream_key"];
    
    oauth
        .start_device_flow(scopes)
        .await
        .map_err(|e| format!("Failed to start device flow: {}", e))
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
    let oauth = TwitchOAuth::new(client_id, String::new())
        .with_app_handle(app_handle.clone());

    oauth
        .poll_for_device_token(&device_code, interval, Some(app_handle))
        .await
        .map_err(|e| format!("Token polling failed: {}", e))
}
