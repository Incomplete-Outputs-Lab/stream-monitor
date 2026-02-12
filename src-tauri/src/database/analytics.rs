use crate::database::{
    repositories::{AggregationRepository, StreamStatsRepository},
    utils,
};
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// 配信者別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcasterAnalytics {
    pub channel_id: i64,
    pub channel_name: String,
    pub login_name: String, // Twitch login name (小文字、URL用)
    pub minutes_watched: i64,
    pub hours_broadcasted: f64,
    pub average_ccu: f64,
    pub main_played_title: String,
    pub main_title_mw_percent: f64,
    // 新規追加
    pub peak_ccu: i32,
    pub stream_count: i32,
    pub total_chat_messages: i64,
    pub avg_chat_rate: f64,
    pub unique_chatters: i32,
    pub engagement_rate: f64,
    pub category_count: i32,
}

/// ゲームタイトル別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalytics {
    pub game_id: String,  // Twitch game ID（プライマリキー）
    pub category: String, // カテゴリ名（表示用、game_categoriesから取得）
    pub minutes_watched: i64,
    pub hours_broadcasted: f64,
    pub average_ccu: f64,
    pub unique_broadcasters: i32,
    pub top_channel: String,
    pub top_channel_login: String, // Twitch login name of top channel (URL用)
    pub total_chat_messages: i64,
    pub avg_chat_rate: f64,
    pub engagement_rate: f64,
}

/// データ可用性情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAvailability {
    pub first_record: String,
    pub last_record: String,
    pub total_days_with_data: i32,
    pub total_records: i64,
}

/// 日次統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub minutes_watched: i64,
    pub hours_broadcasted: f64,
    pub average_ccu: f64,
    pub collection_hours: f64,
}

