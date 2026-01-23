use crate::config::credentials::CredentialManager;
use reqwest::Client;
use twitch_oauth2::Scope;
use twitch_oauth2::ClientId;

// TODO: 環境変数から取得するように変更予定
const TWITCH_CLIENT_ID: &str = "YOUR_CLIENT_ID_HERE"; // ビルド時に env!("TWITCH_CLIENT_ID") から取得

pub struct TwitchOAuth {
    client_id: ClientId,
}

impl TwitchOAuth {
    pub fn new() -> Self {
        Self {
            client_id: ClientId::new(TWITCH_CLIENT_ID.to_string()),
        }
    }

    /// Device Code Flowで認証
    /// 戻り値: (user_code, verification_uri, device_code)
    pub async fn start_device_flow(
        &self,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let http_client = Client::new();

        // スコープを設定
        let scopes = vec![Scope::UserReadEmail, Scope::ChannelReadStreamKey];

        // デバイスコードリクエストを送信
        let response = http_client
            .post("https://id.twitch.tv/oauth2/device")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scopes", &scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get device code: {}", response.status()).into());
        }

        let data: serde_json::Value = response.json().await?;

        let user_code = data["user_code"]
            .as_str()
            .ok_or("Missing user_code")?
            .to_string();
        let verification_uri = data["verification_uri"]
            .as_str()
            .ok_or("Missing verification_uri")?
            .to_string();
        let device_code = data["device_code"]
            .as_str()
            .ok_or("Missing device_code")?
            .to_string();

        Ok((user_code, verification_uri, device_code))
    }

    /// デバイスコードをポーリングしてトークンを取得
    pub async fn poll_for_token(
        &self,
        device_code: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let http_client = Client::new();
        let interval = std::time::Duration::from_secs(5);
        let max_attempts = 60; // 5分間ポーリング

        for _ in 0..max_attempts {
            tokio::time::sleep(interval).await;

            let response = http_client
                .post("https://id.twitch.tv/oauth2/token")
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await?;

            let data: serde_json::Value = response.json().await?;

            // エラーチェック
            if let Some(error) = data["error"].as_str() {
                match error {
                    "authorization_pending" => continue, // まだ承認待ち
                    "slow_down" => {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        continue;
                    }
                    "expired_token" => return Err("Device code expired".into()),
                    _ => return Err(format!("OAuth error: {}", error).into()),
                }
            }

            // トークン取得成功
            if let Some(access_token) = data["access_token"].as_str() {
                // アクセストークンを保存
                CredentialManager::save_token("twitch", access_token)?;

                // リフレッシュトークンがある場合は保存
                if let Some(refresh_token) = data["refresh_token"].as_str() {
                    let _ = CredentialManager::save_token("twitch_refresh", refresh_token);
                }

                return Ok(access_token.to_string());
            }
        }

        Err("Authentication timeout".into())
    }
}
