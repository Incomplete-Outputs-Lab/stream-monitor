use crate::database::utils;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// エンゲージメント統計（時系列）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatEngagementStats {
    pub timestamp: String,
    pub chat_count: i64,
    pub unique_chatters: i64,
    pub viewer_count: i32,
    pub engagement_rate: f64,
}

/// チャットスパイク情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSpike {
    pub timestamp: String,
    pub chat_count: i64,
    pub spike_ratio: f64,
    pub prev_count: i64,
}

/// ユーザーセグメント統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentStats {
    pub segment: String, // "subscriber", "vip", "moderator", "regular"
    pub message_count: i64,
    pub user_count: i64,
    pub avg_messages_per_user: f64,
    pub percentage: f64,
}

/// 上位チャッター情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopChatter {
    pub user_name: String,
    pub message_count: i64,
    pub badges: Vec<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub stream_count: i64,
}

/// 時間パターン統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimePatternStats {
    pub hour: i32,                // 0-23
    pub day_of_week: Option<i32>, // 0-6 (Sunday-Saturday)
    pub avg_chat_rate: f64,
    pub avg_engagement: f64,
    pub total_messages: i64,
}

/// チャッター行動統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatterBehaviorStats {
    pub total_unique_chatters: i64,
    pub repeater_count: i64,
    pub new_chatter_count: i64,
    pub repeater_percentage: f64,
    pub avg_participation_rate: f64,
}

