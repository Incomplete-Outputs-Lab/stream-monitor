use crate::config::credentials::CredentialManager;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const TWITCH_DEVICE_URL: &str = "https://id.twitch.tv/oauth2/device";

#[derive(Debug, Serialize, Deserialize)]
struct TwitchTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
    scope: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    expires_in: u64,
    interval: u64,
    user_code: String,
    verification_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceAuthStatus {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub device_code: String,
    pub interval: u64,
}

pub struct TwitchOAuth {
    client_id: String,
    http_client: Client,
}

impl TwitchOAuth {
    pub fn new(client_id: String, _unused_redirect_uri: String) -> Self {
        Self {
            client_id,
            http_client: Client::new(),
        }
    }

    /// Device Code Grant Flow を開始
    /// 
    /// デバイスコードとユーザーコードを取得します。
    /// ユーザーはブラウザで verification_uri にアクセスして user_code を入力します。
    pub async fn start_device_flow(&self, scopes: Vec<&str>) -> Result<DeviceAuthStatus, Box<dyn std::error::Error>> {
        let scope_string = scopes.join(" ");
        
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("scopes", scope_string.as_str());

        eprintln!("[Twitch Device Flow] Starting device authorization flow");
        eprintln!("  - Client ID length: {}", self.client_id.len());
        eprintln!("  - Scopes: {}", scope_string);

        let response = self
            .http_client
            .post(TWITCH_DEVICE_URL)
            .form(&params)
            .send()
            .await?;

        let status = response.status();
        eprintln!("[Twitch Device Flow] Device code request status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("[Twitch Device Flow] Device code error response: {}", error_text);
            return Err(format!("Device code request failed: {}", error_text).into());
        }

        let device_response: DeviceCodeResponse = response.json().await?;

        eprintln!("[Twitch Device Flow] Device code obtained successfully");
        eprintln!("  - User code: {}", device_response.user_code);
        eprintln!("  - Verification URI: {}", device_response.verification_uri);
        eprintln!("  - Expires in: {} seconds", device_response.expires_in);
        eprintln!("  - Polling interval: {} seconds", device_response.interval);

        Ok(DeviceAuthStatus {
            user_code: device_response.user_code,
            verification_uri: device_response.verification_uri,
            expires_in: device_response.expires_in,
            device_code: device_response.device_code,
            interval: device_response.interval,
        })
    }

    /// Device Code を使用してアクセストークンを取得
    /// 
    /// この関数は1回だけ呼び出され、内部でポーリングを行います。
    /// ユーザーが認証を完了するまで待機します。
    pub async fn poll_for_device_token(
        &self,
        device_code: &str,
        interval_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("device_code", device_code);
        params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

        eprintln!("[Twitch Device Flow] Starting token polling");
        eprintln!("  - Polling interval: {} seconds", interval_secs);

        // ポーリング開始
        loop {
            // 指定された間隔で待機
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

            let response = self
                .http_client
                .post(TWITCH_TOKEN_URL)
                .form(&params)
                .send()
                .await?;

            let status = response.status();

            if status.is_success() {
                // トークン取得成功
                let token_response: TwitchTokenResponse = response.json().await?;

                eprintln!("[Twitch Device Flow] Token obtained successfully");

                // アクセストークンを保存
                CredentialManager::save_token("twitch", &token_response.access_token)?;

                // リフレッシュトークンがある場合は保存
                if let Some(refresh_token) = &token_response.refresh_token {
                    CredentialManager::save_token("twitch_refresh", refresh_token)?;
                    eprintln!("[Twitch Device Flow] Refresh token saved");
                }

                return Ok(token_response.access_token);
            } else {
                // エラーレスポンスをパース
                let error_text = response.text().await?;
                eprintln!("[Twitch Device Flow] Polling response: {}", error_text);

                // JSON としてパース
                if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                        match message {
                            "authorization_pending" => {
                                // ユーザーがまだ認証していない - 継続
                                eprintln!("[Twitch Device Flow] Authorization pending, continuing to poll...");
                                continue;
                            }
                            "slow_down" => {
                                // ポーリングが速すぎる - 間隔を延長
                                eprintln!("[Twitch Device Flow] Slow down requested, increasing interval");
                                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
                                continue;
                            }
                            "expired_token" | "invalid device code" => {
                                // デバイスコードが期限切れまたは無効
                                return Err(format!("Device code error: {}", message).into());
                            }
                            "access_denied" => {
                                // ユーザーが認証を拒否
                                return Err("User denied authorization".into());
                            }
                            _ => {
                                return Err(format!("Unknown error: {}", message).into());
                            }
                        }
                    }
                }

                // JSONパースに失敗した場合
                return Err(format!("Token polling failed: {}", error_text).into());
            }
        }
    }

    /// Device Code Flow用のリフレッシュトークン更新
    /// 
    /// Device Code Flow のリフレッシュトークンは1回限り使用で、Client Secret不要
    pub async fn refresh_device_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        // リフレッシュトークンを取得
        let refresh_token = CredentialManager::get_token("twitch_refresh")
            .map_err(|_| "No refresh token found")?;

        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token.as_str());
        // Device Code Flow では Client Secret は不要

        eprintln!("[Twitch Device Flow] Refreshing access token");

        let response = self
            .http_client
            .post(TWITCH_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("[Twitch Device Flow] Token refresh error: {}", error_text);
            return Err(format!("Token refresh failed: {}", error_text).into());
        }

        let token_response: TwitchTokenResponse = response.json().await?;

        eprintln!("[Twitch Device Flow] Token refreshed successfully");

        // 新しいアクセストークンを保存
        CredentialManager::save_token("twitch", &token_response.access_token)?;

        // 新しいリフレッシュトークンがある場合は保存（1回限り使用）
        if let Some(new_refresh_token) = &token_response.refresh_token {
            CredentialManager::save_token("twitch_refresh", new_refresh_token)?;
            eprintln!("[Twitch Device Flow] New refresh token saved (one-time use)");
        }

        Ok(token_response.access_token)
    }
}
