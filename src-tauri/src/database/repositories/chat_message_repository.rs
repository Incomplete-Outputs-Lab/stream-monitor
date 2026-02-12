/// ChatMessageRepository - chat_messagesテーブル専用レポジトリ
///
/// DuckDBのLIST型（badges）とTIMESTAMP型（timestamp）を安全に扱います。
use crate::database::query_helpers::chat_query;
use crate::database::utils;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// 時間バケット別チャット統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketChatStats {
    pub bucket: String,
    pub chat_count: i64,
    pub unique_chatters: i64,
}

/// ユーザーセグメント別統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSegmentStats {
    pub segment: String,
    pub message_count: i64,
    pub user_count: i64,
}

/// チャッター情報（バッジ付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatterWithBadges {
    pub user_id: Option<String>,      // Twitch user_id（プライマリ識別子）
    pub user_name: String,            // Twitchログイン名
    pub display_name: Option<String>, // Twitch表示名
    pub message_count: i64,
    pub badges: Vec<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub stream_count: i64,
}

/// 時間パターン統計の戻り値型（hour, day_of_week, avg_messages, stddev_messages, total_count）
pub type TimePatternStats = (i32, Option<i32>, f64, f64, i64);

pub struct ChatMessageRepository;

impl ChatMessageRepository {
    /// 時間バケット別でチャット数を集計
    ///
    /// # Arguments
    /// * `conn` - データベース接続
    /// * `interval_minutes` - バケット間隔（分）
    /// * `channel_id` - フィルター用チャンネルID（Optional）
    /// * `stream_id` - フィルター用配信ID（Optional）
    /// * `start_time` - 開始時刻（Optional）
    /// * `end_time` - 終了時刻（Optional）
    pub fn count_by_time_bucket(
        conn: &Connection,
        interval_minutes: i32,
        channel_id: Option<i64>,
        stream_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<TimeBucketChatStats>, duckdb::Error> {
        let mut sql = format!(
            r#"
            SELECT
                time_bucket(INTERVAL '{} minutes', cm.timestamp)::VARCHAR as bucket,
                COUNT(*) as chat_count,
                COUNT(DISTINCT cm.user_id) as unique_chatters
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
            "#,
            interval_minutes
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
        }

        if let Some(st_id) = stream_id {
            sql.push_str(&format!(" AND cm.stream_id = {}", st_id));
        }

        if let Some(start) = start_time {
            sql.push_str(" AND cm.timestamp >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_time {
            sql.push_str(" AND cm.timestamp <= ?");
            params.push(end.to_string());
        }

        sql.push_str(" GROUP BY bucket ORDER BY bucket");

        let mut stmt = conn.prepare(&sql)?;
        let results = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(TimeBucketChatStats {
                bucket: row.get(0)?,
                chat_count: row.get(1)?,
                unique_chatters: row.get(2)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// ユーザーセグメント別でメッセージ数を集計
    ///
    /// バッジ情報を使用してユーザーをセグメント分けします。
    /// badges直接参照問題を解決するため、list_contains()を使用します。
    pub fn count_by_user_segment(
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
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
        }

        if let Some(st_id) = stream_id {
            sql.push_str(&format!(" AND cm.stream_id = {}", st_id));
        }

        if let Some(start) = start_time {
            sql.push_str(" AND cm.timestamp >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_time {
            sql.push_str(" AND cm.timestamp <= ?");
            params.push(end.to_string());
        }

        // 重要: badges を直接 SELECT せず、list_contains() で判定
        sql.push_str(
            r#"
                GROUP BY cm.user_id, cm.badges
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
            )
            SELECT
                segment,
                SUM(message_count) as total_messages,
                COUNT(*) as user_count
            FROM classified_messages
            GROUP BY segment
            ORDER BY total_messages DESC
            "#,
        );

        let mut stmt = conn.prepare(&sql)?;
        let results = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(UserSegmentStats {
                segment: row.get(0)?,
                message_count: row.get(1)?,
                user_count: row.get(2)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// 上位チャッターを取得
    ///
    /// N+1クエリを避けるため、CTEでバッジを事前取得します。
    pub fn get_top_chatters(
        conn: &Connection,
        channel_id: Option<i64>,
        stream_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        limit: i32,
    ) -> Result<Vec<ChatterWithBadges>, duckdb::Error> {
        let mut sql = format!(
            r#"
            WITH user_badges AS (
                SELECT
                    user_id,
                    {},
                    ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY timestamp DESC) as rn
                FROM chat_messages
                WHERE badges IS NOT NULL
            "#,
            chat_query::badges_select("chat_messages")
        );

        let mut params: Vec<String> = Vec::new();

        // CTEにも同じフィルタを適用
        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(" AND channel_id = {}", ch_id));
        }

        if let Some(st_id) = stream_id {
            sql.push_str(&format!(" AND stream_id = {}", st_id));
        }

        if let Some(start) = start_time {
            sql.push_str(" AND timestamp >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_time {
            sql.push_str(" AND timestamp <= ?");
            params.push(end.to_string());
        }
        sql.push_str(
            r#"
            )
            SELECT
                cm.user_id,
                cm.user_name,
                cm.display_name,
                COUNT(*) as message_count,
                MIN(cm.timestamp)::VARCHAR as first_seen,
                MAX(cm.timestamp)::VARCHAR as last_seen,
                COUNT(DISTINCT cm.stream_id) as stream_count,
                ub.badges
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            LEFT JOIN user_badges ub ON cm.user_id = ub.user_id AND ub.rn = 1
            WHERE 1=1
            "#,
        );

        // メインクエリのWHERE句
        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
        }

        if let Some(st_id) = stream_id {
            sql.push_str(&format!(" AND cm.stream_id = {}", st_id));
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
            GROUP BY cm.user_id, cm.user_name, cm.display_name, ub.badges
            ORDER BY message_count DESC
            LIMIT ?
            "#,
        );
        params.push(limit.to_string());

        let mut stmt = conn.prepare(&sql)?;
        let results: Vec<ChatterWithBadges> =
            utils::query_map_with_params(&mut stmt, &params, |row| {
                let badges_str: Option<String> = row.get(7).ok();
                let badges = if let Some(b) = badges_str {
                    utils::parse_badges(&b).unwrap_or_default()
                } else {
                    Vec::new()
                };

                Ok(ChatterWithBadges {
                    user_id: row.get(0).ok(),
                    user_name: row.get(1)?,
                    display_name: row.get(2).ok(),
                    message_count: row.get(3)?,
                    badges,
                    first_seen: row.get(4)?,
                    last_seen: row.get(5)?,
                    stream_count: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// 指定ユーザーのバッジ情報を取得（user_idで検索）
    #[allow(dead_code)]
    pub fn get_user_badges(
        conn: &Connection,
        user_id: &str,
        channel_id: Option<i64>,
    ) -> Result<Option<Vec<String>>, duckdb::Error> {
        let mut sql = format!(
            r#"
            SELECT {}
            FROM chat_messages cm
            WHERE cm.user_id = ?
                AND cm.badges IS NOT NULL
            "#,
            chat_query::badges_select("cm")
        );

        let params = vec![user_id.to_string()];

        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(" AND cm.channel_id = {}", ch_id));
        }

        sql.push_str(" ORDER BY cm.timestamp DESC LIMIT 1");

        let mut stmt = conn.prepare(&sql)?;
        let result = utils::query_map_with_params(&mut stmt, &params, |row| {
            let badges_str: String = row.get(0)?;
            Ok(utils::parse_badges(&badges_str).unwrap_or_default())
        })?
        .next();

        match result {
            Some(Ok(badges)) => Ok(Some(badges)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    /// チャットメッセージ数をカウント
    #[allow(dead_code)]
    pub fn count_messages(
        conn: &Connection,
        channel_id: Option<i64>,
        stream_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<i64, duckdb::Error> {
        let mut sql = String::from(
            r#"
            SELECT COUNT(*)
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
            "#,
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
        }

        if let Some(st_id) = stream_id {
            sql.push_str(&format!(" AND cm.stream_id = {}", st_id));
        }

        if let Some(start) = start_time {
            sql.push_str(" AND cm.timestamp >= ?");
            params.push(start.to_string());
        }

        if let Some(end) = end_time {
            sql.push_str(" AND cm.timestamp <= ?");
            params.push(end.to_string());
        }

        let mut stmt = conn.prepare(&sql)?;
        let mut rows =
            utils::query_map_with_params(&mut stmt, &params, |row| row.get::<_, i64>(0))?;

        rows.next().unwrap_or(Ok(0))
    }

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
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
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
                    "SELECT channel_name FROM channels WHERE id = ?",
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

            Ok((
                hour,
                day_of_week,
                row.get(col_idx)?,
                row.get(col_idx + 1)?,
                row.get(col_idx + 2)?,
            ))
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    pub fn get_chatter_behavior_stats(
        conn: &Connection,
        channel_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<(i64, i64, i64, f64), duckdb::Error> {
        let mut sql = String::from(
            r#"
            WITH chatter_streams AS (
                SELECT
                    cm.user_id,
                    COUNT(DISTINCT cm.stream_id) as stream_count,
                    COUNT(*) as message_count
                FROM chat_messages cm
                LEFT JOIN streams s ON cm.stream_id = s.id
                WHERE cm.stream_id IS NOT NULL
            "#,
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(ch_id) = channel_id {
            sql.push_str(&format!(
                " AND (cm.channel_id = {} OR s.channel_id = {})",
                ch_id, ch_id
            ));
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
                GROUP BY cm.user_id
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
                    "SELECT channel_name FROM channels WHERE id = ?",
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
                COUNT(DISTINCT cs.user_id) as total_unique_chatters,
                SUM(CASE WHEN cs.stream_count > 1 THEN 1 ELSE 0 END) as repeater_count,
                SUM(CASE WHEN cs.stream_count = 1 THEN 1 ELSE 0 END) as new_chatter_count,
                AVG(CASE WHEN sv.peak_ccu > 0 THEN cs.message_count::FLOAT / sv.peak_ccu::FLOAT ELSE 0 END) * 100.0 as avg_participation_rate
            FROM chatter_streams cs
            CROSS JOIN (SELECT AVG(peak_ccu) as peak_ccu FROM stream_viewers) sv
            "#,
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;

        rows.next().unwrap_or(Ok((0, 0, 0, 0.0)))
    }

    /// 直近1分間の全チャンネルの合計チャットメッセージ数を取得
    pub fn get_realtime_chat_rate(conn: &Connection) -> Result<i64, duckdb::Error> {
        // ローカル時刻で1分前を計算（chat_messagesのtimestampはLocal::now()で保存されているため）
        let now = chrono::Local::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let one_minute_ago_str = one_minute_ago.to_rfc3339();

        let sql = "
            SELECT COUNT(*) as chat_count
            FROM chat_messages
            WHERE timestamp >= ?
        ";

        conn.query_row(sql, [&one_minute_ago_str], |row| row.get(0))
    }
}
