use crate::oauth::{server::OAuthServer, twitch::TwitchOAuth, youtube::YouTubeOAuth};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[tauri::command]
pub async fn login_with_twitch(config: OAuthConfig, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8080);
    let redirect_uri = format!("http://localhost:{}/callback", port);

    let oauth = TwitchOAuth::new(config.client_id, config.client_secret, redirect_uri);
    let server = OAuthServer::new(port);

    oauth
        .authenticate(server)
        .await
        .map_err(|e| format!("Twitch authentication failed: {}", e))
}

#[tauri::command]
pub async fn login_with_youtube(config: OAuthConfig, port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8081);
    let redirect_uri = format!("http://localhost:{}/callback", port);

    let oauth = YouTubeOAuth::new(config.client_id, config.client_secret, redirect_uri);
    let server = OAuthServer::new(port);

    oauth
        .authenticate(server)
        .await
        .map_err(|e| format!("YouTube authentication failed: {}", e))
}
