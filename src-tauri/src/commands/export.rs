use crate::database::{
    aggregation::DataAggregator,
    get_connection,
    models::{ChatMessage, StreamStats},
    utils,
};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use serde_json;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub aggregation: Option<String>, // "raw", "1min", "5min", "1hour"
    pub include_chat: Option<bool>,
}

#[tauri::command]
pub async fn export_to_csv(
    app_handle: AppHandle,
    query: ExportQuery,
    file_path: String,
) -> Result<String, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let stats = get_stream_stats_internal(&conn, &query)
        .map_err(|e| format!("Failed to query stats: {}", e))?;

    let stats_len = stats.len();

    // CSV生成
    let mut csv = String::from("id,stream_id,collected_at,viewer_count,chat_rate_1min\n");

    for stat in &stats {
        csv.push_str(&format!(
            "{},{},{},{},{}\n",
            stat.id.unwrap_or(0),
            stat.stream_id,
            stat.collected_at,
            stat.viewer_count.unwrap_or(0),
            stat.chat_rate_1min
        ));
    }

    // ファイルに書き込み
    std::fs::write(&file_path, csv).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!("Exported {} records to {}", stats_len, file_path))
}

#[tauri::command]
pub async fn export_to_json(
    app_handle: AppHandle,
    query: ExportQuery,
    file_path: String,
) -> Result<String, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // ストリーム統計データを取得
    let stats = get_stream_stats_internal(&conn, &query)
        .map_err(|e| format!("Failed to query stats: {}", e))?;

    // 集計処理
    let processed_stats = if let Some(agg) = &query.aggregation {
        match agg.as_str() {
            "1min" => DataAggregator::aggregate_to_1min(&stats),
            "5min" => DataAggregator::aggregate_to_5min(&stats),
            "1hour" => DataAggregator::aggregate_to_1hour(&stats),
            _ => stats
                .into_iter()
                .map(|s| crate::database::aggregation::AggregatedStreamStats {
                    timestamp: s.collected_at,
                    interval_minutes: 0,
                    avg_viewer_count: s.viewer_count.map(|v| v as f64),
                    max_viewer_count: s.viewer_count,
                    min_viewer_count: s.viewer_count,
                    chat_rate_avg: s.chat_rate_1min as f64,
                    data_points: 1,
                })
                .collect(),
        }
    } else {
        stats
            .into_iter()
            .map(|s| crate::database::aggregation::AggregatedStreamStats {
                timestamp: s.collected_at,
                interval_minutes: 0,
                avg_viewer_count: s.viewer_count.map(|v| v as f64),
                max_viewer_count: s.viewer_count,
                min_viewer_count: s.viewer_count,
                chat_rate_avg: s.chat_rate_1min as f64,
                data_points: 1,
            })
            .collect()
    };

    let mut export_data = serde_json::Map::new();
    export_data.insert(
        "stream_stats".to_string(),
        serde_json::to_value(&processed_stats).unwrap(),
    );

    // チャットデータを含む場合
    if query.include_chat.unwrap_or(false) {
        let chat_messages = get_chat_messages_internal(&conn, &query)
            .map_err(|e| format!("Failed to query chat messages: {}", e))?;

        export_data.insert(
            "chat_messages".to_string(),
            serde_json::to_value(&chat_messages).unwrap(),
        );
    }

    // メタデータを追加
    let mut metadata = serde_json::Map::new();
    metadata.insert(
        "exported_at".to_string(),
        chrono::Utc::now().to_rfc3339().into(),
    );
    metadata.insert("total_records".to_string(), processed_stats.len().into());
    if query.include_chat.unwrap_or(false) {
        let chat_count = if let Some(chat_data) = export_data.get("chat_messages") {
            if let Some(arr) = chat_data.as_array() {
                arr.len()
            } else {
                0
            }
        } else {
            0
        };
        metadata.insert("chat_messages_count".to_string(), chat_count.into());
    }
    export_data.insert("metadata".to_string(), serde_json::Value::Object(metadata));

    // JSONファイルに書き込み
    let json_content = serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

    Ok(format!("Exported data to {}", file_path))
}

fn get_stream_stats_internal(
    conn: &Connection,
    query: &ExportQuery,
) -> Result<Vec<StreamStats>, duckdb::Error> {
    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, ss.collected_at, ss.viewer_count, ss.chat_rate_1min 
         FROM stream_stats ss
         INNER JOIN streams s ON ss.stream_id = s.id
         WHERE 1=1",
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(channel_id) = query.channel_id {
        sql.push_str(" AND s.channel_id = ?");
        params.push(channel_id.to_string());
    }

    if let Some(start_time) = &query.start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start_time.clone());
    }

    if let Some(end_time) = &query.end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end_time.clone());
    }

    sql.push_str(" ORDER BY ss.collected_at ASC");

    let mut stmt = conn.prepare(&sql)?;

    let stats: Result<Vec<StreamStats>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StreamStats {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                collected_at: row.get(2)?,
                viewer_count: row.get(3)?,
                chat_rate_1min: row.get(4)?,
            })
        })?
        .collect();

    stats
}

fn get_chat_messages_internal(
    conn: &Connection,
    query: &ExportQuery,
) -> Result<Vec<ChatMessage>, duckdb::Error> {
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

    sql.push_str(" ORDER BY cm.timestamp ASC");

    let mut stmt = conn.prepare(&sql)?;

    let messages: Result<Vec<ChatMessage>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                timestamp: row.get(2)?,
                platform: row.get(3)?,
                user_id: row.get::<_, Option<String>>(4)?,
                user_name: row.get(5)?,
                message: row.get(6)?,
                message_type: row.get(7)?,
            })
        })?
        .collect();

    messages
}
