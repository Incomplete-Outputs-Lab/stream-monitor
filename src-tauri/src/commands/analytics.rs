use crate::database::{analytics, chat_analytics, DatabaseManager};
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
        .await
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
    game_id: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<analytics::GameAnalytics>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_game_analytics(
        &conn,
        game_id.as_deref(),
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
        .await
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
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_data_availability(&conn)
        .db_context("get data availability")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game_daily_stats(
    db_manager: State<'_, DatabaseManager>,
    game_id: String,
    start_time: String,
    end_time: String,
) -> Result<Vec<analytics::DailyStats>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")?;

    analytics::get_game_daily_stats(&conn, &game_id, &start_time, &end_time)
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
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    analytics::get_channel_daily_stats(&conn, channel_id, &start_time, &end_time)
        .db_context("get channel daily stats")
        .map_err(|e| e.to_string())
}

// Chat Analytics Commands

#[tauri::command]
pub async fn get_chat_engagement_timeline(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    interval_minutes: Option<i32>,
) -> Result<Vec<chat_analytics::ChatEngagementStats>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::get_chat_engagement_timeline(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        interval_minutes.unwrap_or(5),
    )
    .db_context("get chat engagement timeline")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_chat_spikes(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    min_spike_ratio: Option<f64>,
) -> Result<Vec<chat_analytics::ChatSpike>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::detect_chat_spikes(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        min_spike_ratio.unwrap_or(2.0),
    )
    .db_context("detect chat spikes")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_segment_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<chat_analytics::UserSegmentStats>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::get_user_segment_stats(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get user segment stats")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_top_chatters(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<chat_analytics::TopChatter>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::get_top_chatters(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        limit.unwrap_or(50),
    )
    .db_context("get top chatters")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_time_pattern_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    group_by_day: Option<bool>,
) -> Result<Vec<chat_analytics::TimePatternStats>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::get_time_pattern_stats(
        &conn,
        channel_id,
        start_time.as_deref(),
        end_time.as_deref(),
        group_by_day.unwrap_or(false),
    )
    .db_context("get time pattern stats")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_chatter_behavior_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<chat_analytics::ChatterBehaviorStats, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    chat_analytics::get_chatter_behavior_stats(
        &conn,
        channel_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get chatter behavior stats")
    .map_err(|e| e.to_string())
}
