use crate::database::{
    models::StreamStats, repositories::ChannelRepository, utils, DatabaseManager,
};
use crate::error::ResultExt;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub channel_id: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub aggregation: Option<String>, // "raw", "1min", "5min", "1hour"
    pub delimiter: Option<String>,   // Custom delimiter (default: comma)
}

fn get_stream_stats_internal(
    conn: &Connection,
    query: &ExportQuery,
) -> Result<Vec<StreamStats>, duckdb::Error> {
    let mut sql = String::from(
        "SELECT ss.id, ss.stream_id, CAST(ss.collected_at AS VARCHAR) as collected_at, ss.viewer_count,
         COALESCE((
             SELECT COUNT(*)
             FROM chat_messages cm
             WHERE cm.stream_id = ss.stream_id
               AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
               AND cm.timestamp < ss.collected_at
         ), 0) AS chat_rate_1min,
         ss.category, ss.title, ss.follower_count, ss.twitch_user_id, ss.channel_name
         FROM stream_stats ss
         INNER JOIN streams s ON ss.stream_id = s.id
         WHERE 1=1",
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(channel_id) = query.channel_id {
        eprintln!("[Export Debug] Filtering by channel_id: {}", channel_id);

        // Debug: Check if channel exists and has streams
        let channel_check = ChannelRepository::get_channel_info(conn, channel_id);
        match channel_check {
            Ok((ch_id, stream_count)) => {
                eprintln!(
                    "[Export Debug] Channel found: platform_id={}, streams={}",
                    ch_id, stream_count
                );
            }
            Err(e) => {
                eprintln!("[Export Debug] Channel not found or error: {:?}", e);
            }
        }

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

    eprintln!("[Export Debug] SQL: {}", sql);
    eprintln!("[Export Debug] Params: {:?}", params);

    let mut stmt = conn.prepare(&sql)?;

    let stats: Result<Vec<StreamStats>, _> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StreamStats {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                collected_at: row.get(2)?,
                viewer_count: row.get(3)?,
                chat_rate_1min: Some(row.get(4)?), // Now properly mapped from query
                category: row.get(5)?,
                game_id: None,
                title: row.get(6)?,
                follower_count: row.get(7)?,
                twitch_user_id: row.get(8)?,
                channel_name: row.get(9)?,
            })
        })?
        .collect();

    match &stats {
        Ok(data) => eprintln!("[Export Debug] Query returned {} records", data.len()),
        Err(e) => eprintln!("[Export Debug] Query error: {:?}", e),
    }

    stats
}

/// Helper function to escape field values for delimited output
fn escape_field(value: &str, delimiter: &str) -> String {
    // Check if field needs escaping (contains delimiter, quotes, or newlines)
    if value.contains(delimiter)
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
    {
        // Escape quotes by doubling them, then wrap in quotes
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[tauri::command]
pub async fn export_to_delimited(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ExportQuery,
    file_path: String,
    include_bom: Option<bool>,
) -> Result<String, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let stats = get_stream_stats_internal(&conn, &query)
        .db_context("query stats")
        .map_err(|e| e.to_string())?;

    let stats_len = stats.len();

    // Determine delimiter (default to comma)
    let delimiter = query.delimiter.as_deref().unwrap_or(",");

    // Build delimited file content
    let mut output = String::new();

    // Add UTF-8 BOM if requested (helps Excel recognize UTF-8)
    if include_bom.unwrap_or(false) {
        output.push('\u{FEFF}');
    }

    // Header row with full columns
    output.push_str(&format!(
        "collected_at{}channel_name{}viewer_count{}category{}title{}chat_rate_1min\n",
        delimiter, delimiter, delimiter, delimiter, delimiter
    ));

    // Data rows
    for stat in &stats {
        let collected_at = &stat.collected_at;
        let channel_name = stat.channel_name.as_deref().unwrap_or("");
        let viewer_count = stat.viewer_count.unwrap_or(0).to_string();
        let category = stat.category.as_deref().unwrap_or("");
        let title = stat.title.as_deref().unwrap_or("");
        let chat_rate = stat
            .chat_rate_1min
            .map(|c| c.to_string())
            .unwrap_or_else(|| "0".to_string());

        output.push_str(&format!(
            "{}{}{}{}{}{}{}{}{}{}{}\n",
            escape_field(collected_at, delimiter),
            delimiter,
            escape_field(channel_name, delimiter),
            delimiter,
            viewer_count,
            delimiter,
            escape_field(category, delimiter),
            delimiter,
            escape_field(title, delimiter),
            delimiter,
            chat_rate
        ));
    }

    // Write to file
    std::fs::write(&file_path, output)
        .io_context("write file")
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Exported {} records to {} (delimiter: {:?})",
        stats_len, file_path, delimiter
    ))
}

#[tauri::command]
pub async fn preview_export_data(
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    query: ExportQuery,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let stats = get_stream_stats_internal(&conn, &query)
        .db_context("query stats")
        .map_err(|e| e.to_string())?;

    // Limit to max_rows (default 10)
    let max_rows = max_rows.unwrap_or(10);
    let preview_stats = stats.iter().take(max_rows);

    // Determine delimiter (default to comma)
    let delimiter = query.delimiter.as_deref().unwrap_or(",");

    // Build preview content
    let mut output = String::new();

    // Header row with full columns
    output.push_str(&format!(
        "collected_at{}channel_name{}viewer_count{}category{}title{}chat_rate_1min\n",
        delimiter, delimiter, delimiter, delimiter, delimiter
    ));

    // Data rows (limited to max_rows)
    for stat in preview_stats {
        let collected_at = &stat.collected_at;
        let channel_name = stat.channel_name.as_deref().unwrap_or("");
        let viewer_count = stat.viewer_count.unwrap_or(0).to_string();
        let category = stat.category.as_deref().unwrap_or("");
        let title = stat.title.as_deref().unwrap_or("");
        let chat_rate = stat
            .chat_rate_1min
            .map(|c| c.to_string())
            .unwrap_or_else(|| "0".to_string());

        output.push_str(&format!(
            "{}{}{}{}{}{}{}{}{}{}{}\n",
            escape_field(collected_at, delimiter),
            delimiter,
            escape_field(channel_name, delimiter),
            delimiter,
            viewer_count,
            delimiter,
            escape_field(category, delimiter),
            delimiter,
            escape_field(title, delimiter),
            delimiter,
            chat_rate
        ));
    }

    Ok(output)
}
