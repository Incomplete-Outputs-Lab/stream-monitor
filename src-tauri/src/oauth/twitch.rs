use crate::config::credentials::CredentialManager;
use crate::oauth::server::OAuthServer;
use reqwest::Client;
use twitch_oauth2::{Scope, UserTokenBuilder};
use url::Url;

pub struct TwitchOAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl TwitchOAuth {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    pub async fn authenticate(
        &self,
        server: OAuthServer,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let redirect_url = Url::parse(&self.redirect_uri)?;

        // UserTokenBuilderを作成
        let mut builder = UserTokenBuilder::new(
            self.client_id.clone(),
            self.client_secret.clone(),
            redirect_url.clone(),
        );

        // 必要なスコープを設定
        builder = builder.set_scopes(vec![Scope::UserReadEmail, Scope::ChannelReadStreamKey]);

        // 認証URLとCSRFステートを生成
        let (auth_url, csrf_state) = builder.generate_url();

        // ブラウザで認証URLを開く
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", auth_url.as_str()])
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(auth_url.as_str())
                .spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(auth_url.as_str())
                .spawn()?;
        }

        // コールバックを待つ
        let callback = server.start_and_wait_for_callback().await?;

        // エラーチェック
        if let Some(error) = callback.error {
            return Err(format!("OAuth error: {}", error).into());
        }

        // ステート検証
        if let Some(returned_state) = callback.state {
            if returned_state != csrf_state.secret() {
                return Err("State mismatch".into());
            }
        } else {
            return Err("No state in callback".into());
        }

        // 認証コードを取得
        let code = callback.code.ok_or("No authorization code received")?;

        // アクセストークンに交換
        let http_client = Client::new();
        let token = builder
            .get_user_token(&http_client, csrf_state.secret(), &code)
            .await?;

        let access_token = token.access_token.secret().to_string();

        // アクセストークンを保存
        CredentialManager::save_token("twitch", &access_token)?;

        // リフレッシュトークンがある場合は保存（将来的に使用可能）
        if let Some(refresh_token) = &token.refresh_token {
            // リフレッシュトークンも保存（別のキーで）
            let _ = CredentialManager::save_token("twitch_refresh", refresh_token.secret());
        }

        Ok(access_token)
    }
}
