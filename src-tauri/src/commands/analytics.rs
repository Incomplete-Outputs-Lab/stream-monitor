use crate::database::{analytics, DatabaseManager};
use crate::error::ResultExt;
use tauri::State;

#[tauri::command]
pub async fn get_broadcaster_analytics(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<analytics::BroadcasterAnalytics>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_broadcaster_analytics(
        &conn,
        channel_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get broadcaster analytics")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_analytics(
    db_manager: State<'_, DatabaseManager>,
    category: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<analytics::GameAnalytics>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_game_analytics(
        &conn,
        category.as_deref(),
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get game analytics")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_game_categories(
    db_manager: State<'_, DatabaseManager>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<String>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")?;

    analytics::list_categories(&conn, start_time.as_deref(), end_time.as_deref())
        .db_context("list categories")
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_data_availability(
    db_manager: State<'_, DatabaseManager>,
) -> Result<analytics::DataAvailability, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_data_availability(&conn)
        .db_context("get data availability")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_daily_stats(
    db_manager: State<'_, DatabaseManager>,
    category: String,
    start_time: String,
    end_time: String,
) -> Result<Vec<analytics::DailyStats>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")?;

    analytics::get_game_daily_stats(&conn, &category, &start_time, &end_time)
        .db_context("get game daily stats")
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_channel_daily_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_id: i64,
    start_time: String,
    end_time: String,
) -> Result<Vec<analytics::DailyStats>, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_channel_daily_stats(&conn, channel_id, &start_time, &end_time)
        .db_context("get channel daily stats")
        .map_err(|e| e.to_string())
}
