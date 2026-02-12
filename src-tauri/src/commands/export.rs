use crate::database::{repositories::StreamStatsRepository, DatabaseManager};
use crate::error::ResultExt;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub channel_id: i64,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub aggregation: Option<String>, // "raw", "1min", "5min", "1hour"
    pub delimiter: Option<String>,   // Custom delimiter (default: comma)
}

fn normalize_timestamp(value: &str) -> String {
    // 1) RFC3339 (元の文字列形式を想定)
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return dt.to_rfc3339();
    }
    // 2) DuckDB の TIMESTAMP 表示形式を想定（秒以下あり/なし両対応）
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
    {
        if let Some(offset) = FixedOffset::east_opt(0) {
            return DateTime::<FixedOffset>::from_naive_utc_and_offset(naive, offset).to_rfc3339();
        }
    }
    // 3) どれにも当てはまらない場合は元の文字列を返す
    value.to_string()
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
    let ExportQuery {
        channel_id,
        start_time,
        end_time,
        aggregation,
        delimiter,
    } = query;

    let stats = db_manager
        .with_connection(|conn| {
            let start_opt = start_time.as_deref();
            let end_opt = end_time.as_deref();

            let interval_minutes = match aggregation.as_deref() {
                Some("1min") => Some(1),
                Some("5min") => Some(5),
                Some("1hour") => Some(60),
                _ => None,
            };

            if let (Some(st), Some(et), Some(interval)) = (start_opt, end_opt, interval_minutes) {
                StreamStatsRepository::get_interpolated_stream_stats_for_export(
                    conn, None, Some(channel_id), st, et, interval,
                )
                .db_context("query interpolated stats for export")
                .map_err(|e| e.to_string())
            } else {
                StreamStatsRepository::get_stream_stats_filtered(
                    conn, None, Some(channel_id), start_opt, end_opt,
                    true, // ORDER BY collected_at ASC for export
                )
                .db_context("query stats")
                .map_err(|e| e.to_string())
            }
        })
        .await?;

    let stats_len = stats.len();

    // Determine delimiter (default to comma)
    let delimiter = delimiter.as_deref().unwrap_or(",");

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
        let collected_at = normalize_timestamp(&stat.collected_at);
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
            escape_field(&collected_at, delimiter),
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
    let ExportQuery {
        channel_id,
        start_time,
        end_time,
        aggregation,
        delimiter,
    } = query;

    let stats = db_manager
        .with_connection(|conn| {
            let start_opt = start_time.as_deref();
            let end_opt = end_time.as_deref();

            let interval_minutes = match aggregation.as_deref() {
                Some("1min") => Some(1),
                Some("5min") => Some(5),
                Some("1hour") => Some(60),
                _ => None,
            };

            if let (Some(st), Some(et), Some(interval)) = (start_opt, end_opt, interval_minutes) {
                StreamStatsRepository::get_interpolated_stream_stats_for_export(
                    conn, None, Some(channel_id), st, et, interval,
                )
                .db_context("query interpolated stats for preview")
                .map_err(|e| e.to_string())
            } else {
                StreamStatsRepository::get_stream_stats_filtered(
                    conn, None, Some(channel_id), start_opt, end_opt,
                    true, // ORDER BY collected_at ASC
                )
                .db_context("query stats")
                .map_err(|e| e.to_string())
            }
        })
        .await?;

    // Limit to max_rows (default 10)
    let max_rows = max_rows.unwrap_or(10);
    let preview_stats = stats.iter().take(max_rows);

    // Determine delimiter (default to comma)
    let delimiter = delimiter.as_deref().unwrap_or(",");

    // Build preview content
    let mut output = String::new();

    // Header row with full columns
    output.push_str(&format!(
        "collected_at{}channel_name{}viewer_count{}category{}title{}chat_rate_1min\n",
        delimiter, delimiter, delimiter, delimiter, delimiter
    ));

    // Data rows (limited to max_rows)
    for stat in preview_stats {
        let collected_at = normalize_timestamp(&stat.collected_at);
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
            escape_field(&collected_at, delimiter),
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
