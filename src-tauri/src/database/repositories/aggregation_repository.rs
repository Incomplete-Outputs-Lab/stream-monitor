/// AggregationRepository - 複雑な統計集計用レポジトリ
///
/// MW（Minutes Watched）計算、配信者別/ゲーム別統計など、
/// 複数のCTEを使用する複雑な集計クエリを提供します。
use crate::database::analytics::{BroadcasterAnalytics, GameAnalytics};
use crate::database::query_helpers::stream_stats_query;
use crate::database::utils;
use duckdb::Connection;

pub struct AggregationRepository;

impl AggregationRepository {
    /// 配信者別統計を計算
    ///
    /// MW、Hours Broadcasted、Peak CCU、チャット統計などを一度に計算します。
    pub fn calculate_broadcaster_analytics(
        conn: &Connection,
        channel_id: Option<i64>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<BroadcasterAnalytics>, duckdb::Error> {
        // channel_id が指定されている場合、channel_name を取得
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

        let mut sql = format!(
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
                    {}
                FROM stream_stats ss
                LEFT JOIN streams s ON ss.stream_id = s.id
                LEFT JOIN channels c1 ON s.channel_id = c1.id
                LEFT JOIN channels c2 ON ss.channel_name = c2.channel_id AND c2.platform = 'twitch'
                WHERE 1=1
            "#,
            stream_stats_query::interval_with_fallback("ss")
        );

        let mut params: Vec<String> = Vec::new();

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
                COALESCE(c.channel_name, cs.channel_name) as channel_name,
                cs.channel_name as login_name,
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
                row.get::<_, String>(2)?,          // login_name
                row.get::<_, i64>(3)?,             // minutes_watched
                row.get::<_, f64>(4)?,             // hours_broadcasted
                row.get::<_, f64>(5)?,             // average_ccu
                row.get::<_, i32>(6)?,             // peak_ccu
                row.get::<_, i32>(7)?,             // stream_count
                row.get::<_, i64>(8)?,             // total_chat_messages
                row.get::<_, f64>(9)?,             // avg_chat_rate
                row.get::<_, i32>(10)?,            // category_count
                row.get::<_, Option<String>>(11)?, // main_title
                row.get::<_, Option<i64>>(12)?,    // main_mw
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
            login_name,
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

            let engagement_rate = if mw > 0 {
                (total_chat as f64 / mw as f64) * 1000.0
            } else {
                0.0
            };

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
                channel_name: ch_name,
                login_name,
                minutes_watched: mw,
                hours_broadcasted: hours,
                average_ccu: avg_ccu,
                main_played_title: main_title,
                main_title_mw_percent: main_mw_percent,
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

    /// ゲーム別統計を計算（game_idベース）
    pub fn calculate_game_analytics(
        conn: &Connection,
        game_id: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<GameAnalytics>, duckdb::Error> {
        let mut sql = format!(
            r#"
            WITH stats_with_interval AS (
                SELECT
                    COALESCE(s.channel_id, c2.id) as channel_id,
                    COALESCE(c1.channel_name, c2.channel_name, ss.channel_name) as channel_name,
                    ss.game_id,
                    ss.viewer_count,
                    ss.collected_at,
                    ss.stream_id,
                    COALESCE((
                        SELECT COUNT(*)
                        FROM chat_messages cm
                        WHERE cm.stream_id = ss.stream_id
                          AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                          AND cm.timestamp < ss.collected_at
                    ), 0) AS chat_rate_1min,
                    {}
                FROM stream_stats ss
                LEFT JOIN streams s ON ss.stream_id = s.id
                LEFT JOIN channels c1 ON s.channel_id = c1.id
                LEFT JOIN channels c2 ON ss.channel_name = c2.channel_id AND c2.platform = 'twitch'
                WHERE ss.game_id IS NOT NULL
            "#,
            stream_stats_query::interval_with_fallback("ss")
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(gid) = game_id {
            sql.push_str(" AND ss.game_id = ?");
            params.push(gid.to_string());
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
                    game_id,
                    COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS minutes_watched,
                    COALESCE(SUM(COALESCE(interval_minutes, 1)) / 60.0, 0) AS hours_broadcasted,
                    COALESCE(AVG(viewer_count), 0) AS average_ccu,
                    COUNT(DISTINCT channel_name) AS unique_broadcasters,
                    COALESCE(SUM(chat_rate_1min * COALESCE(interval_minutes, 1)), 0)::BIGINT AS total_chat_messages,
                    COALESCE(AVG(chat_rate_1min), 0) AS avg_chat_rate
                FROM stats_with_interval
                WHERE viewer_count IS NOT NULL
                    AND channel_name IS NOT NULL
                GROUP BY game_id
            ),
            channel_by_category AS (
                SELECT
                    game_id,
                    channel_name,
                    COALESCE(SUM(viewer_count * COALESCE(interval_minutes, 1)), 0)::BIGINT AS channel_mw,
                    ROW_NUMBER() OVER (PARTITION BY game_id ORDER BY SUM(viewer_count * COALESCE(interval_minutes, 1)) DESC) as rn
                FROM stats_with_interval
                WHERE viewer_count IS NOT NULL
                    AND channel_name IS NOT NULL
                GROUP BY game_id, channel_name
            ),
            top_channels AS (
                SELECT
                    game_id,
                    channel_name as top_channel_login
                FROM channel_by_category
                WHERE rn = 1
            )
            SELECT
                gs.game_id,
                COALESCE(gc.game_name, 'Unknown') as category,
                gs.minutes_watched,
                gs.hours_broadcasted,
                gs.average_ccu,
                gs.unique_broadcasters,
                COALESCE(c.channel_name, tc.top_channel_login) as top_channel,
                tc.top_channel_login,
                gs.total_chat_messages,
                gs.avg_chat_rate,
                CASE
                    WHEN gs.minutes_watched > 0
                    THEN (gs.total_chat_messages::DOUBLE / gs.minutes_watched::DOUBLE) * 1000.0
                    ELSE 0.0
                END as engagement_rate
            FROM game_stats gs
            LEFT JOIN top_channels tc ON gs.game_id = tc.game_id
            LEFT JOIN channels c ON (tc.top_channel_login = c.channel_id AND c.platform = 'twitch')
            LEFT JOIN game_categories gc ON gs.game_id = gc.game_id
            ORDER BY gs.minutes_watched DESC
            "#,
        );

        let mut stmt = conn.prepare(&sql)?;
        let results: Vec<GameAnalytics> =
            utils::query_map_with_params(&mut stmt, &params, |row| {
                Ok(GameAnalytics {
                    game_id: row.get::<_, Option<String>>(0)?,
                    category: row.get::<_, String>(1)?,
                    minutes_watched: row.get::<_, i64>(2)?,
                    hours_broadcasted: row.get::<_, f64>(3)?,
                    average_ccu: row.get::<_, f64>(4)?,
                    unique_broadcasters: row.get::<_, i32>(5)?,
                    top_channel: row.get::<_, Option<String>>(6)?,
                    top_channel_login: row.get::<_, Option<String>>(7)?,
                    total_chat_messages: row.get::<_, i64>(8)?,
                    avg_chat_rate: row.get::<_, f64>(9)?,
                    engagement_rate: row.get::<_, f64>(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// カテゴリ一覧を取得（MW降順）
    pub fn list_categories(
        conn: &Connection,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Result<Vec<String>, duckdb::Error> {
        let mut sql = format!(
            r#"
            WITH stats_with_interval AS (
                SELECT 
                    ss.category,
                    ss.viewer_count,
                    {}
                FROM stream_stats ss
                WHERE ss.category IS NOT NULL
            "#,
            stream_stats_query::interval_with_fallback("ss")
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
}
