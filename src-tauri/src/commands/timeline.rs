use crate::database::DatabaseManager;
use chrono::Local;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: i64,
    pub stream_id: String,
    pub channel_id: i64,
    pub channel_name: String,
    pub title: String,
    pub category: String,
    pub started_at: String,
    pub ended_at: String,
    pub peak_viewers: i32,
    pub avg_viewers: i32,
    pub duration_minutes: i32,
    pub minutes_watched: i64,
    pub follower_gain: i32,
    pub total_chat_messages: i64,
    pub engagement_rate: f64,
    pub last_collected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub collected_at: String,
    pub viewer_count: i32,
    pub chat_rate_1min: i32,
    pub category: String,
    pub title: String,
    pub follower_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryChange {
    pub timestamp: String,
    pub from_category: String,
    pub to_category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleChange {
    pub timestamp: String,
    pub from_title: String,
    pub to_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamTimelineData {
    pub stream_info: StreamInfo,
    pub stats: Vec<TimelinePoint>,
    pub category_changes: Vec<CategoryChange>,
    pub title_changes: Vec<TitleChange>,
}

/// チャンネルの配信一覧を取得
#[tauri::command]
pub async fn get_channel_streams(
    channel_id: i64,
    limit: Option<i32>,
    offset: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_channel_streams_internal(&conn, channel_id, limit, offset)
        .map_err(|e| format!("Failed to get channel streams: {}", e))
}

/// 日付範囲で配信一覧を取得（全チャンネル・カレンダー用）
#[tauri::command]
pub async fn get_streams_by_date_range(
    date_from: String,
    date_to: String,
    limit: Option<i32>,
    offset: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_streams_by_date_range_internal(&conn, &date_from, &date_to, limit, offset)
        .map_err(|e| format!("Failed to get streams by date range: {}", e))
}

/// 比較用：基準配信と時間帯が重なる配信をサジェスト（全チャンネル・カテゴリ・時間帯）
#[tauri::command]
pub async fn get_suggested_streams_for_comparison(
    base_stream_id: i64,
    limit: Option<i32>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<StreamInfo>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_suggested_streams_for_comparison_internal(&conn, base_stream_id, limit)
        .map_err(|e| format!("Failed to get suggested streams: {}", e))
}

fn get_channel_streams_internal(
    conn: &Connection,
    channel_id: i64,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<StreamInfo>, duckdb::Error> {
    let limit_clause = limit.unwrap_or(50);
    let offset_clause = offset.unwrap_or(0);

    let query = format!(
        r#"
        WITH stream_metrics AS (
            SELECT 
                s.id,
                s.stream_id,
                s.channel_id,
                s.title,
                s.category,
                s.started_at,
                s.ended_at,
                COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
                COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
                COALESCE(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 60,
                    0
                ) as duration_minutes,
                MAX(ss.collected_at) as last_collected_at
            FROM streams s
            LEFT JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE s.channel_id = ?
            GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT 
                ss.stream_id,
                ss.viewer_count,
                ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT 
                stream_id,
                COALESCE(SUM(
                    COALESCE(viewer_count, 0) * 
                    EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60
                ), 0)::BIGINT as minutes_watched
            FROM stats_with_next
            WHERE next_collected_at IS NOT NULL
            GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT 
                ss.stream_id,
                COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
              AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT 
                s.id,
                COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s
            LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE s.channel_id = ?
            GROUP BY s.id
        )
        SELECT 
            sm.id,
            sm.stream_id,
            sm.channel_id,
            c.channel_name,
            COALESCE(sm.title, '') as title,
            COALESCE(sm.category, '') as category,
            CAST(sm.started_at AS VARCHAR) as started_at,
            CAST(sm.ended_at AS VARCHAR) as ended_at,
            sm.peak_viewers,
            sm.avg_viewers,
            sm.duration_minutes,
            COALESCE(mw.minutes_watched, 0) as minutes_watched,
            COALESCE(fc.follower_gain, 0) as follower_gain,
            COALESCE(cc.total_chat_messages, 0) as total_chat_messages,
            CASE 
                WHEN COALESCE(mw.minutes_watched, 0) > 0 
                THEN (COALESCE(cc.total_chat_messages, 0)::DOUBLE / mw.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate,
            CAST(sm.last_collected_at AS VARCHAR) as last_collected_at
        FROM stream_metrics sm
        JOIN channels c ON sm.channel_id = c.id
        LEFT JOIN mw_calc mw ON sm.id = mw.stream_id
        LEFT JOIN follower_calc fc ON sm.id = fc.stream_id
        LEFT JOIN chat_calc cc ON sm.id = cc.id
        ORDER BY sm.started_at DESC
        LIMIT {}
        OFFSET {}
        "#,
        limit_clause, offset_clause
    );

    let mut stmt = conn.prepare(&query)?;
    let channel_id_str = channel_id.to_string();

    let streams = stmt.query_map([&channel_id_str, &channel_id_str], |row| {
        Ok(StreamInfo {
            id: row.get::<_, i64>(0)?,
            stream_id: row.get::<_, String>(1)?,
            channel_id: row.get::<_, i64>(2)?,
            channel_name: row.get::<_, String>(3)?,
            title: row.get::<_, String>(4)?,
            category: row.get::<_, String>(5)?,
            started_at: row.get::<_, String>(6)?,
            ended_at: row.get::<_, String>(7).unwrap_or_default(),
            peak_viewers: row.get::<_, i32>(8)?,
            avg_viewers: row.get::<_, i32>(9)?,
            duration_minutes: row.get::<_, i32>(10)?,
            minutes_watched: row.get::<_, i64>(11)?,
            follower_gain: row.get::<_, i32>(12)?,
            total_chat_messages: row.get::<_, i64>(13)?,
            engagement_rate: row.get::<_, f64>(14)?,
            last_collected_at: row.get::<_, String>(15).unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for stream in streams {
        result.push(stream?);
    }

    Ok(result)
}

/// 日付範囲で配信一覧を取得（全チャンネル）
fn get_streams_by_date_range_internal(
    conn: &Connection,
    date_from: &str,
    date_to: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<StreamInfo>, duckdb::Error> {
    let limit_clause = limit.unwrap_or(100);
    let offset_clause = offset.unwrap_or(0);

    let query = format!(
        r#"
        WITH stream_metrics AS (
            SELECT 
                s.id,
                s.stream_id,
                s.channel_id,
                s.title,
                s.category,
                s.started_at,
                s.ended_at,
                COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
                COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
                COALESCE(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 60,
                    0
                ) as duration_minutes,
                MAX(ss.collected_at) as last_collected_at
            FROM streams s
            LEFT JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE CAST(s.started_at AS DATE) >= CAST(? AS DATE)
              AND CAST(s.started_at AS DATE) <= CAST(? AS DATE)
            GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT 
                ss.stream_id,
                ss.viewer_count,
                ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT 
                stream_id,
                COALESCE(SUM(
                    COALESCE(viewer_count, 0) * 
                    EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60
                ), 0)::BIGINT as minutes_watched
            FROM stats_with_next
            WHERE next_collected_at IS NOT NULL
            GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT 
                ss.stream_id,
                COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
              AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT 
                s.id,
                COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s
            LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = s.id)
            GROUP BY s.id
        )
        SELECT 
            sm.id,
            sm.stream_id,
            sm.channel_id,
            c.channel_name,
            COALESCE(sm.title, '') as title,
            COALESCE(sm.category, '') as category,
            CAST(sm.started_at AS VARCHAR) as started_at,
            CAST(sm.ended_at AS VARCHAR) as ended_at,
            sm.peak_viewers,
            sm.avg_viewers,
            sm.duration_minutes,
            COALESCE(mw.minutes_watched, 0) as minutes_watched,
            COALESCE(fc.follower_gain, 0) as follower_gain,
            COALESCE(cc.total_chat_messages, 0) as total_chat_messages,
            CASE 
                WHEN COALESCE(mw.minutes_watched, 0) > 0 
                THEN (COALESCE(cc.total_chat_messages, 0)::DOUBLE / mw.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate,
            CAST(sm.last_collected_at AS VARCHAR) as last_collected_at
        FROM stream_metrics sm
        JOIN channels c ON sm.channel_id = c.id
        LEFT JOIN mw_calc mw ON sm.id = mw.stream_id
        LEFT JOIN follower_calc fc ON sm.id = fc.stream_id
        LEFT JOIN chat_calc cc ON sm.id = cc.id
        ORDER BY sm.started_at DESC
        LIMIT {}
        OFFSET {}
        "#,
        limit_clause, offset_clause
    );

    let mut stmt = conn.prepare(&query)?;
    let streams = stmt.query_map([date_from, date_to], |row| {
        Ok(StreamInfo {
            id: row.get::<_, i64>(0)?,
            stream_id: row.get::<_, String>(1)?,
            channel_id: row.get::<_, i64>(2)?,
            channel_name: row.get::<_, String>(3)?,
            title: row.get::<_, String>(4)?,
            category: row.get::<_, String>(5)?,
            started_at: row.get::<_, String>(6)?,
            ended_at: row.get::<_, String>(7).unwrap_or_default(),
            peak_viewers: row.get::<_, i32>(8)?,
            avg_viewers: row.get::<_, i32>(9)?,
            duration_minutes: row.get::<_, i32>(10)?,
            minutes_watched: row.get::<_, i64>(11)?,
            follower_gain: row.get::<_, i32>(12)?,
            total_chat_messages: row.get::<_, i64>(13)?,
            engagement_rate: row.get::<_, f64>(14)?,
            last_collected_at: row.get::<_, String>(15).unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for stream in streams {
        result.push(stream?);
    }
    Ok(result)
}

/// 比較用：基準配信と時間帯が重なる配信を取得（全チャンネル）
fn get_suggested_streams_for_comparison_internal(
    conn: &Connection,
    base_stream_id: i64,
    limit: Option<i32>,
) -> Result<Vec<StreamInfo>, String> {
    let base = get_stream_info_by_id(conn, base_stream_id).map_err(|e| e.to_string())?;
    let limit_clause = limit.unwrap_or(50);

    // 基準配信の開始・終了（終了未設定＝配信中は現在時刻を使用）
    let base_start = base.started_at.clone();
    let base_end = if base.ended_at.is_empty() {
        Local::now().to_rfc3339()
    } else {
        base.ended_at.clone()
    };

    // 時間重複: 他配信 started_at < base_end AND (ended_at or now) > base_start
    let query = format!(
        r#"
        WITH base_stream AS (
            SELECT id, channel_id, started_at, ended_at, category FROM streams WHERE id = ?
        ),
        stream_metrics AS (
            SELECT 
                s.id,
                s.stream_id,
                s.channel_id,
                s.title,
                s.category,
                s.started_at,
                s.ended_at,
                COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
                COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
                COALESCE(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 60,
                    0
                ) as duration_minutes,
                MAX(ss.collected_at) as last_collected_at
            FROM streams s
            LEFT JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE s.id != ?
              AND s.started_at < CAST(? AS TIMESTAMP)
              AND COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) > CAST(? AS TIMESTAMP)
            GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT 
                ss.stream_id,
                ss.viewer_count,
                ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
        ),
        mw_calc AS (
            SELECT 
                stream_id,
                COALESCE(SUM(
                    COALESCE(viewer_count, 0) * 
                    EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60
                ), 0)::BIGINT as minutes_watched
            FROM stats_with_next
            WHERE next_collected_at IS NOT NULL
            GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT 
                ss.stream_id,
                COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = ss.stream_id)
              AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT 
                s.id,
                COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s
            LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE EXISTS (SELECT 1 FROM stream_metrics sm WHERE sm.id = s.id)
            GROUP BY s.id
        )
        SELECT 
            sm.id,
            sm.stream_id,
            sm.channel_id,
            c.channel_name,
            COALESCE(sm.title, '') as title,
            COALESCE(sm.category, '') as category,
            CAST(sm.started_at AS VARCHAR) as started_at,
            CAST(sm.ended_at AS VARCHAR) as ended_at,
            sm.peak_viewers,
            sm.avg_viewers,
            sm.duration_minutes,
            COALESCE(mw.minutes_watched, 0) as minutes_watched,
            COALESCE(fc.follower_gain, 0) as follower_gain,
            COALESCE(cc.total_chat_messages, 0) as total_chat_messages,
            CASE 
                WHEN COALESCE(mw.minutes_watched, 0) > 0 
                THEN (COALESCE(cc.total_chat_messages, 0)::DOUBLE / mw.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate,
            CAST(sm.last_collected_at AS VARCHAR) as last_collected_at
        FROM stream_metrics sm
        JOIN channels c ON sm.channel_id = c.id
        LEFT JOIN mw_calc mw ON sm.id = mw.stream_id
        LEFT JOIN follower_calc fc ON sm.id = fc.stream_id
        LEFT JOIN chat_calc cc ON sm.id = cc.id
        ORDER BY 
            CASE WHEN sm.category = (SELECT category FROM base_stream) THEN 0 ELSE 1 END,
            sm.started_at ASC
        LIMIT {}
        "#,
        limit_clause
    );

    let base_id_str = base_stream_id.to_string();
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let streams = stmt
        .query_map(
            [&base_id_str, &base_id_str, &base_end, &base_start],
            |row| {
                Ok(StreamInfo {
                    id: row.get::<_, i64>(0)?,
                    stream_id: row.get::<_, String>(1)?,
                    channel_id: row.get::<_, i64>(2)?,
                    channel_name: row.get::<_, String>(3)?,
                    title: row.get::<_, String>(4)?,
                    category: row.get::<_, String>(5)?,
                    started_at: row.get::<_, String>(6)?,
                    ended_at: row.get::<_, String>(7).unwrap_or_default(),
                    peak_viewers: row.get::<_, i32>(8)?,
                    avg_viewers: row.get::<_, i32>(9)?,
                    duration_minutes: row.get::<_, i32>(10)?,
                    minutes_watched: row.get::<_, i64>(11)?,
                    follower_gain: row.get::<_, i32>(12)?,
                    total_chat_messages: row.get::<_, i64>(13)?,
                    engagement_rate: row.get::<_, f64>(14)?,
                    last_collected_at: row.get::<_, String>(15).unwrap_or_default(),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for stream in streams {
        result.push(stream.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

/// 特定配信のタイムラインデータを取得
#[tauri::command]
pub async fn get_stream_timeline(
    stream_id: i64,
    db_manager: State<'_, DatabaseManager>,
) -> Result<StreamTimelineData, String> {
    let conn = db_manager
        .get_connection()
        .await
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_stream_timeline_internal(&conn, stream_id)
        .map_err(|e| format!("Failed to get stream timeline: {}", e))
}

fn get_stream_timeline_internal(
    conn: &Connection,
    stream_id: i64,
) -> Result<StreamTimelineData, Box<dyn std::error::Error + Send + Sync>> {
    // ストリーム基本情報を取得
    let stream_info = get_stream_info_by_id(conn, stream_id)?;

    // タイムライン統計データを取得
    let stats = get_timeline_stats(conn, stream_id)?;

    // カテゴリ変更を検出
    let category_changes = detect_category_changes(&stats);

    // タイトル変更を検出
    let title_changes = detect_title_changes(&stats);

    Ok(StreamTimelineData {
        stream_info,
        stats,
        category_changes,
        title_changes,
    })
}

fn get_stream_info_by_id(
    conn: &Connection,
    stream_id: i64,
) -> Result<StreamInfo, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        WITH stream_metrics AS (
            SELECT 
                s.id,
                s.stream_id,
                s.channel_id,
                s.title,
                s.category,
                s.started_at,
                s.ended_at,
                COALESCE(MAX(ss.viewer_count), 0) as peak_viewers,
                COALESCE(AVG(ss.viewer_count), 0) as avg_viewers,
                COALESCE(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 60,
                    0
                ) as duration_minutes,
                MAX(ss.collected_at) as last_collected_at
            FROM streams s
            LEFT JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE s.id = ?
            GROUP BY s.id, s.stream_id, s.channel_id, s.title, s.category, s.started_at, s.ended_at
        ),
        stats_with_next AS (
            SELECT 
                ss.stream_id,
                ss.viewer_count,
                ss.collected_at,
                LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) as next_collected_at
            FROM stream_stats ss
            WHERE ss.stream_id = ?
        ),
        mw_calc AS (
            SELECT 
                stream_id,
                COALESCE(SUM(
                    COALESCE(viewer_count, 0) * 
                    EXTRACT(EPOCH FROM (next_collected_at - collected_at)) / 60
                ), 0)::BIGINT as minutes_watched
            FROM stats_with_next
            WHERE next_collected_at IS NOT NULL
            GROUP BY stream_id
        ),
        follower_calc AS (
            SELECT 
                ss.stream_id,
                COALESCE(MAX(ss.follower_count) - MIN(ss.follower_count), 0) as follower_gain
            FROM stream_stats ss
            WHERE ss.stream_id = ?
              AND ss.follower_count IS NOT NULL
            GROUP BY ss.stream_id
        ),
        chat_calc AS (
            SELECT 
                s.id,
                COALESCE(COUNT(cm.id), 0)::BIGINT as total_chat_messages
            FROM streams s
            LEFT JOIN chat_messages cm ON s.id = cm.stream_id
            WHERE s.id = ?
            GROUP BY s.id
        )
        SELECT 
            sm.id,
            sm.stream_id,
            sm.channel_id,
            c.channel_name,
            COALESCE(sm.title, '') as title,
            COALESCE(sm.category, '') as category,
            CAST(sm.started_at AS VARCHAR) as started_at,
            CAST(sm.ended_at AS VARCHAR) as ended_at,
            sm.peak_viewers,
            sm.avg_viewers,
            sm.duration_minutes,
            COALESCE(mw.minutes_watched, 0) as minutes_watched,
            COALESCE(fc.follower_gain, 0) as follower_gain,
            COALESCE(cc.total_chat_messages, 0) as total_chat_messages,
            CASE 
                WHEN COALESCE(mw.minutes_watched, 0) > 0 
                THEN (COALESCE(cc.total_chat_messages, 0)::DOUBLE / mw.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate,
            CAST(sm.last_collected_at AS VARCHAR) as last_collected_at
        FROM stream_metrics sm
        JOIN channels c ON sm.channel_id = c.id
        LEFT JOIN mw_calc mw ON sm.id = mw.stream_id
        LEFT JOIN follower_calc fc ON sm.id = fc.stream_id
        LEFT JOIN chat_calc cc ON sm.id = cc.id
    "#;

    let stream_id_str = stream_id.to_string();
    let result = conn.query_row(
        query,
        [
            &stream_id_str,
            &stream_id_str,
            &stream_id_str,
            &stream_id_str,
        ],
        |row| {
            Ok(StreamInfo {
                id: row.get::<_, i64>(0)?,
                stream_id: row.get::<_, String>(1)?,
                channel_id: row.get::<_, i64>(2)?,
                channel_name: row.get::<_, String>(3)?,
                title: row.get::<_, String>(4)?,
                category: row.get::<_, String>(5)?,
                started_at: row.get::<_, String>(6)?,
                ended_at: row.get::<_, String>(7).unwrap_or_default(),
                peak_viewers: row.get::<_, i32>(8)?,
                avg_viewers: row.get::<_, i32>(9)?,
                duration_minutes: row.get::<_, i32>(10)?,
                minutes_watched: row.get::<_, i64>(11)?,
                follower_gain: row.get::<_, i32>(12)?,
                total_chat_messages: row.get::<_, i64>(13)?,
                engagement_rate: row.get::<_, f64>(14)?,
                last_collected_at: row.get::<_, String>(15).unwrap_or_default(),
            })
        },
    )?;

    Ok(result)
}

fn get_timeline_stats(
    conn: &Connection,
    stream_id: i64,
) -> Result<Vec<TimelinePoint>, Box<dyn std::error::Error + Send + Sync>> {
    let query = r#"
        SELECT 
            CAST(ss.collected_at AS VARCHAR) as collected_at,
            ss.viewer_count,
            COALESCE((
                SELECT COUNT(*)
                FROM chat_messages cm
                WHERE cm.stream_id = ss.stream_id
                  AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                  AND cm.timestamp < ss.collected_at
            ), 0) AS chat_rate_1min,
            ss.category,
            ss.title,
            ss.follower_count
        FROM stream_stats ss
        WHERE ss.stream_id = ?
        ORDER BY ss.collected_at ASC
    "#;

    let stream_id_str = stream_id.to_string();
    let mut stmt = conn.prepare(query)?;

    let stats = stmt.query_map([&stream_id_str], |row| {
        Ok(TimelinePoint {
            collected_at: row.get::<_, String>(0)?,
            viewer_count: row.get::<_, i32>(1).unwrap_or_default(),
            chat_rate_1min: row.get::<_, i32>(2)?,
            category: row.get::<_, String>(3).unwrap_or_default(),
            title: row.get::<_, String>(4).unwrap_or_default(),
            follower_count: row.get::<_, i32>(5).unwrap_or_default(),
        })
    })?;

    let mut result = Vec::new();
    for stat in stats {
        result.push(stat?);
    }

    Ok(result)
}

fn detect_category_changes(stats: &[TimelinePoint]) -> Vec<CategoryChange> {
    let mut changes = Vec::new();
    let mut prev_category: Option<String> = None;

    for stat in stats {
        if !stat.category.is_empty() {
            // Only record changes where prev_category is Some and differs from current
            if let Some(ref prev) = prev_category {
                if prev != &stat.category {
                    changes.push(CategoryChange {
                        timestamp: stat.collected_at.clone(),
                        from_category: prev.clone(),
                        to_category: stat.category.clone(),
                    });
                }
            }
            prev_category = Some(stat.category.clone());
        }
    }

    changes
}

fn detect_title_changes(stats: &[TimelinePoint]) -> Vec<TitleChange> {
    let mut changes = Vec::new();
    let mut prev_title: Option<String> = None;

    for stat in stats {
        if !stat.title.is_empty() {
            // Only record changes where prev_title is Some and differs from current
            if let Some(ref prev) = prev_title {
                if prev != &stat.title {
                    changes.push(TitleChange {
                        timestamp: stat.collected_at.clone(),
                        from_title: prev.clone(),
                        to_title: stat.title.clone(),
                    });
                }
            }
            prev_title = Some(stat.title.clone());
        }
    }

    changes
}
