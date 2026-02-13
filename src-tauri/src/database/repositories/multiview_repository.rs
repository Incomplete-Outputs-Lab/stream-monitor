/// MultiviewRepository - マルチビュー用リアルタイム統計・イベント検知
///
/// 複数チャンネルのリアルタイム指標（視聴者数、チャットレート）と
/// イベントフラグ（スパイク、カテゴリ変更）を取得します。
use chrono::{Duration, Local};
use duckdb::Connection;
use serde::{Deserialize, Serialize};

/// マルチビュー用1チャンネルあたりのリアルタイム統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiviewChannelStats {
    pub channel_id: i64,
    pub channel_name: String,
    pub stream_id: Option<i64>,
    pub is_live: bool,
    pub viewer_count: Option<i32>,
    pub chat_rate_1min: i64,
    pub chat_rate_5s: i64,
    pub category: Option<String>,
    pub title: Option<String>,
    pub collected_at: Option<String>,
    pub event_flags: MultiviewEventFlags,
}

/// イベント検知フラグ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiviewEventFlags {
    pub viewer_spike: bool,
    pub chat_spike: bool,
    pub category_change: bool,
}

/// スパイク検出の閾値（マルチビュー設定）
///
/// チューニング方針:
///
/// - VIEWER_SPIKE_RATIO: 直近2-10分平均に対する倍率。1.5 = 50%以上増加でスパイク
/// - VIEWER_SPIKE_MIN_DELTA: 絶対増加分。小規模配信の誤検知を防ぐ
/// - CHAT_SPIKE_RATIO: チャットレートのベースラインに対する倍率
///
/// 将来的に config テーブルや JSON 設定で上書き可能にする余地あり。
const VIEWER_SPIKE_RATIO: f64 = 1.5; // 50%以上増加
const VIEWER_SPIKE_MIN_DELTA: i32 = 100; // 絶対値で100以上
const CHAT_SPIKE_RATIO: f64 = 2.0; // 2倍以上

pub struct MultiviewRepository;

