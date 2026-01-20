use crate::api::youtube_api::YouTubeApiClient;
use crate::api::youtube_live_chat::YouTubeLiveChatCollector;
use crate::collectors::collector_trait::Collector;
use crate::database::models::{Channel, StreamStats};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct YouTubeCollector {
    api_client: Arc<Mutex<YouTubeApiClient>>,
    chat_collectors: Arc<Mutex<HashMap<String, YouTubeLiveChatCollector>>>,
}

#[allow(dead_code)]
impl YouTubeCollector {
    pub async fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let api_client = YouTubeApiClient::new(client_id, client_secret, redirect_uri).await?;
        Ok(Self {
            api_client: Arc::new(Mutex::new(api_client)),
            chat_collectors: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl Collector for YouTubeCollector {
    async fn poll_channel(
        &self,
        channel: &Channel,
    ) -> Result<Option<StreamStats>, Box<dyn std::error::Error>> {
        let mut client = self.api_client.lock().await;

        // チャンネルIDからライブストリームを取得
        let stream_opt = client.get_live_stream(&channel.channel_id).await?;

        if let Some(video) = stream_opt {
            // 視聴者数を取得（statisticsから）
            let viewer_count = video
                .live_streaming_details
                .as_ref()
                .and_then(|details| details.concurrent_viewers)
                .map(|v| v as i32);

            // ストリームIDは動画IDを使用（未使用だが将来的に使用可能）
            let _stream_id = video.id.as_ref().unwrap_or(&String::new()).clone();

            Ok(Some(StreamStats {
                id: None,
                stream_id: 0, // TODO: ストリームIDをデータベースから取得する必要がある
                collected_at: Utc::now().to_rfc3339(),
                viewer_count,
                chat_rate_1min: 0, // Phase 2で実装
            }))
        } else {
            Ok(None)
        }
    }

    async fn start_collection(&self, _channel: &Channel) -> Result<(), Box<dyn std::error::Error>> {
        // 認証はOAuthモジュールで行われているため、ここでは確認のみ
        Ok(())
    }
}

impl YouTubeCollector {
    /// チャット収集を開始（ストリーム開始時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn start_chat_collection(
        &self,
        _stream_id: i64,
        _video_id: &str,
        _poll_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // チャット機能は現在無効化中
        // let mut client = self.api_client.lock().await;
        // let hub = client.get_hub().clone();
        // let chat_collector = YouTubeLiveChatCollector::new(hub, db_conn, stream_id);
        // chat_collector.start_with_video_id(video_id, poll_interval_secs).await?;
        Ok(())
    }

    /// チャット収集を停止（ストリーム終了時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn stop_chat_collection(&self, _video_id: &str) {
        // チャット機能は現在無効化中
        // let mut chat_collectors = self.chat_collectors.lock().await;
        // if let Some(collector) = chat_collectors.remove(video_id) {
        //     collector.stop_collection().await;
        // }
    }
}
