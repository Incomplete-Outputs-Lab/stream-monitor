use crate::collectors::poller::{ChannelPoller, CollectorStatus};
use crate::database::{models::StreamStats, utils, DatabaseManager};
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, ss.collected_at, ss.viewer_count, ss.chat_rate_1min, ss.category 
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
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let stats: Result<Vec<StreamStats>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StreamStats {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                collected_at: row.get(2)?,
                viewer_count: row.get(3)?,
                chat_rate_1min: row.get(4)?,
                category: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query stats: {}", e))?
        .collect();

    stats.map_err(|e| format!("Failed to collect stats: {}", e))
}

#[tauri::command]
pub async fn get_live_channels(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<crate::database::models::ChannelWithStats>, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // 最新のstream_statsを明示的に取得するように修正
    let sql = r#"
        WITH latest_stats AS (
            SELECT 
                stream_id,
                viewer_count,
                collected_at,
                ROW_NUMBER() OVER (PARTITION BY stream_id ORDER BY collected_at DESC) as rn
            FROM stream_stats
        )
        SELECT 
            c.id, c.platform, c.channel_id, c.channel_name, c.display_name, c.profile_image_url, c.enabled, c.poll_interval, 
            c.follower_count, c.broadcaster_type, c.view_count,
            CAST(c.created_at AS VARCHAR) as created_at, CAST(c.updated_at AS VARCHAR) as updated_at,
            TRUE as is_live,
            ls.viewer_count as current_viewers,
            s.title as current_title
        FROM channels c
        INNER JOIN streams s ON c.id = s.channel_id AND s.ended_at IS NULL
        LEFT JOIN latest_stats ls ON s.id = ls.stream_id AND ls.rn = 1
        WHERE c.enabled = TRUE
        ORDER BY COALESCE(ls.collected_at, s.started_at) DESC
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let channels: Result<Vec<_>, _> = stmt
        .query_map([], |row| {
            Ok(crate::database::models::ChannelWithStats {
                channel: crate::database::models::Channel {
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
                },
                is_live: row.get(13)?,
                current_viewers: row.get(14)?,
                current_title: row.get(15)?,
            })
        })
        .map_err(|e| format!("Failed to query channels: {}", e))?
        .collect();

    channels.map_err(|e| format!("Failed to collect channels: {}", e))
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
