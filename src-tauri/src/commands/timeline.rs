use crate::database::DatabaseManager;
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
    pub ended_at: Option<String>,
    pub peak_viewers: i32,
    pub avg_viewers: i32,
    pub duration_minutes: i32,
    pub minutes_watched: i64,
    pub follower_gain: i32,
    pub total_chat_messages: i64,
    pub engagement_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub collected_at: String,
    pub viewer_count: Option<i32>,
    pub chat_rate_1min: i32,
    pub category: Option<String>,
    pub title: Option<String>,
    pub follower_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryChange {
    pub timestamp: String,
    pub from_category: Option<String>,
    pub to_category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleChange {
    pub timestamp: String,
    pub from_title: Option<String>,
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
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_channel_streams_internal(&conn, channel_id, limit, offset)
        .map_err(|e| format!("Failed to get channel streams: {}", e))
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
                ) as duration_minutes
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
            END as engagement_rate
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
            ended_at: row.get::<_, Option<String>>(7)?,
            peak_viewers: row.get::<_, i32>(8)?,
            avg_viewers: row.get::<_, i32>(9)?,
            duration_minutes: row.get::<_, i32>(10)?,
            minutes_watched: row.get::<_, i64>(11)?,
            follower_gain: row.get::<_, i32>(12)?,
            total_chat_messages: row.get::<_, i64>(13)?,
            engagement_rate: row.get::<_, f64>(14)?,
        })
    })?;

    let mut result = Vec::new();
    for stream in streams {
        result.push(stream?);
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
        .map_err(|e| format!("Database connection error: {}", e))?;

    get_stream_timeline_internal(&conn, stream_id)
        .map_err(|e| format!("Failed to get stream timeline: {}", e))
}

fn get_stream_timeline_internal(
    conn: &Connection,
    stream_id: i64,
) -> Result<StreamTimelineData, Box<dyn std::error::Error>> {
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
) -> Result<StreamInfo, Box<dyn std::error::Error>> {
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
                ) as duration_minutes
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
            END as engagement_rate
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
                ended_at: row.get::<_, Option<String>>(7)?,
                peak_viewers: row.get::<_, i32>(8)?,
                avg_viewers: row.get::<_, i32>(9)?,
                duration_minutes: row.get::<_, i32>(10)?,
                minutes_watched: row.get::<_, i64>(11)?,
                follower_gain: row.get::<_, i32>(12)?,
                total_chat_messages: row.get::<_, i64>(13)?,
                engagement_rate: row.get::<_, f64>(14)?,
            })
        },
    )?;

    Ok(result)
}

fn get_timeline_stats(
    conn: &Connection,
    stream_id: i64,
) -> Result<Vec<TimelinePoint>, Box<dyn std::error::Error>> {
    let query = r#"
        SELECT 
            CAST(collected_at AS VARCHAR) as collected_at,
            viewer_count,
            chat_rate_1min,
            category,
            title,
            follower_count
        FROM stream_stats
        WHERE stream_id = ?
        ORDER BY collected_at ASC
    "#;

    let stream_id_str = stream_id.to_string();
    let mut stmt = conn.prepare(query)?;

    let stats = stmt.query_map([&stream_id_str], |row| {
        Ok(TimelinePoint {
            collected_at: row.get::<_, String>(0)?,
            viewer_count: row.get::<_, Option<i32>>(1)?,
            chat_rate_1min: row.get::<_, i32>(2)?,
            category: row.get::<_, Option<String>>(3)?,
            title: row.get::<_, Option<String>>(4)?,
            follower_count: row.get::<_, Option<i32>>(5)?,
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
        if let Some(ref current_category) = stat.category {
            if prev_category.as_ref() != Some(current_category) {
                changes.push(CategoryChange {
                    timestamp: stat.collected_at.clone(),
                    from_category: prev_category.clone(),
                    to_category: current_category.clone(),
                });
                prev_category = Some(current_category.clone());
            }
        }
    }

    changes
}

fn detect_title_changes(stats: &[TimelinePoint]) -> Vec<TitleChange> {
    let mut changes = Vec::new();
    let mut prev_title: Option<String> = None;

    for stat in stats {
        if let Some(ref current_title) = stat.title {
            if prev_title.as_ref() != Some(current_title) {
                changes.push(TitleChange {
                    timestamp: stat.collected_at.clone(),
                    from_title: prev_title.clone(),
                    to_title: current_title.clone(),
                });
                prev_title = Some(current_title.clone());
            }
        }
    }

    changes
}
