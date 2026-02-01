use crate::error::ResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::{command, AppHandle, Manager};

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct GetLogsQuery {
    pub level: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

#[command]
pub async fn get_logs(app_handle: AppHandle, query: GetLogsQuery) -> Result<Vec<LogEntry>, String> {
    // ログファイルのパスを取得（データベースと同じディレクトリにlogs.txtとして保存）
    let log_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        app_data_dir.join("logs.txt")
    } else {
        // フォールバック：現在のディレクトリを使用
        std::env::current_dir()
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
            .join("logs.txt")
    };

    // ログファイルが存在しない場合は空の配列を返す
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    // ログファイルを読み込み
    let content = fs::read_to_string(&log_path)
        .io_context("read log file")
        .map_err(|e| format!("ログファイルの読み込みに失敗しました: {}", e))?;

    // ログをパースしてフィルタリング
    let mut logs: Vec<LogEntry> = content.lines().filter_map(parse_log_line).collect();

    // レベルフィルタリング
    if let Some(ref level) = query.level {
        if !level.is_empty() {
            logs.retain(|log| log.level.to_uppercase() == level.to_uppercase());
        }
    }

    // 検索フィルタリング
    if let Some(ref search) = query.search {
        if !search.is_empty() {
            let search_lower = search.to_lowercase();
            logs.retain(|log| log.message.to_lowercase().contains(&search_lower));
        }
    }

    // 制限を適用（デフォルト500、最も新しいログから）
    let limit = query.limit.unwrap_or(500);
    if logs.len() > limit {
        logs = logs.into_iter().rev().take(limit).rev().collect();
    }

    Ok(logs)
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // ログフォーマット: [YYYY-MM-DD HH:MM:SS] LEVEL: message
    // 例: [2024-01-21 10:30:15] INFO: Database connection established
    if line.trim().is_empty() {
        return None;
    }

    // タイムスタンプ部分を解析
    if !line.starts_with('[') {
        return None;
    }

    let timestamp_end = line.find(']')?;
    let timestamp = &line[1..timestamp_end];

    let remaining = &line[timestamp_end + 1..].trim();

    // レベル部分を解析
    let level_end = remaining.find(':')?;
    let level = remaining[..level_end].trim();

    let message = remaining[level_end + 1..].trim();

    Some(LogEntry {
        timestamp: timestamp.to_string(),
        level: level.to_string(),
        message: message.to_string(),
    })
}
