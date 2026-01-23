use crate::oauth::{server::OAuthServer, twitch::TwitchOAuth, youtube::YouTubeOAuth};
use crate::config::credentials::CredentialManager;
use crate::config::settings::SettingsManager;
use tauri::AppHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
}

/// Device Code Flowを開始してユーザーコードを取得
#[tauri::command]
pub async fn start_twitch_device_flow() -> Result<DeviceCodeResponse, String> {
    let oauth = TwitchOAuth::new();

    let (user_code, verification_uri, device_code) = oauth
        .start_device_flow()
        .await
        .map_err(|e| format!("Failed to start device flow: {}", e))?;

    Ok(DeviceCodeResponse {
        user_code,
        verification_uri,
        device_code,
    })
}

/// デバイスコードをポーリングしてトークンを取得
#[tauri::command]
pub async fn poll_twitch_token(device_code: String) -> Result<String, String> {
    let oauth = TwitchOAuth::new();

    oauth
        .poll_for_token(&device_code)
        .await
        .map_err(|e| format!("Failed to get token: {}", e))
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