/// エンゲージメント統計を時系列で取得（5分間隔）
pub fn get_chat_engagement_timeline(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    interval_minutes: i32,
) -> Result<Vec<ChatEngagementStats>, duckdb::Error> {
    let mut sql = format!(
        r#"
        WITH time_buckets AS (
            SELECT 
                time_bucket(INTERVAL '{} minutes', cm.timestamp) as bucket,
                COUNT(*) as chat_count,
                COUNT(DISTINCT cm.user_name) as unique_chatters
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
        interval_minutes
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(st_id) = stream_id {
        sql.push_str(" AND cm.stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
            GROUP BY bucket
        ),
        viewer_stats AS (
            SELECT 
                time_bucket(INTERVAL '5 minutes', ss.collected_at) as bucket,
                AVG(ss.viewer_count) as avg_viewers
            FROM stream_stats ss
            WHERE ss.viewer_count IS NOT NULL
        "#,
    );

    // viewer_params_offset is not used in this version

    if let Some(ch_id) = channel_id {
        // channel_idからchannel_nameを取得
        let channel_name = conn
            .query_row(
                "SELECT channel_id FROM channels WHERE id = ?",
                [ch_id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .ok();
        if let Some(ch_name) = channel_name {
            sql.push_str(" AND ss.channel_name = ?");
            params.push(ch_name);
        }
    }

    if let Some(st_id) = stream_id {
        sql.push_str(" AND ss.stream_id = ?");
        params.push(st_id.to_string());
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
            GROUP BY bucket
        )
        SELECT 
            tb.bucket::VARCHAR as timestamp,
            tb.chat_count,
            tb.unique_chatters,
            COALESCE(vs.avg_viewers, 0)::INTEGER as viewer_count,
            CASE 
                WHEN vs.avg_viewers > 0 THEN (tb.chat_count::FLOAT / vs.avg_viewers) * 100.0
                ELSE 0.0
            END as engagement_rate
        FROM time_buckets tb
        LEFT JOIN viewer_stats vs ON tb.bucket = vs.bucket
        ORDER BY tb.bucket
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok(ChatEngagementStats {
            timestamp: row.get(0)?,
            chat_count: row.get(1)?,
            unique_chatters: row.get(2)?,
            viewer_count: row.get(3)?,
            engagement_rate: row.get(4)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}

/// チャットスパイク（急増ポイント）を検出
pub fn detect_chat_spikes(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    min_spike_ratio: f64,
) -> Result<Vec<ChatSpike>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH time_buckets AS (
            SELECT 
                time_bucket(INTERVAL '5 minutes', cm.timestamp) as bucket,
                COUNT(*) as chat_count
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(st_id) = stream_id {
        sql.push_str(" AND cm.stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
            GROUP BY bucket
        ),
        with_lag AS (
            SELECT 
                bucket,
                chat_count,
                LAG(chat_count, 1, 0) OVER (ORDER BY bucket) as prev_count
            FROM time_buckets
        )
        SELECT 
            bucket::VARCHAR as timestamp,
            chat_count,
            CASE 
                WHEN prev_count > 0 THEN (chat_count::FLOAT / prev_count::FLOAT)
                ELSE 0.0
            END as spike_ratio,
            prev_count
        FROM with_lag
        WHERE prev_count > 0 
            AND chat_count::FLOAT / prev_count::FLOAT >= ?
        ORDER BY spike_ratio DESC
        LIMIT 20
        "#,
    );

    params.push(min_spike_ratio.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok(ChatSpike {
            timestamp: row.get(0)?,
            chat_count: row.get(1)?,
            spike_ratio: row.get(2)?,
            prev_count: row.get(3)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}

/// ユーザーセグメント別統計を取得
pub fn get_user_segment_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<UserSegmentStats>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH all_messages AS (
            SELECT 
                cm.user_name,
                cm.badges,
                COUNT(*) as message_count
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(st_id) = stream_id {
        sql.push_str(" AND cm.stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
            GROUP BY cm.user_name, cm.badges
        ),
        classified_messages AS (
            SELECT 
                message_count,
                CASE 
                    WHEN badges IS NULL THEN 'regular'
                    WHEN list_contains(badges, 'broadcaster') THEN 'broadcaster'
                    WHEN list_contains(badges, 'moderator') THEN 'moderator'
                    WHEN list_contains(badges, 'vip') THEN 'vip'
                    WHEN list_contains(badges, 'subscriber') THEN 'subscriber'
                    ELSE 'regular'
                END as segment
            FROM all_messages
        ),
        segment_stats AS (
            SELECT 
                segment,
                SUM(message_count) as total_messages,
                COUNT(*) as user_count
            FROM classified_messages
            GROUP BY segment
        ),
        total_stats AS (
            SELECT 
                SUM(message_count) as grand_total
            FROM classified_messages
        )
        SELECT 
            ss.segment,
            ss.total_messages,
            ss.user_count,
            ss.total_messages::FLOAT / ss.user_count::FLOAT as avg_messages_per_user,
            (ss.total_messages::FLOAT / ts.grand_total::FLOAT) * 100.0 as percentage
        FROM segment_stats ss
        CROSS JOIN total_stats ts
        ORDER BY ss.total_messages DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok(UserSegmentStats {
            segment: row.get(0)?,
            message_count: row.get(1)?,
            user_count: row.get(2)?,
            avg_messages_per_user: row.get(3)?,
            percentage: row.get(4)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}

/// 上位チャッターを取得
pub fn get_top_chatters(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    limit: i32,
) -> Result<Vec<TopChatter>, duckdb::Error> {
    // N+1クエリを避けるため、バッジ取得をメインクエリに統合
    let mut sql = String::from(
        r#"
        WITH user_badges AS (
            SELECT 
                user_name,
                badges,
                ROW_NUMBER() OVER (PARTITION BY user_name ORDER BY timestamp DESC) as rn
            FROM chat_messages
            WHERE badges IS NOT NULL
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    // CTEにも同じフィルタを適用
    let mut cte_filters = Vec::new();

    if let Some(ch_id) = channel_id {
        cte_filters.push(" AND channel_id = ?");
        params.push(ch_id.to_string());
    }

    if let Some(st_id) = stream_id {
        cte_filters.push(" AND stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        cte_filters.push(" AND timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        cte_filters.push(" AND timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(&cte_filters.join(""));
    sql.push_str(
        r#"
        )
        SELECT 
            cm.user_name,
            COUNT(*) as message_count,
            MIN(cm.timestamp)::VARCHAR as first_seen,
            MAX(cm.timestamp)::VARCHAR as last_seen,
            COUNT(DISTINCT cm.stream_id) as stream_count,
            ub.badges
        FROM chat_messages cm
        LEFT JOIN streams s ON cm.stream_id = s.id
        LEFT JOIN user_badges ub ON cm.user_name = ub.user_name AND ub.rn = 1
        WHERE 1=1
        "#,
    );

    // メインクエリのWHERE句（CTEとは別にパラメータを追加）
    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(st_id) = stream_id {
        sql.push_str(" AND cm.stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
        GROUP BY cm.user_name, ub.badges
        ORDER BY message_count DESC
        LIMIT ?
        "#,
    );
    params.push(limit.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let results: Vec<TopChatter> = utils::query_map_with_params(&mut stmt, &params, |row| {
        let badges_str: Option<String> = row.get(5).ok();
        let badges = if let Some(b) = badges_str {
            // DuckDBのARRAY型をパース
            crate::database::utils::parse_badges(&b).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(TopChatter {
            user_name: row.get(0)?,
            message_count: row.get(1)?,
            badges,
            first_seen: row.get(2)?,
            last_seen: row.get(3)?,
            stream_count: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

/// 時間パターン統計を取得（時間帯別）
pub fn get_time_pattern_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    group_by_day: bool,
) -> Result<Vec<TimePatternStats>, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH hourly_messages AS (
            SELECT 
                EXTRACT(HOUR FROM cm.timestamp) as hour,
        "#,
    );

    if group_by_day {
        sql.push_str("EXTRACT(DOW FROM cm.timestamp) as day_of_week,");
    }

    sql.push_str(
        r#"
                time_bucket(INTERVAL '1 hour', cm.timestamp) as bucket,
                COUNT(*) as message_count,
                cm.stream_id
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str("GROUP BY hour");
    if group_by_day {
        sql.push_str(", day_of_week");
    }
    sql.push_str(", bucket, cm.stream_id");

    sql.push_str(
        r#"
        ),
        viewer_stats AS (
            SELECT 
                time_bucket(INTERVAL '1 hour', ss.collected_at) as bucket,
                AVG(ss.viewer_count) as avg_viewers
            FROM stream_stats ss
            WHERE ss.viewer_count IS NOT NULL
        "#,
    );

    if let Some(ch_id) = channel_id {
        let channel_name = conn
            .query_row(
                "SELECT channel_id FROM channels WHERE id = ?",
                [ch_id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .ok();
        if let Some(ch_name) = channel_name {
            sql.push_str(" AND ss.channel_name = ?");
            params.push(ch_name);
        }
    }

    if let Some(start) = start_time {
        sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    sql.push_str("GROUP BY bucket");
    sql.push_str(
        r#"
        ),
        combined AS (
            SELECT 
                hm.hour,
        "#,
    );

    if group_by_day {
        sql.push_str("hm.day_of_week,");
    }

    sql.push_str(
        r#"
                hm.message_count,
                COALESCE(vs.avg_viewers, 1) as avg_viewers
            FROM hourly_messages hm
            LEFT JOIN viewer_stats vs ON hm.bucket = vs.bucket
        )
        SELECT 
            hour,
        "#,
    );

    if group_by_day {
        sql.push_str("day_of_week,");
    }

    sql.push_str(
        r#"
            AVG(message_count) as avg_chat_rate,
            AVG(message_count / avg_viewers) * 100.0 as avg_engagement,
            SUM(message_count) as total_messages
        FROM combined
        GROUP BY hour
        "#,
    );

    if group_by_day {
        sql.push_str(", day_of_week ORDER BY day_of_week, hour");
    } else {
        sql.push_str("ORDER BY hour");
    }

    let mut stmt = conn.prepare(&sql)?;
    let results = utils::query_map_with_params(&mut stmt, &params, |row| {
        let hour: i32 = row.get(0)?;
        let mut col_idx = 1;

        let day_of_week = if group_by_day {
            let dow: i32 = row.get(col_idx)?;
            col_idx += 1;
            Some(dow)
        } else {
            None
        };

        Ok(TimePatternStats {
            hour,
            day_of_week,
            avg_chat_rate: row.get(col_idx)?,
            avg_engagement: row.get(col_idx + 1)?,
            total_messages: row.get(col_idx + 2)?,
        })
    })?;

    results.collect::<Result<Vec<_>, _>>()
}

/// チャッター行動統計を取得（リピーター率など）
pub fn get_chatter_behavior_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<ChatterBehaviorStats, duckdb::Error> {
    let mut sql = String::from(
        r#"
        WITH chatter_streams AS (
            SELECT 
                cm.user_name,
                COUNT(DISTINCT cm.stream_id) as stream_count,
                COUNT(*) as message_count
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE cm.stream_id IS NOT NULL
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ch_id) = channel_id {
        sql.push_str(" AND (cm.channel_id = ? OR s.channel_id = ?)");
        params.push(ch_id.to_string());
        params.push(ch_id.to_string());
    }

    if let Some(start) = start_time {
        sql.push_str(" AND cm.timestamp >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        sql.push_str(" AND cm.timestamp <= ?");
        params.push(end.to_string());
    }

    sql.push_str(
        r#"
            GROUP BY cm.user_name
        ),
        stream_viewers AS (
            SELECT 
                ss.stream_id,
                MAX(ss.viewer_count) as peak_ccu
            FROM stream_stats ss
            WHERE ss.viewer_count IS NOT NULL
        "#,
    );

    if let Some(ch_id) = channel_id {
        let channel_name = conn
            .query_row(
                "SELECT channel_id FROM channels WHERE id = ?",
                [ch_id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .ok();
        if let Some(ch_name) = channel_name {
            sql.push_str(" AND ss.channel_name = ?");
            params.push(ch_name);
        }
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
            GROUP BY ss.stream_id
        )
        SELECT 
            COUNT(DISTINCT cs.user_name) as total_unique_chatters,
            SUM(CASE WHEN cs.stream_count > 1 THEN 1 ELSE 0 END) as repeater_count,
            SUM(CASE WHEN cs.stream_count = 1 THEN 1 ELSE 0 END) as new_chatter_count,
            AVG(CASE WHEN sv.peak_ccu > 0 THEN cs.message_count::FLOAT / sv.peak_ccu::FLOAT ELSE 0 END) * 100.0 as avg_participation_rate
        FROM chatter_streams cs
        CROSS JOIN (SELECT AVG(peak_ccu) as peak_ccu FROM stream_viewers) sv
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = utils::query_map_with_params(&mut stmt, &params, |row| {
        let total_unique: i64 = row.get(0)?;
        let repeater: i64 = row.get(1)?;
        let new_chatter: i64 = row.get(2)?;
        let avg_participation: f64 = row.get(3)?;

        let repeater_percentage = if total_unique > 0 {
            (repeater as f64 / total_unique as f64) * 100.0
        } else {
            0.0
        };

        Ok(ChatterBehaviorStats {
            total_unique_chatters: total_unique,
            repeater_count: repeater,
            new_chatter_count: new_chatter,
            repeater_percentage,
            avg_participation_rate: avg_participation,
        })
    })?;

    rows.next().unwrap_or_else(|| {
        Ok(ChatterBehaviorStats {
            total_unique_chatters: 0,
            repeater_count: 0,
            new_chatter_count: 0,
            repeater_percentage: 0.0,
            avg_participation_rate: 0.0,
        })
    })
}
