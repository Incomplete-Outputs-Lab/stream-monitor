use crate::collectors::collector_trait::Collector;
use crate::database::{
    models::{Channel, ChannelStatsEvent, Stream, StreamData, StreamStats},
    writer::DatabaseWriter,
    DatabaseManager,
};
use crate::logger::AppLogger;
use chrono::Utc;
use duckdb::Connection;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[derive(Debug, Clone, Serialize)]
pub struct CollectorStatus {
    pub channel_id: i64,
    pub channel_name: String,
    pub platform: String,
    pub is_running: bool,
    pub last_poll_at: Option<String>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
    pub poll_count: u64,
    pub error_count: u64,
}

pub struct ChannelPoller {
    collectors: HashMap<String, Arc<dyn Collector + Send + Sync>>,
    tasks: HashMap<i64, tokio::task::JoinHandle<()>>,
    status_map: Arc<RwLock<HashMap<i64, CollectorStatus>>>,
}

impl ChannelPoller {
    pub fn new() -> Self {
        Self {
            collectors: HashMap::new(),
            tasks: HashMap::new(),
            status_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_all_status(&self) -> Vec<CollectorStatus> {
        if let Ok(status_map) = self.status_map.read() {
            status_map.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn register_collector(
        &mut self,
        platform: String,
        collector: Arc<dyn Collector + Send + Sync>,
    ) {
        self.collectors.insert(platform, collector);
    }

    pub fn start_polling(
        &mut self,
        channel: Channel,
        db_manager: &State<'_, DatabaseManager>,
        app_handle: AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !channel.enabled {
            return Ok(());
        }

        let collector = self
            .collectors
            .get(&channel.platform)
            .ok_or_else(|| format!("No collector for platform: {}", channel.platform))?
            .clone();

        let channel_id = channel.id.unwrap();
        let poll_interval = Duration::from_secs(channel.poll_interval as u64);
        let db_manager = Arc::new(db_manager.inner().clone());

        // Initialize status
        if let Ok(mut status_map) = self.status_map.write() {
            status_map.insert(
                channel_id,
                CollectorStatus {
                    channel_id,
                    channel_name: channel.channel_name.clone(),
                    platform: channel.platform.clone(),
                    is_running: true,
                    last_poll_at: None,
                    last_success_at: None,
                    last_error: None,
                    poll_count: 0,
                    error_count: 0,
                },
            );
        }

        let status_map = Arc::clone(&self.status_map);
        let task = tokio::spawn(async move {
            // Get logger from app_handle
            let logger = app_handle.state::<AppLogger>();

            let mut interval = interval(poll_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            // 初回認証
            if let Err(e) = collector.start_collection(&channel).await {
                logger.error(&format!(
                    "Failed to start collection for channel {}: {}",
                    channel_id, e
                ));
                // Update status with error
                if let Ok(mut map) = status_map.write() {
                    if let Some(status) = map.get_mut(&channel_id) {
                        status.last_error = Some(format!("Failed to start collection: {}", e));
                        status.is_running = false;
                    }
                }
                return;
            }

            loop {
                interval.tick().await;

                // Update last poll time
                let now = Utc::now().to_rfc3339();
                if let Ok(mut map) = status_map.write() {
                    if let Some(status) = map.get_mut(&channel_id) {
                        status.last_poll_at = Some(now.clone());
                        status.poll_count += 1;
                    }
                }

                // チャンネル情報を再取得（更新されている可能性があるため）
                let conn = match db_manager.get_connection() {
                    Ok(conn) => conn,
                    Err(e) => {
                        logger.error(&format!("Failed to get database connection: {}", e));
                        continue;
                    }
                };

                let updated_channel = match Self::get_channel(&conn, channel_id) {
                    Ok(Some(ch)) => ch,
                    Ok(None) => {
                        // チャンネルが削除された場合はタスクを終了
                        logger.info(&format!(
                            "Channel {} was deleted, stopping polling",
                            channel_id
                        ));
                        break;
                    }
                    Err(e) => {
                        logger.error(&format!("Failed to get channel: {}", e));
                        continue;
                    }
                };

                if !updated_channel.enabled {
                    // チャンネルが無効化された場合はタスクを終了
                    logger.info(&format!(
                        "Channel {} was disabled, stopping polling",
                        channel_id
                    ));
                    break;
                }

                // チャンネル情報を定期的に更新（Twitchの場合のみ）
                // 注: 実際のダウンキャストは複雑なため、ここではスキップ
                // 代わりに、start_polling時に一度だけ取得する方式を採用する必要がある

                // ポーリング実行
                match collector.poll_channel(&updated_channel).await {
                    Ok(Some(stream_data)) => {
                        // ストリーム情報をデータベースに保存
                        if let Err(e) = Self::save_stream_data(&conn, channel_id, &stream_data) {
                            logger.error(&format!(
                                "Failed to save stream data for channel {}: {}",
                                channel_id, e
                            ));
                            // Update status with error
                            if let Ok(mut map) = status_map.write() {
                                if let Some(status) = map.get_mut(&channel_id) {
                                    status.last_error = Some(format!("Failed to save data: {}", e));
                                    status.error_count += 1;
                                }
                            }
                        } else {
                            // Update status with success
                            let now = Utc::now().to_rfc3339();
                            if let Ok(mut map) = status_map.write() {
                                if let Some(status) = map.get_mut(&channel_id) {
                                    status.last_success_at = Some(now);
                                    status.last_error = None;
                                }
                            }

                            // イベント発行: チャンネルがライブ中
                            let event = ChannelStatsEvent {
                                channel_id,
                                is_live: true,
                                viewer_count: stream_data.viewer_count,
                                title: stream_data.title.clone(),
                            };
                            let _ = app_handle.emit("channel-stats-updated", event);
                        }
                    }
                    Ok(None) => {
                        // 配信していない - オフラインイベントを発行
                        // Update status with success (not live is valid state)
                        let now = Utc::now().to_rfc3339();
                        if let Ok(mut map) = status_map.write() {
                            if let Some(status) = map.get_mut(&channel_id) {
                                status.last_success_at = Some(now);
                                status.last_error = None;
                            }
                        }

                        let event = ChannelStatsEvent {
                            channel_id,
                            is_live: false,
                            viewer_count: None,
                            title: None,
                        };
                        let _ = app_handle.emit("channel-stats-updated", event);
                    }
                    Err(e) => {
                        logger.error(&format!("Failed to poll channel {}: {}", channel_id, e));
                        // Update status with error
                        if let Ok(mut map) = status_map.write() {
                            if let Some(status) = map.get_mut(&channel_id) {
                                status.last_error = Some(format!("Poll failed: {}", e));
                                status.error_count += 1;
                            }
                        }
                    }
                }
            }
        });

        self.tasks.insert(channel_id, task);
        Ok(())
    }

    pub fn stop_polling(&mut self, channel_id: i64) {
        if let Some(task) = self.tasks.remove(&channel_id) {
            task.abort();
        }

        // Update status to stopped
        if let Ok(mut status_map) = self.status_map.write() {
            if let Some(status) = status_map.get_mut(&channel_id) {
                status.is_running = false;
            }
        }
    }

    fn get_channel(conn: &Connection, channel_id: i64) -> Result<Option<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare("SELECT id, platform, channel_id, channel_name, display_name, profile_image_url, enabled, poll_interval, follower_count, broadcaster_type, view_count, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels WHERE id = ?")?;

        let channel_id_str = channel_id.to_string();
        let rows: Result<Vec<_>, _> = stmt
            .query_map([channel_id_str.as_str()], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: row.get(4)?,
                    profile_image_url: row.get(5)?,
                    enabled: row.get(6)?,
                    poll_interval: row.get(7)?,
                    follower_count: row.get(8).ok(),
                    broadcaster_type: row.get(9).ok(),
                    view_count: row.get(10).ok(),
                    created_at: Some(row.get(11)?),
                    updated_at: Some(row.get(12)?),
                })
            })?
            .collect();

        match rows {
            Ok(mut channels) => Ok(channels.pop()),
            Err(e) => Err(e),
        }
    }

    /// ストリーム統計情報をデータベースに保存する
    fn save_stream_data(
        conn: &Connection,
        channel_id: i64,
        stream_data: &StreamData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // StreamDataから配信情報を含むStreamレコードを作成
        let stream = Stream {
            id: None,
            channel_id,
            stream_id: stream_data.stream_id.clone(),
            title: stream_data.title.clone(),
            category: stream_data.category.clone(),
            thumbnail_url: stream_data.thumbnail_url.clone(),
            started_at: stream_data.started_at.clone(),
            ended_at: None, // ライブ中なのでNone
        };

        // ストリームを保存（同じstream_idの場合は更新）
        let stream_db_id = DatabaseWriter::insert_or_update_stream(conn, channel_id, &stream)?;

        // StreamStatsを作成して保存
        let stats = StreamStats {
            id: None,
            stream_id: stream_db_id,
            collected_at: Utc::now().to_rfc3339(),
            viewer_count: stream_data.viewer_count,
            chat_rate_1min: stream_data.chat_rate_1min,
            category: stream_data.category.clone(),
        };

        // ストリーム統計を保存
        DatabaseWriter::insert_stream_stats(conn, &stats)?;

        Ok(())
    }

    /// チャット収集を開始する（ストリーム開始時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn start_chat_collection(
        &self,
        _channel: &Channel,
        _stream_id: i64,
        _video_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // チャット機能は現在無効化中
        // match channel.platform.as_str() {
        //     "twitch" => {
        //         if let Some(twitch_collector) = self.collectors.get("twitch") {
        //             if let Ok(twitch_collector) = twitch_collector.as_ref().downcast_ref::<crate::collectors::twitch::TwitchCollector>() {
        //                 twitch_collector.start_chat_collection(stream_id, &channel.channel_name, "oauth_token_here").await?;
        //             }
        //         }
        //     }
        //     "youtube" => {
        //         if let Some(youtube_collector) = self.collectors.get("youtube") {
        //             if let Ok(youtube_collector) = youtube_collector.as_ref().downcast_ref::<crate::collectors::youtube::YouTubeCollector>() {
        //                 if let Some(video_id) = video_id {
        //                     youtube_collector.start_chat_collection(stream_id, video_id, 30).await?;
        //                 }
        //             }
        //         }
        //     }
        //     _ => {}
        // }
        Ok(())
    }

    /// チャット収集を停止する（ストリーム終了時に呼び出し）
    /// TODO: チャット機能実装中
    #[allow(dead_code)]
    pub async fn stop_chat_collection(&self, _channel: &Channel, _video_id: Option<&str>) {
        // チャット機能は現在無効化中
        // match channel.platform.as_str() {
        //     "twitch" => {
        //         if let Some(twitch_collector) = self.collectors.get("twitch") {
        //             if let Ok(twitch_collector) = twitch_collector.as_ref().downcast_ref::<crate::collectors::twitch::TwitchCollector>() {
        //                 twitch_collector.stop_chat_collection(&channel.channel_name).await;
        //             }
        //         }
        //     }
        //     "youtube" => {
        //         if let Some(youtube_collector) = self.collectors.get("youtube") {
        //             if let Ok(youtube_collector) = youtube_collector.as_ref().downcast_ref::<crate::collectors::youtube::YouTubeCollector>() {
        //                 if let Some(video_id) = video_id {
        //                     youtube_collector.stop_chat_collection(video_id).await;
        //                 }
        //             }
        //         }
        //     }
        //     _ => {}
        // }
    }
}
