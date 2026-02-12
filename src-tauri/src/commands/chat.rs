use crate::database::{models::ChatMessage, utils, DatabaseManager};
use crate::error::ResultExt;
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessagesQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyChatQuery {
    pub stream_id: i64,
    pub timestamp: String,
    pub window_minutes: Option<i32>,
}

#[tauri::command]
pub async fn get_chat_messages(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ChatMessagesQuery,
) -> Result<Vec<ChatMessage>, String> {
    eprintln!("[get_chat_messages] Received query: {:?}", query);

    let mut sql = String::from(
        r#"
        SELECT
            cm.id, cm.channel_id, cm.stream_id,
            CAST(cm.timestamp AS VARCHAR) as timestamp,
            cm.platform,
            cm.user_id, cm.user_name, cm.display_name, cm.message, cm.message_type,
            CAST(cm.badges AS VARCHAR) as badges, cm.badge_info
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(stream_id) = query.stream_id {
        sql.push_str(" AND cm.stream_id = ?");
        params.push(stream_id.to_string());
    }

    if let Some(channel_id) = query.channel_id {
        sql.push_str(" AND s.channel_id = ?");
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = &query.start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start_time.clone());
    }

    if let Some(end_time) = &query.end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end_time.clone());
    }

    sql.push_str(" ORDER BY cm.timestamp DESC");

    if let Some(limit) = query.limit {
        sql.push_str(" LIMIT ?");
        params.push(limit.to_string());
    }

    if let Some(offset) = query.offset {
        sql.push_str(" OFFSET ?");
        params.push(offset.to_string());
    }

    eprintln!("[get_chat_messages] SQL: {}", sql);
    eprintln!("[get_chat_messages] Params: {:?}", params);

    let messages = db_manager
        .with_connection(|conn| {
            utils::query_chat_messages(conn, &sql, &params)
                .db_context("query chat messages")
                .map_err(|e| e.to_string())
        })
        .await?;

    eprintln!("[get_chat_messages] Found {} messages", messages.len());

    Ok(messages)
}

#[tauri::command]
pub async fn get_chat_messages_around_timestamp(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: AnomalyChatQuery,
) -> Result<Vec<ChatMessage>, String> {
    // Parse the timestamp as local time (no timezone info)
    let naive_time = NaiveDateTime::parse_from_str(&query.timestamp, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| format!("Invalid timestamp format: {}", e))?;

    // Convert to local timezone
    let anomaly_time = Local
        .from_local_datetime(&naive_time)
        .single()
        .ok_or_else(|| "Ambiguous local time".to_string())?;

    let window_minutes = query.window_minutes.unwrap_or(2);
    let window_duration = Duration::minutes(window_minutes as i64);

    let start_time = (anomaly_time - window_duration).to_rfc3339();
    let end_time = (anomaly_time + window_duration).to_rfc3339();

    eprintln!(
        "[Chat Anomaly] Fetching messages for stream {} around {} (±{}min)",
        query.stream_id, query.timestamp, window_minutes
    );
    eprintln!("[Chat Anomaly] Time window: {} to {}", start_time, end_time);

    let sql = String::from(
        r#"
        SELECT
            cm.id, cm.channel_id, cm.stream_id,
            CAST(cm.timestamp AS VARCHAR) as timestamp,
            cm.platform,
            cm.user_id, cm.user_name, cm.display_name, cm.message, cm.message_type,
            CAST(cm.badges AS VARCHAR) as badges, cm.badge_info
        FROM chat_messages cm
        WHERE cm.stream_id = ?
          AND cm.timestamp >= ?
          AND cm.timestamp <= ?
        ORDER BY cm.timestamp ASC
        "#,
    );

    let params = vec![query.stream_id.to_string(), start_time, end_time];

    let messages = db_manager
        .with_connection(|conn| {
            utils::query_chat_messages(conn, &sql, &params)
                .db_context("query chat messages around timestamp")
                .map_err(|e| e.to_string())
        })
        .await?;

    eprintln!(
        "[Chat Anomaly] Found {} messages in time window",
        messages.len()
    );

    Ok(messages)
}
