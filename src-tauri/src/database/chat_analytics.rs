use crate::database::repositories::ChatMessageRepository;
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
///
/// ChatMessageRepositoryとStreamStatsRepositoryを使用します。
pub fn get_chat_engagement_timeline(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    interval_minutes: i32,
) -> Result<Vec<ChatEngagementStats>, duckdb::Error> {
    use crate::database::repositories::StreamStatsRepository;

    // チャットバケットを取得
    let chat_buckets = ChatMessageRepository::count_by_time_bucket(
        conn,
        interval_minutes,
        channel_id,
        stream_id,
        start_time,
        end_time,
    )?;

    // channel_idからchannel_nameを取得（エラーハンドリングを改善）
    let channel_name = if let Some(ch_id) = channel_id {
        match conn.query_row(
            "SELECT channel_name FROM channels WHERE id = ?",
            [ch_id.to_string()],
            |row| row.get::<_, String>(0),
        ) {
            Ok(name) => {
                eprintln!(
                    "[Engagement Debug] Channel ID {} -> Channel Name: {}",
                    ch_id, name
                );
                Some(name)
            }
            Err(e) => {
                // チャンネルが見つからない場合は空の結果を返す
                eprintln!("[Engagement Debug] Channel ID {} not found: {:?}", ch_id, e);
                return Ok(vec![]);
            }
        }
    } else {
        eprintln!("[Engagement Debug] No channel_id provided, fetching all channels");
        None
    };

    // 視聴者統計を取得
    let viewer_buckets = StreamStatsRepository::get_time_bucketed_viewers(
        conn,
        interval_minutes,
        channel_name.as_deref(),
        stream_id,
        start_time,
        end_time,
    )?;

    // バケット単位でマージ
    let viewer_map: std::collections::HashMap<String, f64> = viewer_buckets
        .into_iter()
        .map(|v| (v.bucket, v.avg_viewers))
        .collect();

    let results = chat_buckets
        .into_iter()
        .map(|chat| {
            let viewer_count = viewer_map.get(&chat.bucket).copied().unwrap_or(0.0) as i32;
            let engagement_rate = if viewer_count > 0 {
                (chat.chat_count as f64 / viewer_count as f64) * 100.0
            } else {
                0.0
            };

            ChatEngagementStats {
                timestamp: chat.bucket,
                chat_count: chat.chat_count,
                unique_chatters: chat.unique_chatters,
                viewer_count,
                engagement_rate,
            }
        })
        .collect();

    Ok(results)
}

/// チャットスパイク（急増ポイント）を検出
///
/// ChatMessageRepositoryを使用してスパイクを検出します。
pub fn detect_chat_spikes(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    min_spike_ratio: f64,
) -> Result<Vec<ChatSpike>, duckdb::Error> {
    // 5分間隔でバケット取得
    let buckets = ChatMessageRepository::count_by_time_bucket(
        conn, 5, channel_id, stream_id, start_time, end_time,
    )?;

    // 前のバケットとの比較でスパイクを検出
    let mut spikes = Vec::new();
    let mut prev_count = 0i64;

    for bucket in buckets {
        if prev_count > 0 {
            let spike_ratio = bucket.chat_count as f64 / prev_count as f64;
            if spike_ratio >= min_spike_ratio {
                spikes.push(ChatSpike {
                    timestamp: bucket.bucket.clone(),
                    chat_count: bucket.chat_count,
                    spike_ratio,
                    prev_count,
                });
            }
        }
        prev_count = bucket.chat_count;
    }

    // spike_ratio降順でソート、上位20件
    spikes.sort_by(|a, b| b.spike_ratio.partial_cmp(&a.spike_ratio).unwrap());
    spikes.truncate(20);

    Ok(spikes)
}

/// ユーザーセグメント別統計を取得
///
/// ChatMessageRepositoryを使用して、badges直接SELECT問題を回避します。
pub fn get_user_segment_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<UserSegmentStats>, duckdb::Error> {
    // ChatMessageRepositoryを使用して安全にセグメント別統計を取得
    let segment_stats = ChatMessageRepository::count_by_user_segment(
        conn, channel_id, stream_id, start_time, end_time,
    )?;

    // パーセンテージと平均メッセージ数を計算
    let total_messages: i64 = segment_stats.iter().map(|s| s.message_count).sum();

    let results: Vec<UserSegmentStats> = segment_stats
        .into_iter()
        .map(|s| {
            let avg_messages_per_user = if s.user_count > 0 {
                s.message_count as f64 / s.user_count as f64
            } else {
                0.0
            };

            let percentage = if total_messages > 0 {
                (s.message_count as f64 / total_messages as f64) * 100.0
            } else {
                0.0
            };

            UserSegmentStats {
                segment: s.segment,
                message_count: s.message_count,
                user_count: s.user_count,
                avg_messages_per_user,
                percentage,
            }
        })
        .collect();

    Ok(results)
}

/// 上位チャッターを取得
///
/// ChatMessageRepositoryを使用して上位チャッターを取得します。
pub fn get_top_chatters(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    limit: i32,
) -> Result<Vec<TopChatter>, duckdb::Error> {
    let chatters = ChatMessageRepository::get_top_chatters(
        conn, channel_id, stream_id, start_time, end_time, limit,
    )?;

    // ChatterWithBadgesからTopChatterに変換
    let results = chatters
        .into_iter()
        .map(|c| TopChatter {
            user_name: c.user_name,
            message_count: c.message_count,
            badges: c.badges,
            first_seen: c.first_seen,
            last_seen: c.last_seen,
            stream_count: c.stream_count,
        })
        .collect();

    Ok(results)
}

pub fn get_time_pattern_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    group_by_day: bool,
) -> Result<Vec<TimePatternStats>, duckdb::Error> {
    let results = ChatMessageRepository::get_time_pattern_stats(
        conn,
        channel_id,
        start_time,
        end_time,
        group_by_day,
    )?;

    Ok(results
        .into_iter()
        .map(
            |(hour, day_of_week, avg_chat_rate, avg_engagement, total_messages)| TimePatternStats {
                hour,
                day_of_week,
                avg_chat_rate,
                avg_engagement,
                total_messages,
            },
        )
        .collect())
}

pub fn get_chatter_behavior_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<ChatterBehaviorStats, duckdb::Error> {
    let (total_unique, repeater, new_chatter, avg_participation) =
        ChatMessageRepository::get_chatter_behavior_stats(conn, channel_id, start_time, end_time)?;

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
}
