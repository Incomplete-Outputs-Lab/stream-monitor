use crate::collectors::poller::{ChannelPoller, CollectorStatus};
use crate::database::{models::StreamStats, utils, DatabaseManager};
use crate::error::ResultExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamStatsQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[tauri::command]
pub async fn get_stream_stats(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: StreamStatsQuery,
) -> Result<Vec<StreamStats>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, ss.collected_at, ss.viewer_count, ss.chat_rate_1min, ss.category, ss.title, ss.follower_count, ss.twitch_user_id, ss.channel_name 
         FROM stream_stats ss
         INNER JOIN streams s ON ss.stream_id = s.id
         WHERE 1=1",
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(stream_id) = query.stream_id {
        sql.push_str(" AND ss.stream_id = ?");
        params.push(stream_id.to_string());
    }

    if let Some(channel_id) = query.channel_id {
        sql.push_str(" AND s.channel_id = ?");
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = query.start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start_time);
    }

    if let Some(end_time) = query.end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end_time);
    }

    sql.push_str(" ORDER BY ss.collected_at DESC");

    let mut stmt = conn
        .prepare(&sql)
        .db_context("prepare statement")
        .map_err(|e| e.to_string())?;

    let stats: Result<Vec<StreamStats>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StreamStats {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                collected_at: row.get(2)?,
                viewer_count: row.get(3)?,
                chat_rate_1min: row.get(4)?,
                category: row.get(5)?,
                title: row.get(6)?,
                follower_count: row.get(7)?,
                twitch_user_id: row.get(8)?,
                channel_name: row.get(9)?,
            })
        })
        .db_context("query stats")
        .map_err(|e| e.to_string())?
        .collect();

    stats.db_context("collect stats").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_channel_stats(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    channel_id: i64,
) -> Result<Vec<StreamStats>, String> {
    get_stream_stats(
        app_handle,
        db_manager,
        StreamStatsQuery {
            stream_id: None,
            channel_id: Some(channel_id),
            start_time: None,
            end_time: None,
        },
    )
    .await
}

#[tauri::command]
pub async fn get_collector_status(
    poller: State<'_, std::sync::Arc<Mutex<ChannelPoller>>>,
) -> Result<Vec<CollectorStatus>, String> {
    let poller = poller.lock().await;
    Ok(poller.get_all_status())
}
