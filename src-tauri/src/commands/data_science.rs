use crate::database::{data_science_analytics, DatabaseManager};
use crate::error::ResultExt;
use tauri::State;

// ============================================================================
// Phase 1: Text Analysis Commands
// ============================================================================

#[tauri::command]
pub async fn get_word_frequency_analysis(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    limit: Option<i32>,
) -> Result<data_science_analytics::WordFrequencyResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_word_frequency_analysis(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        limit.unwrap_or(100),
    )
    .db_context("get word frequency analysis")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_emote_analysis(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<data_science_analytics::EmoteAnalysisResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_emote_analysis(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get emote analysis")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_message_length_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<data_science_analytics::MessageLengthStats, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_message_length_stats(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get message length stats")
    .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 2: Correlation Analysis Commands
// ============================================================================

#[tauri::command]
pub async fn get_viewer_chat_correlation(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<data_science_analytics::CorrelationResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_viewer_chat_correlation(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get viewer chat correlation")
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_category_change_impact(
    db_manager: State<'_, DatabaseManager>,
    channel_id: i64,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<data_science_analytics::CategoryImpactResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_category_change_impact(
        &conn,
        channel_id,
        start_time.as_deref(),
        end_time.as_deref(),
    )
    .db_context("get category change impact")
    .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 3: User Behavior Analysis Commands
// ============================================================================

#[tauri::command]
pub async fn get_chatter_activity_scores(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    limit: Option<i32>,
) -> Result<data_science_analytics::ChatterScoreResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::get_chatter_activity_scores(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        limit.unwrap_or(50),
    )
    .db_context("get chatter activity scores")
    .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 4: Anomaly Detection Commands
// ============================================================================

#[tauri::command]
pub async fn detect_anomalies(
    db_manager: State<'_, DatabaseManager>,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<String>,
    end_time: Option<String>,
    z_threshold: Option<f64>,
) -> Result<data_science_analytics::AnomalyResult, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    data_science_analytics::detect_anomalies(
        &conn,
        channel_id,
        stream_id,
        start_time.as_deref(),
        end_time.as_deref(),
        z_threshold.unwrap_or(2.5),
    )
    .db_context("detect anomalies")
    .map_err(|e| e.to_string())
}