impl MultiviewRepository {
    /// 指定チャンネル群のマルチビュー用リアルタイム統計を取得
    ///
    /// - 有効なstream_statsがあるチャンネルのみ返却
    /// - イベント検知は直近データとの比較で実施
    pub fn get_realtime_stats(
        conn: &Connection,
        channel_ids: &[i64],
    ) -> Result<Vec<MultiviewChannelStats>, duckdb::Error> {
        if channel_ids.is_empty() {
            return Ok(Vec::new());
        }

        let now = Local::now();
        let one_min_ago = (now - Duration::minutes(1)).to_rfc3339();
        let five_sec_ago = (now - Duration::seconds(5)).to_rfc3339();
        let two_min_ago = (now - Duration::minutes(2)).to_rfc3339();
        let ten_min_ago = (now - Duration::minutes(10)).to_rfc3339();

        // channel_ids はDB内部ID（信頼できる値）のため、IN句に直接展開
        let channel_filter: Vec<String> = channel_ids.iter().map(|id| id.to_string()).collect();
        let channel_filter = channel_filter.join(", ");

        // 各チャンネルの最新 stream_stats を取得（stream が存在するもののみ）
        // stream_stats.channel_name は Twitch login、channels.channel_id も login
        let sql_latest = format!(
            r#"
            WITH latest_stats AS (
                SELECT DISTINCT ON (s.channel_id)
                    c.id as channel_id,
                    c.channel_id as channel_login,
                    s.id as stream_id,
                    ss.viewer_count,
                    ss.category,
                    ss.title,
                    ss.collected_at,
                    ss.channel_name
                FROM channels c
                INNER JOIN streams s ON s.channel_id = c.id AND s.ended_at IS NULL
                INNER JOIN stream_stats ss ON ss.stream_id = s.id
                WHERE c.id IN ({})
                ORDER BY s.channel_id, ss.collected_at DESC
            )
            SELECT
                ls.channel_id,
                ls.channel_login,
                ls.stream_id,
                ls.viewer_count,
                ls.category,
                ls.title,
                CAST(ls.collected_at AS VARCHAR) as collected_at,
                ls.channel_name
            FROM latest_stats ls
            "#,
            channel_filter
        );

        let mut stmt = conn.prepare(&sql_latest)?;
        let row_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i32>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
            ))
        })?;

        let mut results: Vec<MultiviewChannelStats> = Vec::new();

        for row_result in row_iter {
            let (channel_id, stream_id, viewer_count, category, title, collected_at, channel_name) =
                row_result?;

            // 直近1分のチャット数
            let chat_rate_1min = Self::count_chat_in_window(
                conn,
                Some(stream_id),
                None,
                &one_min_ago,
                &now.to_rfc3339(),
            )?;

            // 直近5秒のチャット数
            let chat_rate_5s = Self::count_chat_in_window(
                conn,
                Some(stream_id),
                None,
                &five_sec_ago,
                &now.to_rfc3339(),
            )?;

            // イベント検知
            let event_flags = Self::detect_events(
                conn,
                channel_id,
                stream_id,
                viewer_count,
                chat_rate_1min,
                chat_rate_5s,
                category.as_deref(),
                &ten_min_ago,
                &two_min_ago,
                &now.to_rfc3339(),
            )?;

            results.push(MultiviewChannelStats {
                channel_id,
                channel_name,
                stream_id: Some(stream_id),
                is_live: true,
                viewer_count,
                chat_rate_1min,
                chat_rate_5s,
                category,
                title,
                collected_at,
                event_flags,
            });
        }

        // 指定されたがライブでないチャンネルは is_live=false で追加
        for &cid in channel_ids {
            if !results.iter().any(|r| r.channel_id == cid) {
                let channel_name = Self::get_channel_login_by_id(conn, cid)?;
                results.push(MultiviewChannelStats {
                    channel_id: cid,
                    channel_name: channel_name.unwrap_or_else(|| cid.to_string()),
                    stream_id: None,
                    is_live: false,
                    viewer_count: None,
                    chat_rate_1min: 0,
                    chat_rate_5s: 0,
                    category: None,
                    title: None,
                    collected_at: None,
                    event_flags: MultiviewEventFlags::default(),
                });
            }
        }

        // channel_ids の順序でソート
        results.sort_by_key(|r| {
            channel_ids
                .iter()
                .position(|&id| id == r.channel_id)
                .unwrap_or(usize::MAX)
        });

        Ok(results)
    }

    fn count_chat_in_window(
        conn: &Connection,
        stream_id: Option<i64>,
        _channel_id: Option<i64>,
        start: &str,
        end: &str,
    ) -> Result<i64, duckdb::Error> {
        let Some(sid) = stream_id else {
            return Ok(0);
        };
        let sql = r#"
            SELECT COUNT(*) FROM chat_messages
            WHERE stream_id = ? AND timestamp >= ? AND timestamp <= ?
        "#;
        conn.query_row(sql, duckdb::params![sid, start, end], |row| row.get(0))
            .or_else(|_| Ok(0i64))
    }

    fn get_channel_login_by_id(
        conn: &Connection,
        channel_id: i64,
    ) -> Result<Option<String>, duckdb::Error> {
        let mut stmt = conn.prepare("SELECT channel_id FROM channels WHERE id = ?")?;
        let mut rows =
            stmt.query_map(duckdb::params![channel_id], |row| row.get::<_, String>(0))?;
        rows.next().transpose()
    }

    #[allow(clippy::too_many_arguments)]
    fn detect_events(
        conn: &Connection,
        _channel_id: i64,
        stream_id: i64,
        current_viewers: Option<i32>,
        chat_rate_1min: i64,
        _chat_rate_5s: i64,
        current_category: Option<&str>,
        baseline_start: &str,
        baseline_end: &str,
        _now: &str,
    ) -> Result<MultiviewEventFlags, duckdb::Error> {
        let mut flags = MultiviewEventFlags::default();

        let viewer_spike = if let Some(v) = current_viewers {
            Self::detect_viewer_spike(conn, stream_id, v, baseline_start, baseline_end)?
        } else {
            false
        };

        let chat_spike = Self::detect_chat_spike(
            conn,
            stream_id,
            chat_rate_1min,
            baseline_start,
            baseline_end,
        )?;

        let category_change = Self::detect_category_change(conn, stream_id, current_category)?;

        flags.viewer_spike = viewer_spike;
        flags.chat_spike = chat_spike;
        flags.category_change = category_change;

        Ok(flags)
    }

    fn detect_viewer_spike(
        conn: &Connection,
        stream_id: i64,
        current: i32,
        baseline_start: &str,
        baseline_end: &str,
    ) -> Result<bool, duckdb::Error> {
        // ベースライン: 直近10分〜2分前の平均（直近を除く）
        let avg_sql = r#"
            SELECT AVG(viewer_count)::DOUBLE
            FROM stream_stats
            WHERE stream_id = ? AND collected_at >= ? AND collected_at <= ?
        "#;
        let avg: Option<f64> = conn
            .query_row(
                avg_sql,
                duckdb::params![stream_id, baseline_start, baseline_end,],
                |row| row.get(0),
            )
            .ok();

        let Some(baseline) = avg else {
            return Ok(false);
        };
        if baseline <= 0.0 {
            return Ok(false);
        }

        let ratio = current as f64 / baseline;
        let delta = current - baseline as i32;
        Ok(ratio >= VIEWER_SPIKE_RATIO && delta >= VIEWER_SPIKE_MIN_DELTA)
    }

    fn detect_chat_spike(
        conn: &Connection,
        stream_id: i64,
        current: i64,
        baseline_start: &str,
        baseline_end: &str,
    ) -> Result<bool, duckdb::Error> {
        // ベースライン: 直近10分〜2分前の chat_rate 平均
        let sql = r#"
            SELECT AVG(
                COALESCE((
                    SELECT COUNT(*)
                    FROM chat_messages cm
                    WHERE cm.stream_id = ss.stream_id
                      AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                      AND cm.timestamp < ss.collected_at
                ), 0)
            )::DOUBLE
            FROM stream_stats ss
            WHERE ss.stream_id = ? AND ss.collected_at >= ? AND ss.collected_at <= ?
        "#;
        let avg: Option<f64> = conn
            .query_row(
                sql,
                duckdb::params![stream_id, baseline_start, baseline_end,],
                |row| row.get(0),
            )
            .ok();

        let Some(baseline) = avg else {
            return Ok(false);
        };
        if baseline < 1.0 {
            return Ok(current >= 2); // ベースラインがほぼ0なら、2件以上でスパイクとみなす
        }

        let ratio = current as f64 / baseline;
        Ok(ratio >= CHAT_SPIKE_RATIO)
    }

    fn detect_category_change(
        conn: &Connection,
        stream_id: i64,
        _current_category: Option<&str>,
    ) -> Result<bool, duckdb::Error> {
        let sql = r#"
            SELECT category
            FROM stream_stats
            WHERE stream_id = ?
            ORDER BY collected_at DESC
            LIMIT 2
        "#;
        let mut stmt = conn.prepare(sql)?;
        let categories: Vec<Option<String>> = stmt
            .query_map(duckdb::params![stream_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if let (Some(Some(p)), Some(Some(c))) = (categories.get(1), categories.first()) {
            if p != c {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
