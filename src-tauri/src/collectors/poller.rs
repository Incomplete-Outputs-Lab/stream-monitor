use crate::collectors::collector_trait::Collector;
use crate::database::{
    get_connection,
    models::{Channel, Stream, StreamStats},
    writer::DatabaseWriter,
};
use chrono::Utc;
use duckdb::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::time::{interval, Duration, MissedTickBehavior};

pub struct ChannelPoller {
    app_handle: AppHandle,
    collectors: HashMap<String, Arc<dyn Collector + Send + Sync>>,
    tasks: HashMap<i64, tokio::task::JoinHandle<()>>,
}

impl ChannelPoller {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            collectors: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn register_collector(
        &mut self,
        platform: String,
        collector: Arc<dyn Collector + Send + Sync>,
    ) {
        self.collectors.insert(platform, collector);
    }

    pub fn start_polling(&mut self, channel: Channel) -> Result<(), Box<dyn std::error::Error>> {
        if !channel.enabled {
            return Ok(());
        }

        let collector = self
            .collectors
            .get(&channel.platform)
            .ok_or_else(|| format!("No collector for platform: {}", channel.platform))?
            .clone();

        let channel_id = channel.id.unwrap();
        let app_handle = self.app_handle.clone();
        let poll_interval = Duration::from_secs(channel.poll_interval as u64);

        let task = tokio::spawn(async move {
            let mut interval = interval(poll_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            // 初回認証
            if let Err(e) = collector.start_collection(&channel).await {
                eprintln!(
                    "Failed to start collection for channel {}: {}",
                    channel_id, e
                );
                return;
            }

            loop {
                interval.tick().await;

                // チャンネル情報を再取得（更新されている可能性があるため）
                let conn = match get_connection(&app_handle) {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("Failed to get database connection: {}", e);
                        continue;
                    }
                };

                let updated_channel = match Self::get_channel(&conn, channel_id) {
                    Ok(Some(ch)) => ch,
                    Ok(None) => {
                        // チャンネルが削除された場合はタスクを終了
                        break;
                    }
                    Err(e) => {
                        eprintln!("Failed to get channel: {}", e);
                        continue;
                    }
                };

                if !updated_channel.enabled {
                    // チャンネルが無効化された場合はタスクを終了
                    break;
                }

                // ポーリング実行
                match collector.poll_channel(&updated_channel).await {
                    Ok(Some(stats)) => {
                        // ストリーム情報をデータベースに保存
                        if let Err(e) = Self::save_stream_stats(&conn, channel_id, &stats) {
                            eprintln!(
                                "Failed to save stream stats for channel {}: {}",
                                channel_id, e
                            );
                        }
                    }
                    Ok(None) => {
                        // 配信していない
                    }
                    Err(e) => {
                        eprintln!("Failed to poll channel {}: {}", channel_id, e);
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
    }

    fn get_channel(conn: &Connection, channel_id: i64) -> Result<Option<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels WHERE id = ?")?;

        let channel_id_str = channel_id.to_string();
        let rows: Result<Vec<_>, _> = stmt
            .query_map([channel_id_str.as_str()], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: None,
                    enabled: row.get(4)?,
                    poll_interval: row.get(5)?,
                    created_at: Some(row.get(6)?),
                    updated_at: Some(row.get(7)?),
                })
            })?
            .collect();

        match rows {
            Ok(mut channels) => Ok(channels.pop()),
            Err(e) => Err(e),
        }
    }

    /// ストリーム統計情報をデータベースに保存する
    fn save_stream_stats(
        conn: &Connection,
        channel_id: i64,
        stats: &StreamStats,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ストリームを作成または更新（現在は簡易実装：channel_id + timestampをstream_idとして使用）
        let stream_id = format!("{}_{}", channel_id, Utc::now().timestamp());

        let stream = Stream {
            id: None,
            channel_id,
            stream_id: stream_id.clone(),
            title: None,         // 現在はタイトル情報なし
            category: None,      // 現在はカテゴリ情報なし
            thumbnail_url: None, // 現在はサムネイル情報なし
            started_at: stats.collected_at.clone(),
            ended_at: None, // ライブ中なのでNone
        };

        // ストリームを保存
        let stream_db_id = DatabaseWriter::insert_or_update_stream(conn, channel_id, &stream)?;

        // StreamStatsに正しいstream_idを設定
        let mut stats_with_id = stats.clone();
        stats_with_id.stream_id = stream_db_id;

        // ストリーム統計を保存
        DatabaseWriter::insert_stream_stats(conn, &stats_with_id)?;

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
