use crate::database::models::ChatMessage;
use crate::database::DatabaseManager;
use crate::logger::AppLogger;
use chrono::Local;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

/// チャンネルごとのIRC接続管理
struct ChannelConnection {
    channel_id: i64,
    channel_name: String,
    stream_id: Arc<Mutex<Option<i64>>>,
    is_connected: Arc<AtomicBool>,
    message_count: Arc<AtomicU64>,
    last_message_at: Arc<Mutex<Option<String>>>,
}

/// 複数のTwitch IRC接続を管理するマネージャー
pub struct TwitchIrcManager {
    channels: Arc<Mutex<HashMap<i64, ChannelConnection>>>,
    client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    logger: Arc<AppLogger>,
}

impl TwitchIrcManager {
    pub fn new(db_manager: Arc<DatabaseManager>, logger: Arc<AppLogger>) -> Self {
        // 匿名ログインの設定
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let client = Arc::new(client);
        let channels: Arc<Mutex<HashMap<i64, ChannelConnection>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let db_manager_clone = Arc::clone(&db_manager);
        let logger_clone = Arc::clone(&logger);
        let channels_clone = Arc::clone(&channels);

        // メッセージ受信タスクを開始（バックグラウンドで継続実行）
        let _incoming_task = tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut last_flush = std::time::Instant::now();

            while let Some(message) = incoming_messages.recv().await {
                match message {
                    ServerMessage::Privmsg(msg) => {
                        // チャンネル名から channel_id と stream_id を取得
                        let channel_name = msg.channel_login.as_str();
                        let channels_lock = channels_clone.lock().await;

                        if let Some(conn) = channels_lock
                            .values()
                            .find(|c| c.channel_name.to_lowercase() == channel_name)
                        {
                            let channel_id = conn.channel_id;
                            let stream_id = *conn.stream_id.lock().await;

                            // 統計を更新
                            conn.message_count.fetch_add(1, Ordering::SeqCst);
                            *conn.last_message_at.lock().await = Some(Local::now().to_rfc3339());

                            // バッジ情報を配列として取得（バッジ名のみ）
                            let badges = if msg.badges.is_empty() {
                                None
                            } else {
                                Some(
                                    msg.badges
                                        .iter()
                                        .map(|badge| badge.name.clone())
                                        .collect::<Vec<String>>(),
                                )
                            };

                            // badge_info（サブスク月数等の詳細情報）を取得
                            let badge_info = if msg.badge_info.is_empty() {
                                None
                            } else {
                                Some(
                                    msg.badge_info
                                        .iter()
                                        .map(|bi| format!("{}:{}", bi.name, bi.version))
                                        .collect::<Vec<_>>()
                                        .join(","),
                                )
                            };

                            let chat_message = ChatMessage {
                                id: None,
                                channel_id: Some(channel_id),
                                stream_id,
                                timestamp: Local::now().to_rfc3339(),
                                platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
                                user_id: Some(msg.sender.id.clone()),
                                user_name: msg.sender.login.clone(),
                                display_name: Some(msg.sender.name.clone()), // Twitch表示名を保存
                                message: msg.message_text.clone(),
                                message_type: "normal".to_string(),
                                badges,
                                badge_info,
                            };

                            batch.push(chat_message);
                        }
                    }
                    ServerMessage::Join(_) => {}
                    ServerMessage::Part(_) => {}
                    ServerMessage::Reconnect(_) => {
                        logger_clone.info("[IRC] Server requested reconnect");
                    }
                    _ => {}
                }

                // バッチフラッシュ（100件または5秒ごと）
                if (batch.len() >= 100 || last_flush.elapsed().as_secs() >= 5) && !batch.is_empty()
                {
                    Self::flush_batch(&db_manager_clone, &mut batch, &logger_clone).await;
                    last_flush = std::time::Instant::now();
                }
            }

            // 残りのメッセージをフラッシュ
            if !batch.is_empty() {
                Self::flush_batch(&db_manager_clone, &mut batch, &logger_clone).await;
            }
        });

        // incoming_taskを保持しないため、バックグラウンドで継続実行される
        // db_managerは直接保持せず、flush_batch内で使用する

        Self {
            channels,
            client,
            logger,
        }
    }

    /// データベース書き込みハンドラーを起動（twitch-ircでは不要だが互換性のために残す）
    pub async fn start_db_handler(&self) {
        self.logger
            .info("[IRC] Database handler is integrated into message receiver");
    }

    /// バッチメッセージをデータベースに書き込み
    async fn flush_batch(
        db_manager: &Arc<DatabaseManager>,
        batch: &mut Vec<ChatMessage>,
        logger: &Arc<AppLogger>,
    ) {
        if batch.is_empty() {
            return;
        }

        let result = db_manager
            .with_connection(|conn| {
                crate::database::writer::DatabaseWriter::insert_chat_messages_batch(conn, batch)
            })
            .await;

        match result {
            Ok(_) => {
                logger.info(&format!(
                    "[IRC] Saved {} chat messages to database",
                    batch.len()
                ));
                batch.clear();
            }
            Err(e) => {
                logger.error(&format!("[IRC] Failed to save chat messages: {}", e));
                // エラー時はバッチを保持して次回再試行
            }
        }
    }

    /// 指定したチャンネルのIRC接続を開始
    pub async fn start_channel_collection(
        &self,
        channel_id: i64,
        channel_name: &str,
        _access_token: &str,
    ) -> Result<(), String> {
        let mut channels = self.channels.lock().await;

        if channels.contains_key(&channel_id) {
            self.logger.info(&format!(
                "[IRC] Channel {} is already connected",
                channel_name
            ));
            return Ok(());
        }

        // チャンネルに参加
        let channel_login = channel_name.to_lowercase();
        self.client
            .join(channel_login.clone())
            .map_err(|e| e.to_string())?;

        let connection = ChannelConnection {
            channel_id,
            channel_name: channel_name.to_string(),
            stream_id: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(AtomicBool::new(true)),
            message_count: Arc::new(AtomicU64::new(0)),
            last_message_at: Arc::new(Mutex::new(None)),
        };

        channels.insert(channel_id, connection);

        self.logger.info(&format!(
            "[IRC] Started collection for channel {} (id: {})",
            channel_name, channel_id
        ));
        Ok(())
    }

    /// 指定したチャンネルのIRC接続を停止
    pub async fn stop_channel_collection(&self, channel_id: i64) -> Result<(), String> {
        let mut channels = self.channels.lock().await;

        if let Some(connection) = channels.remove(&channel_id) {
            // チャンネルから退出
            let channel_login = connection.channel_name.to_lowercase();
            self.client.part(channel_login);

            connection.is_connected.store(false, Ordering::SeqCst);

            self.logger.info(&format!(
                "[IRC] Stopped collection for channel_id: {}",
                channel_id
            ));
            Ok(())
        } else {
            Err(format!(
                "No IRC connection found for channel_id: {}",
                channel_id
            ))
        }
    }

    /// 配信状態変更時にstream_idを更新
    pub async fn update_channel_stream(&self, channel_id: i64, stream_id: Option<i64>) {
        let channels = self.channels.lock().await;

        if let Some(connection) = channels.get(&channel_id) {
            *connection.stream_id.lock().await = stream_id;
            self.logger.info(&format!(
                "[IRC] Updated stream_id to {:?} for channel {}",
                stream_id, connection.channel_name
            ));
        }
    }

    /// アクセストークンを更新（twitch-ircでは認証なし接続のため不要だが互換性のために残す）
    pub async fn update_access_token(&self, _token: String) {
        self.logger
            .info("[IRC] Token update not needed for anonymous connection");
    }
}
