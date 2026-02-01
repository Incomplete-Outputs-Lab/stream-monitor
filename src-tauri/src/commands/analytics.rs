use crate::database::{analytics, DatabaseManager};
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    analytics::get_broadcaster_analytics(
        &conn,
        channel_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .map_err(|e| format!("Failed to get broadcaster analytics: {}", e))
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    analytics::get_game_analytics(
        &conn,
        category.as_deref(),
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .map_err(|e| format!("Failed to get game analytics: {}", e))
}

#[tauri::command]
pub async fn list_game_categories(
    db_manager: State<'_, DatabaseManager>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<String>, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    analytics::list_categories(&conn, start_time.as_deref(), end_time.as_deref())
        .map_err(|e| format!("Failed to list categories: {}", e))
}
