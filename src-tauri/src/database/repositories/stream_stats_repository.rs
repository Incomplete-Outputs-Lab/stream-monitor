/// StreamStatsRepository - stream_statsテーブル専用レポジトリ
///
/// DuckDBのTIMESTAMP型（collected_at）を安全に扱い、
/// インターバル計算などの複雑なクエリを生成します。
use crate::database::analytics::DailyStats;
use crate::database::models::StreamStats;
use crate::database::query_helpers::stream_stats_query;
use crate::database::utils;
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime};
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
    /// フィルタ付きで stream_stats を取得（get_stream_stats / export 共用）
    ///
    /// ORDER BY collected_at は order_asc で制御（true = ASC, false = DESC）
    pub fn get_stream_stats_filtered(
        conn: &Connection,
        stream_id: Option<i64>,
        channel_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        order_asc: bool,
    ) -> Result<Vec<StreamStats>, duckdb::Error> {
        let mut sql = String::from(
            "SELECT ss.id, ss.stream_id, CAST(ss.collected_at AS VARCHAR) as collected_at, ss.viewer_count,
             COALESCE((
                 SELECT COUNT(*)
                 FROM chat_messages cm
                 WHERE cm.stream_id = ss.stream_id
                   AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                   AND cm.timestamp < ss.collected_at
             ), 0) AS chat_rate_1min,
             ss.category, ss.title, ss.follower_count, ss.twitch_user_id, ss.channel_name
             FROM stream_stats ss
             INNER JOIN streams s ON ss.stream_id = s.id
             WHERE 1=1",
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(sid) = stream_id {
            sql.push_str(" AND ss.stream_id = ?");
            params.push(sid.to_string());
        }
        if let Some(cid) = channel_id {
            sql.push_str(" AND s.channel_id = ?");
            params.push(cid.to_string());
        }
        if let Some(st) = start_time {
            sql.push_str(" AND ss.collected_at >= ?");
            params.push(st.to_string());
        }
        if let Some(et) = end_time {
            sql.push_str(" AND ss.collected_at <= ?");
            params.push(et.to_string());
        }

        if order_asc {
            sql.push_str(" ORDER BY ss.collected_at ASC");
        } else {
            sql.push_str(" ORDER BY ss.collected_at DESC");
        }

        let mut stmt = conn.prepare(&sql)?;
        let results = utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok(StreamStats {
                id: Some(row.get(0)?),
                stream_id: row.get(1)?,
                collected_at: row.get(2)?,
                viewer_count: row.get(3)?,
                chat_rate_1min: Some(row.get(4)?),
                category: row.get(5)?,
                game_id: None,
                title: row.get(6)?,
                follower_count: row.get(7)?,
                twitch_user_id: row.get(8)?,
                channel_name: row.get(9)?,
            })
        })?;
        results.collect::<Result<Vec<_>, _>>()
    }

    /// 指定した時間範囲と間隔で線形補完した統計データを取得（エクスポート用）
    ///
    /// - 元データは get_stream_stats_filtered で取得
    /// - viewer_count / chat_rate_1min を線形補完
    /// - それ以外の文字列系は直前の値を採用
    pub fn get_interpolated_stream_stats_for_export(
        conn: &Connection,
        stream_id: Option<i64>,
        channel_id: Option<i64>,
        start_time: &str,
        end_time: &str,
        interval_minutes: i64,
    ) -> Result<Vec<StreamStats>, duckdb::Error> {
        // ベースとなる生データを取得（昇順）
        let base_stats = Self::get_stream_stats_filtered(
            conn,
            stream_id,
            channel_id,
            Some(start_time),
            Some(end_time),
            true,
        )?;

        if base_stats.is_empty() {
            return Ok(Vec::new());
        }

        // 補完の対象範囲は「実際にデータが存在する最初と最後の時刻」に限定する
        let effective_start = &base_stats
            .first()
            .map(|s| s.collected_at.as_str())
            .unwrap_or(start_time);
        let effective_end = &base_stats
            .last()
            .map(|s| s.collected_at.as_str())
            .unwrap_or(end_time);

        Ok(Self::interpolate_stats(
            &base_stats,
            effective_start,
            effective_end,
            interval_minutes,
        ))
    }

    /// 内部ヘルパー: 線形補完を行う
    fn interpolate_stats(
        stats: &[StreamStats],
        start_time: &str,
        end_time: &str,
        interval_minutes: i64,
    ) -> Vec<StreamStats> {
        fn parse_ts(s: &str) -> Option<DateTime<FixedOffset>> {
            // 1) RFC3339 (元の文字列形式を想定)
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Some(dt);
            }
            // 2) DuckDB の TIMESTAMP 表示形式を想定（秒以下あり/なし両対応）
            if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
            {
                let offset = FixedOffset::east_opt(0)?;
                return Some(DateTime::from_naive_utc_and_offset(naive, offset));
            }
            None
        }

        fn lerp_i32(a: Option<i32>, b: Option<i32>, frac: f64) -> Option<i32> {
            match (a, b) {
                (Some(av), Some(bv)) => {
                    let v = av as f64 + (bv as f64 - av as f64) * frac;
                    Some(v.round() as i32)
                }
                (Some(av), None) => Some(av),
                (None, Some(bv)) => Some(bv),
                (None, None) => None,
            }
        }
        fn lerp_i64(a: Option<i64>, b: Option<i64>, frac: f64) -> Option<i64> {
            match (a, b) {
                (Some(av), Some(bv)) => {
                    let v = av as f64 + (bv as f64 - av as f64) * frac;
                    Some(v.round() as i64)
                }
                (Some(av), None) => Some(av),
                (None, Some(bv)) => Some(bv),
                (None, None) => None,
            }
        }

        // 元データのタイムスタンプをパース
        let mut times: Vec<DateTime<FixedOffset>> = Vec::with_capacity(stats.len());
        for s in stats {
            if let Some(dt) = parse_ts(&s.collected_at) {
                times.push(dt);
            } else {
                // パースできない場合は補完せずそのまま返す
                return stats.to_vec();
            }
        }

        let n = stats.len();
        if n == 0 {
            return Vec::new();
        }

        // 線形補完の範囲は start_time〜end_time（呼び出し元で first/last にクリップ済み）
        let start_dt = parse_ts(start_time).unwrap_or(times[0]);
        let end_dt = parse_ts(end_time).unwrap_or(*times.last().unwrap());

        let step = Duration::minutes(interval_minutes.max(1));
        let mut grid: Vec<DateTime<FixedOffset>> = Vec::new();
        let mut t = start_dt;
        while t <= end_dt {
            grid.push(t);
            t += step;
        }

        let mut result: Vec<StreamStats> = Vec::with_capacity(grid.len());

        for t in grid {
            // 前後のサンプルを探す
            let mut j = 0usize;
            while j < n && times[j] < t {
                j += 1;
            }

            let prev_idx = if j == 0 { 0 } else { j - 1 };
            let next_idx = if j >= n { n - 1 } else { j };

            let prev_t = times[prev_idx];
            let next_t = times[next_idx];

            let prev = &stats[prev_idx];
            let next = &stats[next_idx];

            let frac = if next_t <= prev_t {
                0.0
            } else {
                let total = (next_t - prev_t).num_seconds() as f64;
                let elapsed = (t - prev_t).num_seconds() as f64;
                (elapsed / total).clamp(0.0, 1.0)
            };

            let viewer = lerp_i32(prev.viewer_count, next.viewer_count, frac);
            let chat = lerp_i64(prev.chat_rate_1min, next.chat_rate_1min, frac);

            let base = prev; // カテゴリ等は直前の値を使用

            result.push(StreamStats {
                id: None,
                stream_id: base.stream_id,
                collected_at: t.to_rfc3339(),
                viewer_count: viewer,
                chat_rate_1min: chat,
                category: base.category.clone(),
                game_id: base.game_id.clone(),
                title: base.title.clone(),
                follower_count: base.follower_count,
                twitch_user_id: base.twitch_user_id.clone(),
                channel_name: base.channel_name.clone(),
            });
        }

        result
    }

    /// 自動発見された配信の統計データを挿入
    ///
    /// stream_idがNULLの状態で、自動発見時の統計データを記録します。
    /// game_id を保存することで、ゲーム分析・トップゲーム集計に自動発見チャンネルも含まれる。
    pub fn insert_auto_discovery_stats(
        conn: &Connection,
        collected_at: &str,
        viewer_count: i32,
        twitch_user_id: &str,
        channel_name: &str,
        category: &str,
        game_id: &str,
    ) -> Result<(), duckdb::Error> {
        conn.execute(
            r#"
            INSERT INTO stream_stats (
                stream_id, collected_at, viewer_count,
                twitch_user_id, channel_name, category, game_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            duckdb::params![
                None::<i64>, // stream_id = NULL
                collected_at,
                viewer_count,
                twitch_user_id,
                channel_name,
                category,
                game_id,
            ],
        )?;
        Ok(())
    }

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
