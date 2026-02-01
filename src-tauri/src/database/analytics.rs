use crate::database::utils;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// 配信者別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcasterAnalytics {
    pub channel_id: i64,
    pub channel_name: String,
    pub minutes_watched: i64,
    pub hours_broadcasted: f64,
    pub average_ccu: f64,
    pub main_played_title: Option<String>,
    pub main_title_mw_percent: Option<f64>,
}

/// ゲームタイトル別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalytics {
    pub category: String,
    pub minutes_watched: i64,
    pub hours_broadcasted: f64,
    pub average_ccu: f64,
    pub unique_broadcasters: i32,
    pub top_channel: Option<String>,
}

/// カテゴリ別 MinutesWatched 中間データ
#[derive(Debug)]
struct CategoryMinutesWatched {
    category: String,
    minutes_watched: i64,
}

/// 配信者別統計を取得
pub fn get_broadcaster_analytics(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<BroadcasterAnalytics>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                s.channel_id,
                c.channel_name,
                ss.viewer_count,
                ss.category,
                ss.collected_at,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            JOIN streams s ON ss.stream_id = s.id
            JOIN channels c ON s.channel_id = c.id
            WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND s.channel_id = ?");
        params.push(ch_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
        ),
        channel_stats AS (
            SELECT 
                channel_id,
                channel_name,
                COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
                COALESCE(AVG(viewer_count), 0) AS average_ccu
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL
            GROUP BY channel_id, channel_name
        )
        SELECT 
            channel_id,
            channel_name,
            minutes_watched,
            average_ccu
        FROM channel_stats
        ORDER BY minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let channel_stats: Vec<_> = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok((
            row.get::<_, i64>(0)?,     // channel_id
            row.get::<_, String>(1)?,  // channel_name
            row.get::<_, i64>(2)?,     // minutes_watched
            row.get::<_, f64>(3)?,     // average_ccu
        ))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    // HoursBroadcasted を個別に計算
    let hours_sql = format!(
        r#"
        SELECT 
            s.channel_id,
            COALESCE(SUM(
                EXTRACT(EPOCH FROM (COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at)) / 3600.0
            ), 0) AS hours_broadcasted
        FROM streams s
        WHERE 1=1 {}
        GROUP BY s.channel_id
        "#,
        if channel_id.is_some() { " AND s.channel_id = ?" } else { "" }
    );

    let mut hours_params: Vec<String> = Vec::new();
    if let Some(ch_id) = channel_id {
        hours_params.push(ch_id.to_string());
    }

    let mut hours_stmt = conn.prepare(&hours_sql)?;
    let hours_map: std::collections::HashMap<i64, f64> =
        utils::query_map_with_params(&mut hours_stmt, &hours_params, |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        })?
        .collect::<Result<_, _>>()?;

    // カテゴリ別 MinutesWatched を取得してメインタイトルを計算
    let mut results = Vec::new();
    for (ch_id, ch_name, mw, avg_ccu) in channel_stats {
        let hours = hours_map.get(&ch_id).copied().unwrap_or(0.0);

        // カテゴリ別 MinutesWatched を取得
        let category_mw = get_category_minutes_watched(conn, ch_id, start_time, end_time)?;

        let (main_title, main_mw_percent) = if let Some(max_category) =
            category_mw.into_iter().max_by_key(|c| c.minutes_watched)
        {
            let percent = if mw > 0 {
                (max_category.minutes_watched as f64 / mw as f64) * 100.0
            } else {
                0.0
            };
            (Some(max_category.category), Some(percent))
        } else {
            (None, None)
        };

        results.push(BroadcasterAnalytics {
            channel_id: ch_id,
            channel_name: ch_name,
            minutes_watched: mw,
            hours_broadcasted: hours,
            average_ccu: avg_ccu,
            main_played_title: main_title,
            main_title_mw_percent: main_mw_percent,
        });
    }

    Ok(results)
}

