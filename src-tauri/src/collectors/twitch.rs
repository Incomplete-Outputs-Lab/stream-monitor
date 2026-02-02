use crate::api::twitch_api::TwitchApiClient;
use crate::collectors::collector_trait::Collector;
use crate::database::models::{Channel, StreamData};
use crate::logger::AppLogger;
use crate::websocket::twitch_irc::TwitchIrcManager;
use async_trait::async_trait;
use duckdb::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TwitchCollector {
    api_client: Arc<TwitchApiClient>,
    irc_manager: Arc<TwitchIrcManager>,
}

impl TwitchCollector {
    /// Create a new TwitchCollector with AppHandle and IRC support
    pub fn new_with_app(
        client_id: String,
        client_secret: Option<String>,
        app_handle: tauri::AppHandle,
        db_conn: Arc<Mutex<Connection>>,
        logger: Arc<AppLogger>,
    ) -> Self {
        let irc_manager = Arc::new(TwitchIrcManager::new(db_conn, Arc::clone(&logger)));

        Self {
            api_client: Arc::new(
                TwitchApiClient::new(client_id, client_secret).with_app_handle(app_handle),
            ),
            irc_manager,
        }
    }

    /// レート制限トラッカーへのアクセスを提供
    pub fn get_api_client(&self) -> &Arc<TwitchApiClient> {
        &self.api_client
    }

    /// IRC Manager を初期化（DBハンドラーを起動）
    pub async fn initialize_irc(&self) {
        self.irc_manager.start_db_handler().await;
    }
}

#[async_trait]
impl Collector for TwitchCollector {
    async fn poll_channel(
        &self,
        channel: &Channel,
    ) -> Result<Option<StreamData>, Box<dyn std::error::Error + Send + Sync>> {
        // twitch_user_idがあればそれを優先使用、なければloginで取得（後方互換性）
        let user_id_string = if let Some(twitch_user_id) = channel.twitch_user_id {
            twitch_user_id.to_string()
        } else {
            // loginからuser_idを取得
            let user = self
                .api_client
                .get_user_by_login(&channel.channel_id)
                .await?;
            user.id.to_string()
        };

        // 配信情報を取得
        let stream_opt = self
            .api_client
            .get_stream_by_user_id(&user_id_string)
            .await?;

        if let Some(stream) = stream_opt {
            // フォロワー数を取得（エラー時は None）
            let follower_count = match self
                .api_client
                .get_followers_batch(&[user_id_string.as_str()])
                .await
            {
                Ok(results) => results.first().map(|(_, count)| *count),
                Err(e) => {
                    eprintln!(
                        "[TwitchCollector] Failed to get follower count for {}: {}",
                        channel.channel_id, e
                    );
                    None
                }
            };

            // Twitch APIから取得したストリーム情報を構造化して返す
            Ok(Some(StreamData {
                stream_id: stream.id.to_string(),
                title: Some(stream.title.to_string()),
                category: Some(stream.game_name.to_string()),
                thumbnail_url: Some(stream.thumbnail_url.to_string()),
                started_at: stream.started_at.as_str().to_string(),
                viewer_count: Some(stream.viewer_count as i32),
                chat_rate_1min: 0, // Phase 2で実装（チャット機能）
                follower_count,
            }))
        } else {
            // 配信していない場合はNone
            Ok(None)
        }
    }

    async fn start_collection(
        &self,
        _channel: &Channel,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 認証を確認
        self.api_client.authenticate().await?;
        Ok(())
    }
}

impl TwitchCollector {
    /// トークンの有効期限をチェックし、必要に応じてリフレッシュ
    pub async fn check_and_refresh_token_if_needed(
        &self,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let refreshed = self.api_client.check_and_refresh_token_if_needed().await?;

        // トークンが更新された場合、IRC Manager にも反映
        if refreshed {
            let token_result = self
                .api_client
                .get_access_token()
                .await
                .map_err(|e| e.to_string());
            if let Ok(token) = token_result {
                self.irc_manager.update_access_token(token).await;
            }
        }

        Ok(refreshed)
    }

    /// チャット収集を開始（手動登録チャンネルに対して）
    pub async fn start_chat_collection(
        &self,
        channel_id: i64,
        channel_name: &str,
    ) -> Result<(), String> {
        let access_token = self
            .api_client
            .get_access_token()
            .await
            .map_err(|e| e.to_string())?;
        self.irc_manager
            .start_channel_collection(channel_id, channel_name, &access_token)
            .await
            .map_err(|e| e.to_string())
    }

    /// チャット収集を停止
    pub async fn stop_chat_collection(&self, channel_id: i64) -> Result<(), String> {
        self.irc_manager.stop_channel_collection(channel_id).await
    }

    /// 配信状態変更時にstream_idを更新
    pub async fn update_stream_id(&self, channel_id: i64, stream_id: Option<i64>) {
        self.irc_manager
            .update_channel_stream(channel_id, stream_id)
            .await;
    }
}
