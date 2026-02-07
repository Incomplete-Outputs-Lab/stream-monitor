/// StreamStatsRepository - stream_statsテーブル専用レポジトリ
///
/// DuckDBのTIMESTAMP型（collected_at）を安全に扱い、
/// インターバル計算などの複雑なクエリを生成します。
use crate::database::analytics::DailyStats;
use crate::database::query_helpers::stream_stats_query;
use crate::database::utils;
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// インターバル付き統計データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsWithInterval {
    pub channel_name: String,
    pub stream_id: Option<i64>,
    pub viewer_count: i32,
    pub category: Option<String>,
    pub interval_minutes: f64,
    pub collected_at: String,
}

/// 時間バケット別視聴者統計
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucketViewerStats {
    pub bucket: String,
    pub avg_viewers: f64,
}

pub struct StreamStatsRepository;

impl StreamStatsRepository {
    /// インターバル計算付きで統計を取得
    ///
    /// LEAD関数を使用して次のレコードとの時間差を計算します。
    /// これはMW（Minutes Watched）計算の基礎となります。
    #[allow(dead_code)]
    pub fn get_stats_with_interval(
        conn: &Connection,
        channel_name: Option<&str>,
        stream_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<StatsWithInterval>, duckdb::Error> {
        let mut sql = format!(
            r#"
            SELECT 
                COALESCE(c1.channel_name, c2.channel_name, ss.channel_name) as channel_name,
                ss.stream_id,
                ss.viewer_count,
                ss.category,
                {},
                {}
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            LEFT JOIN channels c1 ON s.channel_id = c1.id
            LEFT JOIN channels c2 ON ss.channel_name = c2.channel_id AND c2.platform = 'twitch'
            WHERE ss.viewer_count IS NOT NULL
            "#,
            stream_stats_query::interval_with_fallback("ss"),
            stream_stats_query::collected_at_select("ss")
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(ch_name) = channel_name {
            sql.push_str(" AND ss.channel_name = ?");
            params.push(ch_name.to_string());
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

        sql.push_str(" ORDER BY ss.collected_at");

        let mut stmt = conn.prepare(&sql)?;
        let results = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StatsWithInterval {
                channel_name: row.get(0)?,
                stream_id: row.get(1)?,
                viewer_count: row.get(2)?,
                category: row.get(3)?,
                interval_minutes: row.get::<_, Option<f64>>(4)?.unwrap_or(1.0),
                collected_at: row.get(5)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// 時間バケット別で視聴者数を集計
    ///
    /// 指定された間隔で視聴者数を平均化します。
    pub fn get_time_bucketed_viewers(
        conn: &Connection,
        interval_minutes: i32,
        channel_name: Option<&str>,
        stream_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<TimeBucketViewerStats>, duckdb::Error> {
        let mut sql = format!(
            r#"
            SELECT 
                time_bucket(INTERVAL '{} minutes', ss.collected_at)::VARCHAR as bucket,
                AVG(ss.viewer_count) as avg_viewers
            FROM stream_stats ss
            WHERE ss.viewer_count IS NOT NULL
            "#,
            interval_minutes
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(ch_name) = channel_name {
            sql.push_str(" AND ss.channel_name = ?");
            params.push(ch_name.to_string());
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

        sql.push_str(" GROUP BY bucket ORDER BY bucket");

        let mut stmt = conn.prepare(&sql)?;
        let results = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(TimeBucketViewerStats {
                bucket: row.get(0)?,
                avg_viewers: row.get(1)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// チャンネル別日次統計を取得
    ///
    /// streamsテーブルと結合して配信時間も計算します。
    pub fn get_channel_daily_stats(
        conn: &Connection,
        channel_id: i64,
        start_time: &str,
        end_time: &str,
    ) -> Result<Vec<DailyStats>, duckdb::Error> {
        let sql = format!(
            r#"
            WITH channel_lookup AS (
                SELECT channel_name FROM channels WHERE id = ?
            ),
            stats_with_interval AS (
                SELECT 
                    DATE(ss.collected_at) as date,
                    ss.viewer_count,
                    ss.stream_id,
                    ss.channel_name,
                    {},
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
            "#,
            stream_stats_query::interval_with_fallback("ss")
        );

        let mut stmt = conn.prepare(&sql)?;
        let params = vec![
            channel_id.to_string(),
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

    /// ゲーム別日次統計を取得（game_idベース）
    pub fn get_game_daily_stats(
        conn: &Connection,
        game_id: &str,
        start_time: &str,
        end_time: &str,
    ) -> Result<Vec<DailyStats>, duckdb::Error> {
        let sql = format!(
            r#"
            WITH stats_with_interval AS (
                SELECT
                    DATE(ss.collected_at) as date,
                    ss.viewer_count,
                    ss.stream_id,
                    ss.channel_name,
                    {},
                    ss.collected_at
                FROM stream_stats ss
                WHERE ss.game_id = ?
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
                WHERE ss.game_id = ?
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
            "#,
            stream_stats_query::interval_with_fallback("ss")
        );

        let mut stmt = conn.prepare(&sql)?;
        let params = vec![
            game_id.to_string(),
            start_time.to_string(),
            end_time.to_string(),
            game_id.to_string(),
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

    /// データ可用性情報を取得
    pub fn get_data_availability(
        conn: &Connection,
    ) -> Result<(String, String, i64, i64), duckdb::Error> {
        // MIN(collected_at)
        let first_record: String = conn
            .query_row(
                "SELECT MIN(collected_at)::VARCHAR FROM stream_stats WHERE collected_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| String::new());

        // MAX(collected_at)
        let last_record: String = conn
            .query_row(
                "SELECT MAX(collected_at)::VARCHAR FROM stream_stats WHERE collected_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| String::new());

        // COUNT DISTINCT DATE
        let total_days_with_data: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT DATE(collected_at)) FROM stream_stats WHERE collected_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // COUNT(*)
        let total_records: i64 = conn
            .query_row("SELECT COUNT(*) FROM stream_stats", [], |row| row.get(0))
            .unwrap_or(0);

        Ok((
            first_record,
            last_record,
            total_days_with_data,
            total_records,
        ))
    }
}
