use crate::constants::youtube;
use crate::database::models::ChatMessage;
use google_youtube3::api::LiveChatMessage;
use google_youtube3::{hyper_rustls, hyper_util, YouTube};
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
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
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
        if self.live_chat_id.is_none() {
            return Ok(vec![]);
        }

        let live_chat_id = self.live_chat_id.as_ref().unwrap();
        let part = vec![
            youtube::PART_SNIPPET.to_string(),
            youtube::PART_AUTHOR_DETAILS.to_string(),
        ];

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
            channel_id: None, // YouTube の場合は channel_id を保存しない
            stream_id: Some(self.stream_id),
            timestamp: timestamp.to_string(),
            platform: youtube::PLATFORM_NAME.to_string(),
            user_id,
            user_name,
            message: message_text,
            message_type,
            badges: None,     // YouTube の場合は badges を保存しない（現状未対応）
            badge_info: None, // YouTube の場合は badge_info も未対応
        })
    }

    /// メッセージタイプを決定
    fn determine_message_type(
        &self,
        snippet: &google_youtube3::api::LiveChatMessageSnippet,
    ) -> String {
        if snippet.super_chat_details.is_some() {
            youtube::MESSAGE_TYPE_SUPERCHAT.to_string()
        } else if snippet.fan_funding_event_details.is_some() {
            youtube::MESSAGE_TYPE_FAN_FUNDING.to_string()
        } else if snippet.new_sponsor_details.is_some() {
            youtube::MESSAGE_TYPE_SPONSOR.to_string()
        } else {
            youtube::MESSAGE_TYPE_NORMAL.to_string()
        }
    }

    /// 定期的にチャットメッセージを収集してチャンネルに送信
    pub async fn start_collection(
        &mut self,
        message_tx: tokio::sync::mpsc::UnboundedSender<crate::database::models::ChatMessage>,
        poll_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                    let message_count = messages.len();
                    if message_count > 0 {
                        for message in messages {
                            if let Err(e) = message_tx.send(message) {
                                eprintln!("Failed to send YouTube chat message: {}", e);
                                // エラーが発生しても継続
                            }
                        }
                        println!("Sent {} YouTube chat messages to channel", message_count);
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
    message_tx: mpsc::UnboundedSender<ChatMessage>,
    db_handler_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[allow(dead_code)]
impl YouTubeLiveChatCollector {
    pub fn new(
        hub: Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>>,
        db_conn: Arc<Mutex<duckdb::Connection>>,
        stream_id: i64,
    ) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        // データベース書き込みハンドラーを起動
        let db_conn_clone = Arc::clone(&db_conn);
        let db_handler_task = tokio::spawn(async move {
            Self::start_db_handler(db_conn_clone, message_rx).await;
        });

        let client = YouTubeLiveChatClient::new(Arc::clone(&hub), stream_id);
        Self {
            client: Arc::new(Mutex::new(client)),
            message_tx,
            db_handler_task: Arc::new(Mutex::new(Some(db_handler_task))),
        }
    }

    /// ライブチャットIDを設定してコレクションを開始
    pub async fn start_with_video_id(
        &self,
        video_id: &str,
        poll_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = self.client.lock().await;

        // ビデオIDからライブチャットIDを取得
        let video_id_clone = video_id.to_string();
        if let Some(live_chat_id) = client.get_live_chat_id_from_video(&video_id_clone).await? {
            println!(
                "Found live chat ID: {} for video: {}",
                live_chat_id, video_id
            );

            // コレクションを開始 - clientを独立したインスタンスとして作成
            let message_tx_clone = self.message_tx.clone();
            let mut client_for_task = YouTubeLiveChatClient::new(
                Arc::clone(&self.client.lock().await.hub),
                self.client.lock().await.stream_id,
            );
            let collection_task = tokio::spawn(async move {
                if let Err(e) = client_for_task
                    .start_collection(message_tx_clone, poll_interval_secs)
                    .await
                {
                    eprintln!(
                        "YouTube live chat collection failed for video {}: {}",
                        video_id_clone, e
                    );
                }
            });

            let mut db_handler_task = self.db_handler_task.lock().await;
            *db_handler_task = Some(collection_task);

            Ok(())
        } else {
            Err("Live chat not available for this video".into())
        }
    }

    /// コレクションを停止
    pub async fn stop_collection(&self) {
        let mut task = self.db_handler_task.lock().await;
        if let Some(task_handle) = task.take() {
            task_handle.abort();
        }
    }

    /// データベース書き込みハンドラーを起動
    async fn start_db_handler(
        db_conn: Arc<Mutex<duckdb::Connection>>,
        mut message_rx: mpsc::UnboundedReceiver<ChatMessage>,
    ) {
        let mut batch = Vec::new();
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                msg = message_rx.recv() => {
                    match msg {
                        Some(chat_message) => {
                            batch.push(chat_message);
                            // バッチサイズに達したら即座に書き込み
                            if batch.len() >= 100 {
                                Self::flush_batch(&db_conn, &mut batch).await;
                            }
                        }
                        None => break, // チャンネルが閉じられた
                    }
                }
                _ = interval.tick() => {
                    // 定期的にバッチをフラッシュ
                    Self::flush_batch(&db_conn, &mut batch).await;
                }
            }
        }

        // ループ終了時に残りのメッセージを書き込み
        if !batch.is_empty() {
            Self::flush_batch(&db_conn, &mut batch).await;
        }
    }

    /// バッチメッセージをデータベースに書き込み
    async fn flush_batch(db_conn: &Arc<Mutex<duckdb::Connection>>, batch: &mut Vec<ChatMessage>) {
        if batch.is_empty() {
            return;
        }

        let conn = db_conn.lock().await;
        match crate::database::writer::DatabaseWriter::insert_chat_messages_batch(&conn, batch) {
            Ok(_) => {
                println!("Saved {} YouTube chat messages to database", batch.len());
                batch.clear();
            }
            Err(e) => {
                eprintln!("Failed to save YouTube chat messages batch: {}", e);
                // エラーが発生してもバッチはクリアせず、次回再試行
            }
        }
    }
}
