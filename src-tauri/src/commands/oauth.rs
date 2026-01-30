use crate::oauth::{server::OAuthServer, twitch::{TwitchOAuth, DeviceAuthStatus}, youtube::YouTubeOAuth};
use crate::config::credentials::CredentialManager;
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
    _app_handle: AppHandle,
    device_code: String,
    interval: u64,
    client_id: String,
) -> Result<String, String> {
    eprintln!("[Twitch Device Auth] Starting token polling");
    eprintln!("  - Device code length: {}", device_code.len());
    eprintln!("  - Polling interval: {} seconds", interval);

    // TwitchOAuthインスタンスを作成（Client Secret不要）
    let oauth = TwitchOAuth::new(client_id, String::new());

    oauth
        .poll_for_device_token(&device_code, interval)
        .await
        .map_err(|e| format!("Token polling failed: {}", e))
}

#[tauri::command]
pub async fn login_with_youtube(app_handle: AppHandle, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8081);
    let redirect_uri = format!("http://localhost:{}/callback", port);

    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    let client_id = settings.youtube.client_id
        .ok_or_else(|| "YouTube Client ID is not configured. Please set it in settings first.".to_string())?;

    // keyringからClient Secretを取得
    let client_secret = CredentialManager::get_oauth_secret("youtube")
        .map_err(|e| format!("Failed to get YouTube Client Secret: {}. Please configure OAuth settings first.", e))?;

    let oauth = YouTubeOAuth::new(client_id, client_secret, redirect_uri);
    let server = OAuthServer::new(port);

    oauth
        .authenticate(server)
        .await
        .map_err(|e| format!("YouTube authentication failed: {}", e))
}
