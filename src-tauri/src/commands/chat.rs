use crate::database::{models::ChatMessage, utils, DatabaseManager};
use crate::error::ResultExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessagesQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn get_chat_messages(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ChatMessagesQuery,
) -> Result<Vec<ChatMessage>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let mut sql = String::from(
        r#"
        SELECT
            cm.id, cm.stream_id, cm.timestamp, cm.platform,
            cm.user_id, cm.user_name, cm.message, cm.message_type,
            cm.badges, cm.badge_info, cm.channel_id
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

    utils::query_chat_messages(&conn, &sql, &params)
        .db_context("query chat messages")
        .map_err(|e| e.to_string())
}