/// カテゴリ別 MinutesWatched を取得（配信者ごとのメインタイトル計算用）
fn get_category_minutes_watched(
    conn: &Connection,
    channel_id: i64,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<CategoryMinutesWatched>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                ss.category,
                ss.viewer_count,
                ss.collected_at,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            JOIN streams s ON ss.stream_id = s.id
            WHERE s.channel_id = ?
        "#,
    );

    let mut params: Vec<String> = vec![channel_id.to_string()];

    if let Some(start) = start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
        )
        SELECT 
            COALESCE(category, 'Unknown') AS category,
            COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched
        FROM stats_with_interval
        WHERE viewer_count IS NOT NULL
        GROUP BY category
        ORDER BY minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let results: Vec<CategoryMinutesWatched> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(CategoryMinutesWatched {
                category: row.get(0)?,
                minutes_watched: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// ゲームタイトル別統計を取得
pub fn get_game_analytics(
    conn: &Connection,
    category: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<GameAnalytics>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                s.channel_id,
                c.channel_name,
                ss.category,
                ss.viewer_count,
                ss.collected_at,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            JOIN streams s ON ss.stream_id = s.id
            JOIN channels c ON s.channel_id = c.id
            WHERE ss.category IS NOT NULL
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(cat) = category {
        sql.push_str(" AND ss.category = ?");
        params.push(cat.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
        ),
        game_stats AS (
            SELECT 
                category,
                COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
                COALESCE(AVG(viewer_count), 0) AS average_ccu,
                COUNT(DISTINCT channel_id) AS unique_broadcasters
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL
            GROUP BY category
        )
        SELECT 
            category,
            minutes_watched,
            average_ccu,
            unique_broadcasters
        FROM game_stats
        ORDER BY minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let game_stats: Vec<_> = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok((
            row.get::<_, String>(0)?,  // category
            row.get::<_, i64>(1)?,     // minutes_watched
            row.get::<_, f64>(2)?,     // average_ccu
            row.get::<_, i32>(3)?,     // unique_broadcasters
        ))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    // HoursBroadcasted をカテゴリ別に計算
    let mut hours_sql = String::from(
        r#"
        SELECT 
            COALESCE(ss.category, 'Unknown') AS category,
            COALESCE(SUM(
                EXTRACT(EPOCH FROM (
                    COALESCE(
                        (SELECT MAX(collected_at) FROM stream_stats WHERE stream_id = s.id),
                        s.started_at
                    ) - s.started_at
                )) / 3600.0
            ), 0) AS hours_broadcasted
        FROM streams s
        LEFT JOIN stream_stats ss ON s.id = ss.stream_id
        WHERE ss.category IS NOT NULL
        "#,
    );

    let mut hours_params: Vec<String> = Vec::new();

    if let Some(cat) = category {
        hours_sql.push_str(" AND ss.category = ?");
        hours_params.push(cat.to_string());
    }

    hours_sql.push_str(" GROUP BY ss.category");

    let mut hours_stmt = conn.prepare(&hours_sql)?;
    let hours_map: std::collections::HashMap<String, f64> =
        utils::query_map_with_params(&mut hours_stmt, &hours_params, |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .collect::<Result<_, _>>()?;

    // トップチャンネルを取得
    let mut results = Vec::new();
    for (cat, mw, avg_ccu, unique_bc) in game_stats {
        let hours = hours_map.get(&cat).copied().unwrap_or(0.0);
        let top_channel = get_top_channel_for_category(conn, &cat, start_time, end_time)?;

        results.push(GameAnalytics {
            category: cat,
            minutes_watched: mw,
            hours_broadcasted: hours,
            average_ccu: avg_ccu,
            unique_broadcasters: unique_bc,
            top_channel,
        });
    }

    Ok(results)
}

/// カテゴリごとのトップチャンネルを取得
fn get_top_channel_for_category(
    conn: &Connection,
    category: &str,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Option<String>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                c.channel_name,
                ss.viewer_count,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY ss.stream_id ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            JOIN streams s ON ss.stream_id = s.id
            JOIN channels c ON s.channel_id = c.id
            WHERE ss.category = ?
        "#,
    );

    let mut params: Vec<String> = vec![category.to_string()];

    if let Some(start) = start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
        )
        SELECT 
            channel_name,
            COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched
        FROM stats_with_interval
        WHERE viewer_count IS NOT NULL
        GROUP BY channel_name
        ORDER BY minutes_watched DESC
        LIMIT 1
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let result: Option<String> =
        utils::query_map_with_params(&mut stmt, &params, |row| Ok(row.get::<_, String>(0)?))
            .ok()
            .and_then(|mut iter| iter.next())
            .and_then(|r| r.ok());

    Ok(result)
}
