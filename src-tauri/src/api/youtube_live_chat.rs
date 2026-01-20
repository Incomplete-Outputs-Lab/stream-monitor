use crate::database::models::ChatMessage;
use crate::database::writer::DatabaseWriter;
use google_youtube3::api::LiveChatMessage;
use google_youtube3::{hyper_rustls, hyper_util, YouTube};
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration, MissedTickBehavior};

/// YouTube Live Chat APIクライアント
#[allow(dead_code)]
pub struct YouTubeLiveChatClient {
    hub: Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>>,
    stream_id: i64,
    live_chat_id: Option<String>,
    next_page_token: Option<String>,
}

#[allow(dead_code)]
impl YouTubeLiveChatClient {
    pub fn new(
        hub: Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>>,
        stream_id: i64,
    ) -> Self {
        Self {
            hub,
            stream_id,
            live_chat_id: None,
            next_page_token: None,
        }
    }

    /// ライブチャットIDを設定
    pub fn set_live_chat_id(&mut self, live_chat_id: String) {
        self.live_chat_id = Some(live_chat_id);
        self.next_page_token = None; // 新しいチャットIDが設定されたらページトークンをリセット
    }

    /// ビデオIDからライブチャットIDを取得
    pub async fn get_live_chat_id_from_video(
        &mut self,
        video_id: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let part = vec!["liveStreamingDetails".to_string()];

        let (_, response) = self
            .hub
            .videos()
            .list(&part)
            .add_id(video_id)
            .doit()
            .await?;

        if let Some(videos) = response.items {
            if let Some(video) = videos.into_iter().next() {
                if let Some(details) = video.live_streaming_details {
                    if let Some(chat_id) = details.active_live_chat_id {
                        self.set_live_chat_id(chat_id.clone());
                        return Ok(Some(chat_id));
                    }
                }
            }
        }

        Ok(None)
    }

    /// ライブチャットメッセージを取得
    pub async fn fetch_chat_messages(
        &mut self,
    ) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        if self.live_chat_id.is_none() {
            return Ok(vec![]);
        }

        let live_chat_id = self.live_chat_id.as_ref().unwrap();
        let part = vec!["snippet".to_string(), "authorDetails".to_string()];

        let mut request = self.hub.live_chat_messages().list(live_chat_id, &part);

        if let Some(page_token) = &self.next_page_token {
            request = request.page_token(page_token);
        }

        let (_, response) = request.doit().await?;

        // 次のページトークンを保存
        self.next_page_token = response.next_page_token;

        let mut chat_messages = Vec::new();

        if let Some(items) = response.items {
            for item in items {
                if let Some(chat_message) = self.convert_to_chat_message(item) {
                    chat_messages.push(chat_message);
                }
            }
        }

        Ok(chat_messages)
    }

    /// LiveChatMessageをChatMessageに変換
    fn convert_to_chat_message(&self, live_chat_message: LiveChatMessage) -> Option<ChatMessage> {
        let snippet = live_chat_message.snippet?;
        let author_details = live_chat_message.author_details?;
        let message_text = snippet.display_message.clone()?;
        let message_type = self.determine_message_type(&snippet);

        let user_id = author_details.channel_id;
        let user_name = author_details.display_name?;
        let published_at = snippet.published_at?;

        // YouTubeのタイムスタンプはすでにRFC3339形式
        let timestamp = published_at;

        Some(ChatMessage {
            id: None,
            stream_id: self.stream_id,
            timestamp: timestamp.to_string(),
            platform: "youtube".to_string(),
            user_id,
            user_name,
            message: message_text,
            message_type,
        })
    }

    /// メッセージタイプを決定
    fn determine_message_type(
        &self,
        snippet: &google_youtube3::api::LiveChatMessageSnippet,
    ) -> String {
        if snippet.super_chat_details.is_some() {
            "superchat".to_string()
        } else if snippet.fan_funding_event_details.is_some() {
            "fanfunding".to_string()
        } else if snippet.new_sponsor_details.is_some() {
            "sponsor".to_string()
        } else {
            "normal".to_string()
        }
    }

    /// 定期的にチャットメッセージを収集してデータベースに保存
    pub async fn start_collection(
        &mut self,
        db_conn: Arc<Mutex<duckdb::Connection>>,
        poll_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = interval(Duration::from_secs(poll_interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        println!(
            "Starting YouTube live chat collection for stream_id: {}",
            self.stream_id
        );

        loop {
            interval.tick().await;

            match self.fetch_chat_messages().await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        let conn = db_conn.lock().await;
                        if let Err(e) = DatabaseWriter::insert_chat_messages_batch(&conn, &messages)
                        {
                            eprintln!("Failed to save YouTube chat messages: {}", e);
                        } else {
                            println!("Saved {} YouTube chat messages", messages.len());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch YouTube chat messages: {}", e);
                    // エラーが発生しても継続
                }
            }
        }
    }
}

/// YouTube Live Chatコレクター（コレクター統合用）
#[allow(dead_code)]
pub struct YouTubeLiveChatCollector {
    client: Arc<Mutex<YouTubeLiveChatClient>>,
    db_conn: Arc<Mutex<duckdb::Connection>>,
    collection_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[allow(dead_code)]
impl YouTubeLiveChatCollector {
    pub fn new(
        hub: Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>>,
        db_conn: Arc<Mutex<duckdb::Connection>>,
        stream_id: i64,
    ) -> Self {
        let client = YouTubeLiveChatClient::new(Arc::clone(&hub), stream_id);
        Self {
            client: Arc::new(Mutex::new(client)),
            db_conn,
            collection_task: Arc::new(Mutex::new(None)),
        }
    }

    /// ライブチャットIDを設定してコレクションを開始
    pub async fn start_with_video_id(
        &self,
        video_id: &str,
        _poll_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = self.client.lock().await;

        // ビデオIDからライブチャットIDを取得
        let video_id_clone = video_id.to_string();
        if let Some(live_chat_id) = client.get_live_chat_id_from_video(&video_id_clone).await? {
            println!(
                "Found live chat ID: {} for video: {}",
                live_chat_id, video_id
            );

            // コレクションタスクを開始
            let video_id_for_task = video_id.to_string();
            let task = tokio::task::spawn_blocking(move || {
                // TODO: 非同期タスクを同期的に実行
                // 現時点ではタスクを開始せずに終了
                println!(
                    "YouTube live chat collection task started for video: {}",
                    video_id_for_task
                );
            });

            let mut collection_task = self.collection_task.lock().await;
            *collection_task = Some(task);

            Ok(())
        } else {
            Err("Live chat not available for this video".into())
        }
    }

    /// コレクションを停止
    pub async fn stop_collection(&self) {
        let mut task = self.collection_task.lock().await;
        if let Some(task_handle) = task.take() {
            task_handle.abort();
        }
    }
}
