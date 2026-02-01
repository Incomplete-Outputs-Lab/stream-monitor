use crate::database::models::ChatMessage;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tungstenite::connect;
use tungstenite::protocol::Message;
use url::Url;

/// Twitch IRC接続管理構造体
#[allow(dead_code)]
pub struct TwitchIrcClient {
    stream_id: i64,
    channel_name: String,
    access_token: String,
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

#[allow(dead_code)]
impl TwitchIrcClient {
    pub fn new(stream_id: i64, channel_name: String, access_token: String) -> Self {
        Self {
            stream_id,
            channel_name,
            access_token,
            shutdown_tx: None,
        }
    }

    /// Twitch IRCに接続し、チャットメッセージを収集する
    pub async fn connect_and_collect(
        &mut self,
        message_tx: mpsc::UnboundedSender<ChatMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Twitch IRCサーバーに接続
        let url = Url::parse("wss://irc-ws.chat.twitch.tv:443")?;
        let (mut socket, _response) = connect(url)?;

        // 認証
        socket.send(Message::Text(format!("PASS oauth:{}", self.access_token)))?;
        socket.send(Message::Text("NICK justinfan12345".to_string()))?;
        socket.send(Message::Text(format!("JOIN #{}", self.channel_name)))?;

        println!("Connected to Twitch IRC for channel: {}", self.channel_name);

        let channel_name = self.channel_name.clone();
        let stream_id = self.stream_id;
        let socket = Arc::new(tokio::sync::Mutex::new(socket));

        loop {
            let socket_clone = Arc::clone(&socket);
            tokio::select! {
                message = tokio::task::spawn_blocking(move || {
                    let mut socket = socket_clone.blocking_lock();
                    socket.read()
                }) => {
                    match message?? {
                        Message::Text(text) => {
                            // デバッグログ（本番環境では削除）
                            if text.contains("PRIVMSG") {
                                println!("Received chat message for channel: {}", channel_name);
                            }

                            let chat_message_opt = {
                                let text_clone = text.clone();
                                // parse_irc_messageは&selfを必要とするが、ここではselfにアクセスできない
                                // 一時的な解決策として、直接パースする
                                if let Some(privmsg_start) = text_clone.find("PRIVMSG") {
                                    let after_privmsg = &text_clone[privmsg_start..];
                                    if let Some(user_end) = text_clone.find('!') {
                                        let user_name = text_clone[1..user_end].to_string();
                                        if let Some(msg_start) = after_privmsg.find(" :") {
                                            let message_content = after_privmsg[msg_start + 2..].to_string();
                                            Some(ChatMessage {
                                                id: None,
                                                stream_id,
                                                timestamp: Utc::now().to_rfc3339(),
                                                platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
                                                user_id: None,
                                                user_name,
                                                message: message_content,
                                                message_type: "normal".to_string(),
                                            })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };

                            if let Some(chat_message) = chat_message_opt {
                                if let Err(e) = message_tx.send(chat_message) {
                                    eprintln!("Failed to send chat message for channel {}: {}", channel_name, e);
                                    // エラーが発生しても接続を継続
                                }
                            }

                            // PINGに応答して接続を維持
                            if text.starts_with("PING") {
                                let pong_response = "PONG :tmi.twitch.tv\r\n".to_string();
                                let socket_clone = Arc::clone(&socket);
                                match tokio::task::spawn_blocking(move || {
                                    let mut socket = socket_clone.blocking_lock();
                                    socket.send(Message::Text(pong_response))
                                }).await {
                                    Ok(Ok(_)) => {}
                                    Ok(Err(e)) => {
                                        eprintln!("Failed to send PONG for channel {}: {}", channel_name, e);
                                        break;
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to spawn blocking task for PONG: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Message::Close(_) => {
                            println!("Twitch IRC connection closed for channel: {}", channel_name);
                            break;
                        }
                        _ => {}
                    }
                }
                _ = shutdown_rx.recv() => {
                    println!("Shutting down Twitch IRC connection for channel: {}", channel_name);
                    break;
                }
            }
        }

        Ok(())
    }

    /// IRCメッセージをパースしてChatMessageに変換
    fn parse_irc_message(&self, message: &str) -> Option<ChatMessage> {
        // PRIVMSG形式: :user!user@user.tmi.twitch.tv PRIVMSG #channel :message
        if let Some(privmsg_start) = message.find("PRIVMSG") {
            let after_privmsg = &message[privmsg_start..];

            // ユーザー名の抽出
            if let Some(user_end) = message.find('!') {
                let user_name = message[1..user_end].to_string();

                // メッセージ内容の抽出
                if let Some(msg_start) = after_privmsg.find(" :") {
                    let message_content = after_privmsg[msg_start + 2..].to_string();

                    return Some(ChatMessage {
                        id: None,
                        stream_id: self.stream_id,
                        timestamp: Utc::now().to_rfc3339(),
                        platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
                        user_id: None, // Twitch IRCではユーザーIDを取得できない
                        user_name,
                        message: message_content,
                        message_type: "normal".to_string(),
                    });
                }
            }
        }

        None
    }

    /// 接続をシャットダウンする
    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

/// 複数のTwitch IRC接続を管理するマネージャー
#[allow(dead_code)]
pub struct TwitchIrcManager {
    connections: Arc<Mutex<std::collections::HashMap<String, TwitchIrcClient>>>,
    message_tx: mpsc::UnboundedSender<ChatMessage>,
    db_handler_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[allow(dead_code)]
impl TwitchIrcManager {
    pub fn new(db_conn: Arc<Mutex<duckdb::Connection>>) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        // データベース書き込みハンドラーを起動
        let db_conn_clone = Arc::clone(&db_conn);
        let db_handler_task = tokio::spawn(async move {
            Self::start_db_handler(db_conn_clone, message_rx).await;
        });

        Self {
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
            message_tx,
            db_handler_task: Arc::new(Mutex::new(Some(db_handler_task))),
        }
    }

    /// 指定したチャンネルのIRC接続を開始
    pub async fn start_channel_collection(
        &self,
        stream_id: i64,
        channel_name: &str,
        access_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;

        if connections.contains_key(channel_name) {
            return Ok(()); // 既に接続中
        }

        let client = TwitchIrcClient::new(
            stream_id,
            channel_name.to_string(),
            access_token.to_string(),
        );

        let message_tx_clone = self.message_tx.clone();
        let channel_name_clone = channel_name.to_string();
        let mut client_for_task = TwitchIrcClient::new(
            stream_id,
            channel_name.to_string(),
            access_token.to_string(),
        );

        tokio::spawn(async move {
            if let Err(e) = client_for_task.connect_and_collect(message_tx_clone).await {
                eprintln!(
                    "Twitch IRC collection failed for {}: {}",
                    channel_name_clone, e
                );
            }
        });

        connections.insert(channel_name.to_string(), client);
        Ok(())
    }

    /// 指定したチャンネルのIRC接続を停止
    pub async fn stop_channel_collection(&self, channel_name: &str) {
        let mut connections = self.connections.lock().await;
        if let Some(client) = connections.remove(channel_name) {
            client.shutdown();
        }
    }

    /// 全てのIRC接続を停止
    pub async fn stop_all_collections(&self) {
        let mut connections = self.connections.lock().await;
        for client in connections.values() {
            client.shutdown();
        }
        connections.clear();

        // データベースハンドラータスクを停止
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
                println!("Saved {} chat messages to database", batch.len());
                batch.clear();
            }
            Err(e) => {
                eprintln!("Failed to save chat messages batch: {}", e);
                // エラーが発生してもバッチはクリアせず、次回再試行
            }
        }
    }
}