/// 配信者別統計を取得
///
/// AggregationRepositoryを使用して統計を計算します。
pub fn get_broadcaster_analytics(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<BroadcasterAnalytics>, duckdb::Error> {
    AggregationRepository::calculate_broadcaster_analytics(conn, channel_id, start_time, end_time)
}

/// 配信者別統計を取得（旧実装 - 使用しない）
#[allow(dead_code)]
fn get_broadcaster_analytics_old(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<BroadcasterAnalytics>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                COALESCE(s.channel_id, c2.id) as channel_id,
                COALESCE(c1.channel_name, c2.channel_name, ss.channel_name) as channel_name,
                ss.stream_id,
                ss.viewer_count,
                ss.category,
                COALESCE((
                    SELECT COUNT(*)
                    FROM chat_messages cm
                    WHERE cm.stream_id = ss.stream_id
                      AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                      AND cm.timestamp < ss.collected_at
                ), 0) AS chat_rate_1min,
                ss.collected_at,
                ss.twitch_user_id,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY COALESCE(CAST(ss.stream_id AS VARCHAR), ss.channel_name || '_' || CAST(DATE(ss.collected_at) AS VARCHAR)) ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            LEFT JOIN channels c1 ON s.channel_id = c1.id
            LEFT JOIN channels c2 ON ss.channel_name = c2.channel_id AND c2.platform = 'twitch'
            WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    // channel_id が指定されている場合、channel_name を取得してフィルター
    let filter_channel_name = if let Some(ch_id) = channel_id {
        conn.query_row(
            "SELECT channel_name FROM channels WHERE id = ? AND platform = 'twitch'",
            [ch_id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .ok()
    } else {
        None
    };

    if let Some(ref ch_name) = filter_channel_name {
        sql.push_str(" AND ss.channel_name = ?");
        params.push(ch_name.clone());
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
                channel_name,
                COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
                COALESCE(SUM(COALESCE(interval_minutes, 1)) / 60.0, 0) AS hours_broadcasted,
                COALESCE(AVG(viewer_count), 0) AS average_ccu,
                COALESCE(MAX(viewer_count), 0) AS peak_ccu,
                COUNT(DISTINCT stream_id) AS stream_count,
                COALESCE(SUM(chat_rate_1min * COALESCE(interval_minutes, 1)), 0)::BIGINT AS total_chat_messages,
                COALESCE(AVG(chat_rate_1min), 0) AS avg_chat_rate,
                COUNT(DISTINCT category) AS category_count
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL
                AND channel_name IS NOT NULL
            GROUP BY channel_name
        ),
        category_mw AS (
            SELECT 
                channel_name,
                category,
                COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
                ROW_NUMBER() OVER (PARTITION BY channel_name ORDER BY SUM(viewer_count * COALESCE(interval_minutes, 1)) DESC) as rn
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL 
                AND category IS NOT NULL
                AND channel_name IS NOT NULL
            GROUP BY channel_name, category
        ),
        main_category AS (
            SELECT 
                channel_name,
                category as main_title,
                minutes_watched as main_mw
            FROM category_mw
            WHERE rn = 1
        )
        SELECT 
            COALESCE(c.id, 0) as channel_id,
            cs.channel_name,
            cs.minutes_watched,
            cs.hours_broadcasted,
            cs.average_ccu,
            cs.peak_ccu,
            cs.stream_count,
            cs.total_chat_messages,
            cs.avg_chat_rate,
            cs.category_count,
            mc.main_title,
            mc.main_mw
        FROM channel_stats cs
        LEFT JOIN main_category mc ON cs.channel_name = mc.channel_name
        LEFT JOIN channels c ON (cs.channel_name = c.channel_id AND c.platform = 'twitch')
        ORDER BY cs.minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let channel_stats: Vec<_> = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok((
            row.get::<_, i64>(0)?,             // channel_id
            row.get::<_, String>(1)?,          // channel_name
            row.get::<_, i64>(2)?,             // minutes_watched
            row.get::<_, f64>(3)?,             // hours_broadcasted
            row.get::<_, f64>(4)?,             // average_ccu
            row.get::<_, i32>(5)?,             // peak_ccu
            row.get::<_, i32>(6)?,             // stream_count
            row.get::<_, i64>(7)?,             // total_chat_messages
            row.get::<_, f64>(8)?,             // avg_chat_rate
            row.get::<_, i32>(9)?,             // category_count
            row.get::<_, Option<String>>(10)?, // main_title
            row.get::<_, Option<i64>>(11)?,    // main_mw
        ))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    // ユニークチャッター数を取得
    let mut chatters_sql = String::from(
        r#"
        SELECT
            c.id AS channel_id,
            COUNT(DISTINCT cm.user_id) AS unique_chatters
        FROM channels c
        LEFT JOIN streams s ON c.id = s.channel_id
        LEFT JOIN chat_messages cm ON s.id = cm.stream_id
        WHERE 1=1
        "#,
    );

    let mut chatters_params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        chatters_sql.push_str(&format!(" AND c.id = {}", ch_id));
    }

    if let Some(start) = start_time {
        chatters_sql.push_str(" AND cm.timestamp >= ?");
        chatters_params.push(start.to_string());
    }

    if let Some(end) = end_time {
        chatters_sql.push_str(" AND cm.timestamp <= ?");
        chatters_params.push(end.to_string());
    }

    chatters_sql.push_str(" GROUP BY c.id");

    let mut chatters_stmt = conn.prepare(&chatters_sql)?;
    let chatters_map: std::collections::HashMap<i64, i32> =
        utils::query_map_with_params(&mut chatters_stmt, &chatters_params, |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
        })?
        .collect::<Result<_, _>>()?;

    // 結果を構築
    let mut results = Vec::new();
    for (
        ch_id,
        ch_name,
        mw,
        hours,
        avg_ccu,
        peak_ccu,
        stream_count,
        total_chat,
        avg_chat_rate,
        category_count,
        main_title,
        main_mw,
    ) in channel_stats
    {
        let unique_chatters = chatters_map.get(&ch_id).copied().unwrap_or(0);

        // エンゲージメント率を計算 (チャット数 / (視聴者数 * 時間))
        let engagement_rate = if mw > 0 {
            (total_chat as f64 / mw as f64) * 1000.0 // 1000視聴分あたりのメッセージ数
        } else {
            0.0
        };

        // メインタイトルのMW割合を計算
        let main_mw_percent = if let Some(main_mw_val) = main_mw {
            if mw > 0 {
                Some((main_mw_val as f64 / mw as f64) * 100.0)
            } else {
                Some(0.0)
            }
        } else {
            None
        };

        results.push(BroadcasterAnalytics {
            channel_id: ch_id,
            channel_name: ch_name.clone(),
            login_name: ch_name,
            minutes_watched: mw,
            hours_broadcasted: hours,
            average_ccu: avg_ccu,
            main_played_title: main_title.unwrap_or_default(),
            main_title_mw_percent: main_mw_percent.unwrap_or_default(),
            peak_ccu,
            stream_count,
            total_chat_messages: total_chat,
            avg_chat_rate,
            unique_chatters,
            engagement_rate,
            category_count,
        });
    }

    Ok(results)
}

/// ゲームタイトル別統計を取得（game_idベース）
///
/// AggregationRepositoryを使用して統計を計算します。
pub fn get_game_analytics(
    conn: &Connection,
    game_id: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<GameAnalytics>, duckdb::Error> {
    AggregationRepository::calculate_game_analytics(conn, game_id, start_time, end_time)
}

/// ゲームタイトル別統計を取得（旧実装 - 使用しない）
#[allow(dead_code)]
fn get_game_analytics_old(
    conn: &Connection,
    category: Option<&str>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<GameAnalytics>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                COALESCE(s.channel_id, c2.id) as channel_id,
                COALESCE(c1.channel_name, c2.channel_name, ss.channel_name) as channel_name,
                ss.category,
                ss.viewer_count,
                ss.collected_at,
                ss.stream_id,
                ss.twitch_user_id,
                COALESCE((
                    SELECT COUNT(*)
                    FROM chat_messages cm
                    WHERE cm.stream_id = ss.stream_id
                      AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                      AND cm.timestamp < ss.collected_at
                ), 0) AS chat_rate_1min,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY COALESCE(CAST(ss.stream_id AS VARCHAR), ss.channel_name || '_' || CAST(DATE(ss.collected_at) AS VARCHAR)) ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            LEFT JOIN channels c1 ON s.channel_id = c1.id
            LEFT JOIN channels c2 ON ss.channel_name = c2.channel_id AND c2.platform = 'twitch'
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
                COALESCE(SUM(COALESCE(interval_minutes, 1)) / 60.0, 0) AS hours_broadcasted,
                COALESCE(AVG(viewer_count), 0) AS average_ccu,
                COUNT(DISTINCT channel_name) AS unique_broadcasters,
                COALESCE(SUM(chat_rate_1min * COALESCE(interval_minutes, 1)), 0)::BIGINT AS total_chat_messages,
                COALESCE(AVG(chat_rate_1min), 0) AS avg_chat_rate
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL
                AND channel_name IS NOT NULL
            GROUP BY category
        ),
        channel_by_category AS (
            SELECT 
                category,
                channel_name,
                COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS channel_mw,
                ROW_NUMBER() OVER (PARTITION BY category ORDER BY SUM(viewer_count * COALESCE(interval_minutes, 1)) DESC) as rn
            FROM stats_with_interval
            WHERE viewer_count IS NOT NULL 
                AND channel_name IS NOT NULL
            GROUP BY category, channel_name
        ),
        top_channels AS (
            SELECT 
                category,
                channel_name as top_channel
            FROM channel_by_category
            WHERE rn = 1
        )
        SELECT 
            gs.category,
            gs.minutes_watched,
            gs.hours_broadcasted,
            gs.average_ccu,
            gs.unique_broadcasters,
            tc.top_channel,
            gs.total_chat_messages,
            gs.avg_chat_rate,
            CASE 
                WHEN gs.minutes_watched > 0 
                THEN (gs.total_chat_messages::DOUBLE / gs.minutes_watched::DOUBLE) * 1000.0
                ELSE 0.0
            END as engagement_rate
        FROM game_stats gs
        LEFT JOIN top_channels tc ON gs.category = tc.category
        ORDER BY gs.minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let results: Vec<GameAnalytics> = utils::query_map_with_params(&mut stmt, &params, |row| {
        let top_channel_login = row.get::<_, String>(5).unwrap_or_default();
        Ok(GameAnalytics {
            game_id: String::new(), // Old implementation doesn't have game_id
            category: row.get::<_, String>(0)?,
            minutes_watched: row.get::<_, i64>(1)?,
            hours_broadcasted: row.get::<_, f64>(2)?,
            average_ccu: row.get::<_, f64>(3)?,
            unique_broadcasters: row.get::<_, i32>(4)?,
            top_channel: top_channel_login.clone(),
            top_channel_login,
            total_chat_messages: row.get::<_, i64>(6)?,
            avg_chat_rate: row.get::<_, f64>(7)?,
            engagement_rate: row.get::<_, f64>(8)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// カテゴリ一覧を取得（MinutesWatched降順）
///
/// AggregationRepositoryを使用してカテゴリを取得します。
pub fn list_categories(
    conn: &Connection,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<String>, duckdb::Error> {
    AggregationRepository::list_categories(conn, start_time, end_time)
}

/// カテゴリ一覧を取得（旧実装 - 使用しない）
#[allow(dead_code)]
fn list_categories_old(
    conn: &Connection,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<String>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT 
                ss.category,
                ss.viewer_count,
                ss.stream_id,
                ss.twitch_user_id,
                ss.collected_at,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY COALESCE(CAST(ss.stream_id AS VARCHAR), ss.twitch_user_id || '_' || CAST(DATE(ss.collected_at) AS VARCHAR)) ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            WHERE ss.category IS NOT NULL
        "#,
    );

    let mut params: Vec<String> = Vec::new();

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
            category,
            COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched
        FROM stats_with_interval
        WHERE viewer_count IS NOT NULL
        GROUP BY category
        ORDER BY minutes_watched DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let categories: Vec<String> =
        utils::query_map_with_params(&mut stmt, &params, |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

    Ok(categories)
}

/// データ可用性情報を取得
///
/// StreamStatsRepositoryを使用してデータ可用性を取得します。
pub fn get_data_availability(conn: &Connection) -> Result<DataAvailability, duckdb::Error> {
    let (first_record, last_record, total_days_with_data, total_records) =
        StreamStatsRepository::get_data_availability(conn)?;

    Ok(DataAvailability {
        first_record,
        last_record,
        total_days_with_data: total_days_with_data as i32,
        total_records,
    })
}

/// ゲーム別日次統計を取得（game_idベース）
///
/// StreamStatsRepositoryを使用して日次統計を取得します。
pub fn get_game_daily_stats(
    conn: &Connection,
    game_id: &str,
    start_time: &str,
    end_time: &str,
) -> Result<Vec<DailyStats>, duckdb::Error> {
    StreamStatsRepository::get_game_daily_stats(conn, game_id, start_time, end_time)
}

/// ゲーム別日次統計を取得（旧実装 - 使用しない）
#[allow(dead_code)]
fn get_game_daily_stats_old(
    conn: &Connection,
    category: &str,
    start_time: &str,
    end_time: &str,
) -> Result<Vec<DailyStats>, duckdb::Error> {
    let sql = r#"
        WITH stats_with_interval AS (
            SELECT 
                DATE(ss.collected_at) as date,
                ss.viewer_count,
                ss.stream_id,
                ss.channel_name,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY COALESCE(CAST(ss.stream_id AS VARCHAR), ss.channel_name || '_' || CAST(DATE(ss.collected_at) AS VARCHAR)) ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes,
                ss.collected_at
            FROM stream_stats ss
            WHERE ss.category = ?
                AND ss.collected_at >= ?
                AND ss.collected_at <= ?
        ),
        daily_broadcast_hours AS (
            SELECT 
                DATE(s.started_at) as date,
                COALESCE(SUM(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 3600.0
                ), 0) AS hours_broadcasted
            FROM streams s
            JOIN stream_stats ss ON s.id = ss.stream_id
            WHERE ss.category = ?
                AND s.started_at >= ?
                AND s.started_at <= ?
            GROUP BY DATE(s.started_at)
        )
        SELECT 
            swi.date::VARCHAR as date,
            COALESCE(SUM(swi.viewer_count * COALESCE(swi.interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
            COALESCE(dbh.hours_broadcasted, 0) AS hours_broadcasted,
            COALESCE(AVG(swi.viewer_count), 0) AS average_ccu,
            COALESCE(SUM(COALESCE(swi.interval_minutes, 1)) / 60.0, 0) AS collection_hours
        FROM stats_with_interval swi
        LEFT JOIN daily_broadcast_hours dbh ON swi.date = dbh.date
        WHERE swi.viewer_count IS NOT NULL
        GROUP BY swi.date, dbh.hours_broadcasted
        ORDER BY swi.date
    "#;

    let mut stmt = conn.prepare(sql)?;
    let params = vec![
        category.to_string(),
        start_time.to_string(),
        end_time.to_string(),
        category.to_string(),
        start_time.to_string(),
        end_time.to_string(),
    ];

    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok(DailyStats {
            date: row.get(0)?,
            minutes_watched: row.get(1)?,
            hours_broadcasted: row.get(2)?,
            average_ccu: row.get(3)?,
            collection_hours: row.get(4)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}

/// チャンネル別日次統計を取得
///
/// StreamStatsRepositoryを使用して日次統計を取得します。
pub fn get_channel_daily_stats(
    conn: &Connection,
    channel_id: i64,
    start_time: &str,
    end_time: &str,
) -> Result<Vec<DailyStats>, duckdb::Error> {
    StreamStatsRepository::get_channel_daily_stats(conn, channel_id, start_time, end_time)
}

/// チャンネル別日次統計を取得（旧実装 - 使用しない）
#[allow(dead_code)]
fn get_channel_daily_stats_old(
    conn: &Connection,
    channel_id: i64,
    start_time: &str,
    end_time: &str,
) -> Result<Vec<DailyStats>, duckdb::Error> {
    let sql = r#"
        WITH channel_lookup AS (
            SELECT channel_name FROM channels WHERE id = ? AND platform = 'twitch'
        ),
        stats_with_interval AS (
            SELECT 
                DATE(ss.collected_at) as date,
                ss.viewer_count,
                ss.stream_id,
                ss.channel_name,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (PARTITION BY COALESCE(CAST(ss.stream_id AS VARCHAR), ss.channel_name || '_' || CAST(DATE(ss.collected_at) AS VARCHAR)) ORDER BY ss.collected_at) 
                    - ss.collected_at
                )) / 60.0 AS interval_minutes,
                ss.collected_at
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            LEFT JOIN channels c ON (s.channel_id = c.id OR (ss.channel_name = c.channel_id AND c.platform = 'twitch'))
            WHERE (COALESCE(s.channel_id, c.id) = ? OR ss.channel_name = (SELECT channel_id FROM channel_lookup))
                AND ss.collected_at >= ?
                AND ss.collected_at <= ?
        ),
        daily_broadcast_hours AS (
            SELECT 
                DATE(s.started_at) as date,
                COALESCE(SUM(
                    EXTRACT(EPOCH FROM (
                        COALESCE(s.ended_at, CAST(CURRENT_TIMESTAMP AS TIMESTAMP)) - s.started_at
                    )) / 3600.0
                ), 0) AS hours_broadcasted
            FROM streams s
            WHERE s.channel_id = ?
                AND s.started_at >= ?
                AND s.started_at <= ?
            GROUP BY DATE(s.started_at)
        )
        SELECT 
            swi.date::VARCHAR as date,
            COALESCE(SUM(swi.viewer_count * COALESCE(swi.interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
            COALESCE(dbh.hours_broadcasted, 0) AS hours_broadcasted,
            COALESCE(AVG(swi.viewer_count), 0) AS average_ccu,
            COALESCE(SUM(COALESCE(swi.interval_minutes, 1)) / 60.0, 0) AS collection_hours
        FROM stats_with_interval swi
        LEFT JOIN daily_broadcast_hours dbh ON swi.date = dbh.date
        WHERE swi.viewer_count IS NOT NULL
        GROUP BY swi.date, dbh.hours_broadcasted
        ORDER BY swi.date
    "#;

    let mut stmt = conn.prepare(sql)?;
    let params = vec![
        channel_id.to_string(),
        start_time.to_string(),
        end_time.to_string(),
        channel_id.to_string(),
        start_time.to_string(),
        end_time.to_string(),
    ];

    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok(DailyStats {
            date: row.get(0)?,
            minutes_watched: row.get(1)?,
            hours_broadcasted: row.get(2)?,
            average_ccu: row.get(3)?,
            collection_hours: row.get(4)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}
