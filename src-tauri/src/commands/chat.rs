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

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatStatsQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatStats {
    pub total_messages: i64,
    pub unique_users: i64,
    pub messages_per_minute: f64,
    pub top_users: Vec<UserMessageCount>,
    pub message_types: Vec<MessageTypeCount>,
    pub hourly_distribution: Vec<HourlyStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMessageCount {
    pub user_name: String,
    pub message_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageTypeCount {
    pub message_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: i32,
    pub message_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRateQuery {
    pub stream_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub interval_minutes: Option<i32>, // 集計間隔（分）
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRateData {
    pub timestamp: String,
    pub message_count: i64,
    pub interval_minutes: i32,
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
            cm.user_id, cm.user_name, cm.message, cm.message_type
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

#[tauri::command]
pub async fn get_chat_stats(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ChatStatsQuery,
) -> Result<ChatStats, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    // 基本的なWHERE条件を構築
    let mut where_conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(stream_id) = query.stream_id {
        where_conditions.push("cm.stream_id = ?".to_string());
        params.push(stream_id.to_string());
    }

    if let Some(channel_id) = query.channel_id {
        where_conditions.push("s.channel_id = ?".to_string());
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = &query.start_time {
        where_conditions.push("cm.timestamp >= ?".to_string());
        params.push(start_time.clone());
    }

    if let Some(end_time) = &query.end_time {
        where_conditions.push("cm.timestamp <= ?".to_string());
        params.push(end_time.clone());
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // 総メッセージ数を取得
    let total_sql = format!(
        r#"
        SELECT COUNT(*) as total
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        "#,
        where_clause
    );

    let total_messages: i64 =
        utils::query_row_with_params(&conn, &total_sql, &params, |row| row.get(0))
            .db_context("get total messages")
            .map_err(|e| e.to_string())?;

    // ユニークユーザー数を取得
    let unique_users_sql = format!(
        r#"
        SELECT COUNT(DISTINCT user_name) as unique_users
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        "#,
        where_clause
    );

    let unique_users: i64 =
        utils::query_row_with_params(&conn, &unique_users_sql, &params, |row| row.get(0))
            .db_context("get unique users")
            .map_err(|e| e.to_string())?;

    // 1分あたりのメッセージ数を計算（期間内の総メッセージ数 ÷ 期間の分数）
    let messages_per_minute = if total_messages > 0 {
        // 期間の開始と終了を取得
        let time_range_sql = format!(
            r#"
            SELECT
                MIN(cm.timestamp) as start_time,
                MAX(cm.timestamp) as end_time
            FROM chat_messages cm
            INNER JOIN streams s ON cm.stream_id = s.id
            {}
            "#,
            where_clause
        );

        let (start_time, end_time): (Option<String>, Option<String>) =
            utils::query_row_with_params(&conn, &time_range_sql, &params, |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap_or((None, None));

        if let (Some(start), Some(end)) = (start_time, end_time) {
            let start_ms = chrono::DateTime::parse_from_rfc3339(&start)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);
            let end_ms = chrono::DateTime::parse_from_rfc3339(&end)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);

            if end_ms > start_ms {
                let minutes = (end_ms - start_ms) as f64 / (1000.0 * 60.0);
                total_messages as f64 / minutes
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    // トップユーザー（メッセージ数が多い順）
    let top_users_sql = format!(
        r#"
        SELECT user_name, COUNT(*) as count
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        GROUP BY user_name
        ORDER BY count DESC
        LIMIT 10
        "#,
        where_clause
    );

    let mut top_users_stmt = conn
        .prepare(&top_users_sql)
        .db_context("prepare top users query")
        .map_err(|e| e.to_string())?;

    let top_users: Result<Vec<UserMessageCount>, _> =
        utils::query_map_with_params(&mut top_users_stmt, &params, |row| {
            Ok(UserMessageCount {
                user_name: row.get(0)?,
                message_count: row.get(1)?,
            })
        })
        .db_context("query top users")
        .map_err(|e| e.to_string())?
        .collect();

    let top_users = top_users
        .db_context("collect top users")
        .map_err(|e| e.to_string())?;

    // メッセージタイプ別集計
    let message_types_sql = format!(
        r#"
        SELECT message_type, COUNT(*) as count
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        GROUP BY message_type
        ORDER BY count DESC
        "#,
        where_clause
    );

    let mut message_types_stmt = conn
        .prepare(&message_types_sql)
        .db_context("prepare message types query")
        .map_err(|e| e.to_string())?;

    let message_types: Result<Vec<MessageTypeCount>, _> =
        utils::query_map_with_params(&mut message_types_stmt, &params, |row| {
            Ok(MessageTypeCount {
                message_type: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .db_context("query message types")
        .map_err(|e| e.to_string())?
        .collect();

    let message_types = message_types
        .db_context("collect message types")
        .map_err(|e| e.to_string())?;

    // 時間帯別分布
    let hourly_sql = format!(
        r#"
        SELECT
            CAST(strftime('%H', cm.timestamp) AS INTEGER) as hour,
            COUNT(*) as count
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        GROUP BY hour
        ORDER BY hour
        "#,
        where_clause
    );

    let mut hourly_stmt = conn
        .prepare(&hourly_sql)
        .db_context("prepare hourly query")
        .map_err(|e| e.to_string())?;

    let hourly_distribution: Result<Vec<HourlyStats>, _> =
        utils::query_map_with_params(&mut hourly_stmt, &params, |row| {
            Ok(HourlyStats {
                hour: row.get(0)?,
                message_count: row.get(1)?,
            })
        })
        .db_context("query hourly distribution")
        .map_err(|e| e.to_string())?
        .collect();

    let hourly_distribution = hourly_distribution
        .db_context("collect hourly distribution")
        .map_err(|e| e.to_string())?;

    Ok(ChatStats {
        total_messages,
        unique_users,
        messages_per_minute,
        top_users,
        message_types,
        hourly_distribution,
    })
}

#[tauri::command]
pub async fn get_chat_rate(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ChatRateQuery,
) -> Result<Vec<ChatRateData>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let interval_minutes = query.interval_minutes.unwrap_or(1);

    // 基本的なWHERE条件を構築
    let mut where_conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(stream_id) = query.stream_id {
        where_conditions.push("cm.stream_id = ?".to_string());
        params.push(stream_id.to_string());
    }

    if let Some(channel_id) = query.channel_id {
        where_conditions.push("s.channel_id = ?".to_string());
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = &query.start_time {
        where_conditions.push("cm.timestamp >= ?".to_string());
        params.push(start_time.clone());
    }

    if let Some(end_time) = &query.end_time {
        where_conditions.push("cm.timestamp <= ?".to_string());
        params.push(end_time.clone());
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // 時間間隔でグループ化してメッセージ数を集計
    let sql = format!(
        r#"
        SELECT
            strftime('%Y-%m-%dT%H:%M:%S', datetime(cm.timestamp, 'unixepoch', 'start of minute')) as timestamp,
            COUNT(*) as message_count
        FROM chat_messages cm
        INNER JOIN streams s ON cm.stream_id = s.id
        {}
        GROUP BY strftime('%Y-%m-%dT%H:%M:00', cm.timestamp)
        ORDER BY timestamp ASC
        "#,
        where_clause
    );

    let mut stmt = conn
        .prepare(&sql)
        .db_context("prepare chat rate query")
        .map_err(|e| e.to_string())?;

    let chat_rates: Result<Vec<ChatRateData>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(ChatRateData {
                timestamp: row.get(0)?,
                message_count: row.get(1)?,
                interval_minutes,
            })
        })
        .db_context("query chat rates")
        .map_err(|e| e.to_string())?
        .collect();

    chat_rates
        .db_context("collect chat rates")
        .map_err(|e| e.to_string())
}
