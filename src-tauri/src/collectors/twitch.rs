use crate::api::twitch_api::TwitchApiClient;
use crate::collectors::collector_trait::Collector;
use crate::database::models::{Channel, StreamData};
use crate::websocket::twitch_irc::TwitchIrcManager;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct TwitchCollector {
    api_client: Arc<TwitchApiClient>,
    irc_manager: Arc<Mutex<Option<TwitchIrcManager>>>,
}

#[allow(dead_code)]
impl TwitchCollector {
    /// Create a new TwitchCollector
    ///
    /// For Device Code Flow (user authentication), client_secret can be None.
    /// For App Access Token (client credentials flow), client_secret is required.
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        Self {
            api_client: Arc::new(TwitchApiClient::new(client_id, client_secret)),
            irc_manager: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new TwitchCollector with AppHandle for Stronghold access
    pub fn new_with_app(client_id: String, client_secret: Option<String>, app_handle: tauri::AppHandle) -> Self {
        Self {
            api_client: Arc::new(TwitchApiClient::new(client_id, client_secret).with_app_handle(app_handle)),
            irc_manager: Arc::new(Mutex::new(None)),
        }
    }

    async fn get_client_id_and_secret() -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
        // TODO: 設定から取得する
        // 現時点では環境変数または設定ファイルから取得する必要がある
        let client_id =
            std::env::var("TWITCH_CLIENT_ID").map_err(|_| "TWITCH_CLIENT_ID not set")?;
        let client_secret = std::env::var("TWITCH_CLIENT_SECRET").ok();
        Ok((client_id, client_secret))
    }

    pub async fn new_from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let (client_id, client_secret) = Self::get_client_id_and_secret().await?;
        Ok(Self::new(client_id, client_secret))
    }
}

#[async_trait]
impl Collector for TwitchCollector {
    async fn poll_channel(
        &self,
        channel: &Channel,
    ) -> Result<Option<StreamData>, Box<dyn std::error::Error>> {
        // ユーザー情報を取得
        let user = self
            .api_client
            .get_user_by_login(&channel.channel_id)
            .await?;

        // 配信情報を取得
        let stream_opt = self
            .api_client
            .get_stream_by_user_id(user.id.as_str())
            .await?;

        if let Some(stream) = stream_opt {
            // Twitch APIから取得したストリーム情報を構造化して返す
            Ok(Some(StreamData {
                stream_id: stream.id.to_string(),
                title: Some(stream.title.to_string()),
                category: Some(stream.game_name.to_string()),
                thumbnail_url: Some(stream.thumbnail_url.to_string()),
                started_at: stream.started_at.as_str().to_string(),
                viewer_count: Some(stream.viewer_count as i32),
                chat_rate_1min: 0, // Phase 2で実装（チャット機能）
            }))
        } else {
            // 配信していない場合はNone
            Ok(None)
        }
    }

    async fn start_collection(&self, _channel: &Channel) -> Result<(), Box<dyn std::error::Error>> {
        // 認証を確認
        self.api_client.authenticate().await?;
        Ok(())
    }
}

impl TwitchCollector {
    /// チャット収集を開始（ストリーム開始時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn start_chat_collection(
        &self,
        _stream_id: i64,
        _channel_name: &str,
        _access_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // チャット機能は現在無効化中
        // let irc_manager_opt = self.irc_manager.lock().await;
        // if let Some(irc_manager) = &*irc_manager_opt {
        //     irc_manager.start_channel_collection(stream_id, channel_name, access_token).await?;
        // }
        Ok(())
    }

    /// チャット収集を停止（ストリーム終了時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn stop_chat_collection(&self, _channel_name: &str) {
        // チャット機能は現在無効化中
        // let irc_manager_opt = self.irc_manager.lock().await;
        // if let Some(irc_manager) = &*irc_manager_opt {
        //     irc_manager.stop_channel_collection(channel_name).await;
        // }
    }
}
