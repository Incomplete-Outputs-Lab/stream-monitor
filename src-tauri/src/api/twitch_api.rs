use crate::config::stronghold_store::StrongholdStore;
use crate::oauth::twitch::TwitchOAuth;
use std::sync::Arc;
use twitch_api::{
    helix::{
        streams::{GetStreamsRequest, Stream},
        users::{GetUsersRequest, User},
        HelixClient,
    },
    twitch_oauth2::{AccessToken, UserToken as TwitchApiUserToken},
    types,
};
use twitch_oauth2::{AppAccessToken, ClientId, ClientSecret};

pub struct TwitchApiClient {
    client: Arc<HelixClient<'static, reqwest::Client>>,
    client_id: String,
    client_secret: Option<String>,
    app_handle: Option<tauri::AppHandle>,
}

impl TwitchApiClient {
    /// Create a new TwitchApiClient
    ///
    /// For Device Code Flow (user authentication), client_secret can be None.
    /// For App Access Token (client credentials flow), client_secret is required.
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        let client = Arc::new(HelixClient::default());

        Self {
            client,
            client_id,
            client_secret,
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    async fn get_access_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        // Strongholdからトークンを取得を試みる（Device Code Flowで取得したユーザートークン）
        if let Some(ref handle) = self.app_handle {
            if let Ok(token_str) = StrongholdStore::get_token_with_app(handle, "twitch") {
                return Ok(AccessToken::from(token_str));
            }
        }

        // Client Secretがある場合のみ、OAuth 2.0 Client Credentials Flowを試行
        if let Some(ref client_secret) = self.client_secret {
            let client_id = ClientId::new(self.client_id.clone());
            let client_secret = ClientSecret::new(client_secret.clone());

            let http_client = reqwest::Client::new();
            let app_token =
                AppAccessToken::get_app_access_token(&http_client, client_id, client_secret, vec![])
                    .await?;

            let access_token_str = app_token.access_token.secret().to_string();

            // トークンを保存
            if let Some(ref handle) = self.app_handle {
                StrongholdStore::save_token_with_app(handle, "twitch", &access_token_str)?;
            }

            return Ok(AccessToken::from(access_token_str));
        }

        // トークンが見つからず、Client Secretもない場合はエラー
        Err("No Twitch access token found. Please authenticate using Device Code Flow first.".into())
    }

    /// トークンをリフレッシュ
    async fn refresh_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        // TwitchOAuthインスタンスを作成してリフレッシュ
        let mut oauth = TwitchOAuth::new(
            self.client_id.clone(),
            String::new(), // Device Code Flowでは不要
        );

        if let Some(ref handle) = self.app_handle {
            oauth = oauth.with_app_handle(handle.clone());
        }

        // リフレッシュ時はイベント通知なし（バックグラウンド処理のため）
        let new_token = oauth.refresh_device_token(self.app_handle.clone()).await?;
        Ok(AccessToken::from(new_token))
    }

    async fn get_user_token(&self) -> Result<TwitchApiUserToken, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        // twitch_apiのUserTokenに変換
        TwitchApiUserToken::from_token(&*self.client, access_token)
            .await
            .map_err(|e| format!("Failed to create UserToken: {}", e).into())
    }

    pub async fn authenticate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // トークンの取得と検証
        let _token = self.get_access_token().await?;
        Ok(())
    }

    pub async fn get_user_by_login(&self, login: &str) -> Result<User, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let login_refs: &[&types::UserNameRef] = &[login.into()];
        let request = GetUsersRequest::logins(login_refs);

        match self.client.req_get(request, &token).await {
            Ok(response) => response
                .data
                .into_iter()
                .next()
                .ok_or_else(|| "User not found".into()),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains("401") || e.to_string().contains("Unauthorized") {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;
                    
                    let response = self.client.req_get(GetUsersRequest::logins(login_refs), &refreshed_token).await?;
                    response
                        .data
                        .into_iter()
                        .next()
                        .ok_or_else(|| "User not found".into())
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub async fn get_stream_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<Stream>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let user_id_refs: &[&types::UserIdRef] = &[user_id.into()];
        let request = GetStreamsRequest::user_ids(user_id_refs);

        match self.client.req_get(request, &token).await {
            Ok(response) => Ok(response.data.into_iter().next()),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains("401") || e.to_string().contains("Unauthorized") {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;
                    
                    let response = self.client.req_get(GetStreamsRequest::user_ids(user_id_refs), &refreshed_token).await?;
                    Ok(response.data.into_iter().next())
                } else {
                    Err(e.into())
                }
            }
        }
    }
}

// 既存コードとの互換性のため、レートリミッターは残しておく（将来的に使用可能）
#[allow(dead_code)]
pub struct RateLimiter {
    // 将来的に実装
}
