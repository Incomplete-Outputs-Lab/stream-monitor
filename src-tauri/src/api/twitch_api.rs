use crate::config::credentials::CredentialManager;
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
    client_secret: String,
}

impl TwitchApiClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let client = Arc::new(HelixClient::default());

        Self {
            client,
            client_id,
            client_secret,
        }
    }

    async fn get_access_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        // OS キーチェーンからトークンを取得を試みる
        if let Ok(token_str) = CredentialManager::get_token("twitch") {
            return Ok(AccessToken::from(token_str));
        }

        // OAuth 2.0 Client Credentials Flow
        let client_id = ClientId::new(self.client_id.clone());
        let client_secret = ClientSecret::new(self.client_secret.clone());

        let http_client = reqwest::Client::new();
        let app_token =
            AppAccessToken::get_app_access_token(&http_client, client_id, client_secret, vec![])
                .await?;

        let access_token_str = app_token.access_token.secret().to_string();

        // トークンを保存
        CredentialManager::save_token("twitch", &access_token_str)?;

        Ok(AccessToken::from(access_token_str))
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

        let response = self.client.req_get(request, &token).await?;

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| "User not found".into())
    }

    pub async fn get_stream_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<Stream>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let user_id_refs: &[&types::UserIdRef] = &[user_id.into()];
        let request = GetStreamsRequest::user_ids(user_id_refs);

        let response = self.client.req_get(request, &token).await?;
        Ok(response.data.into_iter().next())
    }
}

// 既存コードとの互換性のため、レートリミッターは残しておく（将来的に使用可能）
#[allow(dead_code)]
pub struct RateLimiter {
    // 将来的に実装
}
