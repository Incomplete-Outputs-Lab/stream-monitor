use crate::database::{get_connection, models::StreamStats, utils};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamStatsQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[tauri::command]
pub async fn get_stream_stats(
    app_handle: AppHandle,
    query: StreamStatsQuery,
) -> Result<Vec<StreamStats>, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, ss.collected_at, ss.viewer_count, ss.chat_rate_1min 
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
            })
        })
        .map_err(|e| format!("Failed to query stats: {}", e))?
        .collect();

    stats.map_err(|e| format!("Failed to collect stats: {}", e))
}

#[tauri::command]
pub async fn get_live_channels(
    app_handle: AppHandle,
) -> Result<Vec<crate::database::models::ChannelWithStats>, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let sql = r#"
        SELECT 
            c.id, c.platform, c.channel_id, c.channel_name, c.enabled, c.poll_interval, CAST(c.created_at AS VARCHAR) as created_at, CAST(c.updated_at AS VARCHAR) as updated_at,
            CASE WHEN s.id IS NOT NULL THEN 1 ELSE 0 END as is_live,
            ss.viewer_count as current_viewers,
            s.title as current_title
        FROM channels c
        LEFT JOIN streams s ON c.id = s.channel_id AND s.ended_at IS NULL
        LEFT JOIN stream_stats ss ON s.id = ss.stream_id
        WHERE c.enabled = 1
        GROUP BY c.id, s.id
        HAVING is_live = 1
        ORDER BY ss.collected_at DESC
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
                    display_name: None,
                    enabled: row.get(4)?,
                    poll_interval: row.get(5)?,
                    created_at: Some(row.get(6)?),
                    updated_at: Some(row.get(7)?),
                },
                is_live: row.get(8)?,
                current_viewers: row.get(9)?,
                current_title: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to query channels: {}", e))?
        .collect();

    channels.map_err(|e| format!("Failed to collect channels: {}", e))
}

#[tauri::command]
pub async fn get_channel_stats(
    app_handle: AppHandle,
    channel_id: i64,
) -> Result<Vec<StreamStats>, String> {
    get_stream_stats(
        app_handle,
        StreamStatsQuery {
            stream_id: None,
            channel_id: Some(channel_id),
            start_time: None,
            end_time: None,
        },
    )
    .await
}
