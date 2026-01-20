use crate::config::credentials::CredentialManager;
use crate::oauth::server::OAuthServer;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const YOUTUBE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const YOUTUBE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const YOUTUBE_SCOPES: &str = "https://www.googleapis.com/auth/youtube.readonly";

#[derive(Debug, Serialize, Deserialize)]
struct YouTubeTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
    scope: Option<String>,
}

pub struct YouTubeOAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: Client,
}

impl YouTubeOAuth {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            http_client: Client::new(),
        }
    }

    fn generate_code_verifier() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..128).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    fn generate_code_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }

    fn generate_state() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    pub fn build_authorization_url(&self, state: &str, code_challenge: &str) -> String {
        let mut params = HashMap::new();
        params.insert("response_type", "code");
        params.insert("client_id", &self.client_id);
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("scope", YOUTUBE_SCOPES);
        params.insert("state", state);
        params.insert("code_challenge", code_challenge);
        params.insert("code_challenge_method", "S256");
        params.insert("access_type", "offline");
        params.insert("prompt", "consent");

        let query_string: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect();

        format!("{}?{}", YOUTUBE_AUTH_URL, query_string.join("&"))
    }

    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_uri.as_str());
        params.insert("code_verifier", code_verifier);

        let response = self
            .http_client
            .post(YOUTUBE_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Token exchange failed: {}", error_text).into());
        }

        let token_response: YouTubeTokenResponse = response.json().await?;

        // アクセストークンを保存
        CredentialManager::save_token("youtube", &token_response.access_token)?;

        Ok(token_response.access_token)
    }

    pub async fn authenticate(
        &self,
        server: OAuthServer,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // PKCE用のコード生成
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier);
        let state = Self::generate_state();

        // 認証URLを生成
        let auth_url = self.build_authorization_url(&state, &code_challenge);

        // ブラウザで認証URLを開く
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &auth_url])
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(&auth_url).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&auth_url)
                .spawn()?;
        }

        // コールバックを待つ
        let callback = server.start_and_wait_for_callback().await?;

        // エラーチェック
        if let Some(error) = callback.error {
            return Err(format!("OAuth error: {}", error).into());
        }

        // ステート検証
        if callback.state.as_deref() != Some(&state) {
            return Err("State mismatch".into());
        }

        // 認証コードを取得
        let code = callback.code.ok_or("No authorization code received")?;

        // アクセストークンに交換
        self.exchange_code_for_token(&code, &code_verifier).await
    }
}
