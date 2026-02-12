use crate::database::{query_helpers::chat_query, utils};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Phase 1: Text Analysis
// ============================================================================

/// Word frequency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordFrequencyResult {
    pub words: Vec<WordFrequency>,
    pub total_words: i64,
    pub unique_words: i64,
    pub avg_words_per_message: f64,
    pub total_messages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordFrequency {
    pub word: String,
    pub count: i64,
    pub percentage: f64,
}

/// Emote analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmoteAnalysisResult {
    pub emotes: Vec<EmoteUsage>,
    pub total_emote_uses: i64,
    pub emote_per_message_rate: f64,
    pub hourly_pattern: Vec<HourlyEmotePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmoteUsage {
    pub name: String,
    pub count: i64,
    pub users: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyEmotePattern {
    pub hour: i32,
    pub count: i64,
}

/// Message length statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageLengthStats {
    pub avg_length: f64,
    pub median_length: f64,
    pub std_dev: f64,
    pub min_length: i32,
    pub max_length: i32,
    pub distribution: Vec<LengthDistribution>,
    pub by_segment: Vec<SegmentLengthStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LengthDistribution {
    pub bucket: String, // "0-10", "11-20", etc.
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentLengthStats {
    pub segment: String,
    pub avg_length: f64,
    pub message_count: i64,
}

// ============================================================================
// Phase 2: Correlation Analysis
// ============================================================================

/// Correlation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrelationResult {
    pub pearson_coefficient: f64,
    pub interpretation: String,
    pub scatter_data: Vec<ScatterPoint>,
    pub hourly_correlation: Vec<HourlyCorrelation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterPoint {
    pub viewers: i32,
    pub chats: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyCorrelation {
    pub hour: i32,
    pub correlation: f64,
    pub sample_count: i64,
}

/// Category change impact result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryImpactResult {
    pub changes: Vec<CategoryChange>,
    pub category_performance: Vec<CategoryPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryChange {
    pub timestamp: String,
    pub from_category: String,
    pub to_category: String,
    pub before_viewers: i32,
    pub after_viewers: i32,
    pub viewer_change_percent: f64,
    pub chat_change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryPerformance {
    pub category: String,
    pub avg_viewers: f64,
    pub avg_chat_rate: f64,
    pub total_time_minutes: f64,
    pub change_count: i64,
}

// ============================================================================
// Phase 3: User Behavior Analysis
// ============================================================================

/// Chatter activity score result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatterScoreResult {
    pub scores: Vec<ChatterActivityScore>,
    pub score_distribution: Vec<ScoreDistribution>,
    pub segment_avg_scores: Vec<SegmentAvgScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatterActivityScore {
    pub user_name: String,
    pub score: f64,
    pub message_count: i64,
    pub stream_count: i64,
    pub badges: Vec<String>,
    pub rank: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDistribution {
    pub score_range: String, // "0-10", "11-20", etc.
    pub user_count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentAvgScore {
    pub segment: String,
    pub avg_score: f64,
    pub user_count: i64,
}

// ============================================================================
// Phase 4: Anomaly Detection
// ============================================================================

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyResult {
    pub viewer_anomalies: Vec<Anomaly>,
    pub chat_anomalies: Vec<Anomaly>,
    pub trend_stats: TrendStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anomaly {
    pub timestamp: String,
    pub value: f64,
    pub previous_value: f64,
    pub change_amount: f64,
    pub change_rate: f64,
    pub modified_z_score: f64,
    pub is_positive: bool,                      // true = spike, false = drop
    pub minutes_from_stream_start: Option<i64>, // Minutes since stream started
    pub stream_phase: String,                   // "early", "mid", "late", "unknown"
    pub stream_id: Option<i64>,                 // Stream ID for chat lookup
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendStats {
    pub viewer_trend: String, // "increasing", "decreasing", "stable"
    pub viewer_median: f64,
    pub viewer_mad: f64, // Median Absolute Deviation
    pub viewer_avg: f64,
    pub viewer_std_dev: f64,
    pub chat_trend: String,
    pub chat_median: f64,
    pub chat_mad: f64,
    pub chat_avg: f64,
    pub chat_std_dev: f64,
}

// ============================================================================
// Implementation Functions
// ============================================================================

// Japanese stopwords list
const JAPANESE_STOPWORDS: &[&str] = &[
    "の",
    "に",
    "は",
    "を",
    "た",
    "が",
    "で",
    "て",
    "と",
    "し",
    "れ",
    "さ",
    "ある",
    "いる",
    "も",
    "する",
    "から",
    "な",
    "こと",
    "として",
    "い",
    "や",
    "れる",
    "など",
    "なっ",
    "ない",
    "この",
    "ため",
    "その",
    "あっ",
    "よう",
    "また",
    "もの",
    "という",
    "あり",
    "まで",
    "られ",
    "なる",
    "へ",
    "か",
    "だ",
    "これ",
    "によって",
    "により",
    "おり",
    "より",
    "による",
    "ず",
    "なり",
    "られる",
    "において",
    "ば",
    "なかっ",
    "なく",
    "しかし",
    "について",
    "せ",
    "だっ",
    "その後",
    "できる",
    "それ",
    "う",
    "ので",
    "なお",
    "のみ",
    "でき",
    "き",
    "つ",
    "における",
    "および",
    "いう",
    "さらに",
    "でも",
    "ら",
    "たり",
    "その他",
    "に関する",
    "たち",
    "ます",
    "ん",
    "なら",
    "に対して",
    "特に",
    "せる",
    "および",
    "これら",
    "とき",
    "では",
    "にて",
    "ほか",
    "ながら",
    "うち",
    "そして",
    "とともに",
    "ただし",
    "かつて",
    "それぞれ",
    "または",
    "お",
    "ほど",
    "ものの",
    "に対する",
    "ほとんど",
    "と共に",
    "といった",
    "です",
    "とも",
    "ところ",
    "ここ",
    "wwww",
    "www",
    "ww",
    "w",
    "草",
    "くさ",
];

const ENGLISH_STOPWORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not", "on",
    "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say",
    "her", "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so",
    "up", "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like",
    "time", "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some",
    "could", "them", "see", "other", "than", "then", "now", "look", "only", "come", "its", "over",
    "think", "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way",
    "even", "new", "want", "because", "any", "these", "give", "day", "most", "us", "is", "was",
    "are", "been", "has", "had", "were", "said", "did", "having", "lol", "lmao", "omg", "wtf",
    "brb", "afk", "gg", "ez", "pog", "kekw", "kappa", "pogchamp",
];

/// Phase 1: Get word frequency analysis
pub fn get_word_frequency_analysis(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    limit: i32,
) -> Result<WordFrequencyResult, duckdb::Error> {
    // First, get all messages
    let mut sql = String::from(
        r#"
        SELECT message
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

    // パフォーマンス最適化: 最新100,000件に制限
    sql.push_str(" ORDER BY cm.timestamp DESC LIMIT 100000");

    let mut stmt = conn.prepare(&sql)?;
    let messages: Vec<String> =
        utils::query_map_with_params(&mut stmt, &params, |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

    // Process words in Rust (more flexible than SQL for text processing)
    let mut word_counts: HashMap<String, i64> = HashMap::new();
    let mut total_words = 0i64;
    let total_messages = messages.len() as i64;

    for message in messages {
        let words: Vec<String> = message
            .to_lowercase()
            .split_whitespace()
            .filter(|w| {
                w.len() > 1
                    && !JAPANESE_STOPWORDS.contains(w)
                    && !ENGLISH_STOPWORDS.contains(w)
                    && !w.starts_with("http")
                    && !w.starts_with("@")
            })
            .map(|w| w.to_string())
            .collect();

        total_words += words.len() as i64;

        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }
    }

    // Sort and limit
    let mut word_vec: Vec<(String, i64)> = word_counts.into_iter().collect();
    word_vec.sort_by(|a, b| b.1.cmp(&a.1));
    word_vec.truncate(limit as usize);

    let unique_words = word_vec.len() as i64;
    let avg_words_per_message = if total_messages > 0 {
        total_words as f64 / total_messages as f64
    } else {
        0.0
    };

    let words: Vec<WordFrequency> = word_vec
        .into_iter()
        .map(|(word, count)| WordFrequency {
            word,
            count,
            percentage: if total_words > 0 {
                (count as f64 / total_words as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    Ok(WordFrequencyResult {
        words,
        total_words,
        unique_words,
        avg_words_per_message,
        total_messages,
    })
}

/// Phase 1: Get emote analysis
pub fn get_emote_analysis(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<EmoteAnalysisResult, duckdb::Error> {
    // Get messages with hourly grouping
    let mut sql = String::from(
        r#"
        SELECT 
            cm.message,
            cm.user_name,
            EXTRACT(HOUR FROM cm.timestamp) as hour
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

    // パフォーマンス最適化: 最新100,000件に制限
    sql.push_str(" ORDER BY cm.timestamp DESC LIMIT 100000");

    let mut stmt = conn.prepare(&sql)?;
    let data: Vec<(String, String, i32)> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Process emotes
    let mut emote_counts: HashMap<String, i64> = HashMap::new();
    let mut emote_users: HashMap<String, Vec<String>> = HashMap::new();
    let mut hourly_counts: HashMap<i32, i64> = HashMap::new();
    let mut total_emotes = 0i64;
    let total_messages = data.len() as i64;

    for (message, user, hour) in data {
        // Simple emote detection (common Twitch emotes or words in PascalCase/CAPSLOCK)
        let emotes: Vec<String> = message
            .split_whitespace()
            .filter(|w| {
                (w.chars().all(|c| c.is_uppercase() || c.is_numeric())
                    || (w.chars().next().is_some_and(|c| c.is_uppercase())
                        && w.chars().skip(1).any(|c| c.is_uppercase())))
                    && w.len() > 2
                    && w.len() < 30
                    && !w.starts_with("HTTP")
            })
            .map(|w| w.to_string())
            .collect();

        total_emotes += emotes.len() as i64;
        *hourly_counts.entry(hour).or_insert(0) += emotes.len() as i64;

        for emote in emotes {
            *emote_counts.entry(emote.clone()).or_insert(0) += 1;
            emote_users.entry(emote).or_default().push(user.clone());
        }
    }

    // Sort emotes by count
    let mut emote_vec: Vec<(String, i64)> = emote_counts.into_iter().collect();
    emote_vec.sort_by(|a, b| b.1.cmp(&a.1));
    emote_vec.truncate(100);

    let emotes: Vec<EmoteUsage> = emote_vec
        .into_iter()
        .map(|(name, count)| {
            let users = emote_users
                .get(&name)
                .map(|u| {
                    let mut unique_users: Vec<_> = u.clone();
                    unique_users.sort();
                    unique_users.dedup();
                    unique_users.len() as i64
                })
                .unwrap_or(0);

            EmoteUsage {
                name,
                count,
                users,
                percentage: if total_emotes > 0 {
                    (count as f64 / total_emotes as f64) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    let hourly_pattern: Vec<HourlyEmotePattern> = (0..24)
        .map(|hour| HourlyEmotePattern {
            hour,
            count: *hourly_counts.get(&hour).unwrap_or(&0),
        })
        .collect();

    let emote_per_message_rate = if total_messages > 0 {
        total_emotes as f64 / total_messages as f64
    } else {
        0.0
    };

    Ok(EmoteAnalysisResult {
        emotes,
        total_emote_uses: total_emotes,
        emote_per_message_rate,
        hourly_pattern,
    })
}

/// Phase 1: Get message length statistics
pub fn get_message_length_stats(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<MessageLengthStats, duckdb::Error> {
    let mut sql = format!(
        r#"
        SELECT 
            LENGTH(cm.message) as msg_length,
            {}
        FROM chat_messages cm
        LEFT JOIN streams s ON cm.stream_id = s.id
        WHERE 1=1
        "#,
        chat_query::badges_select("cm")
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

    // パフォーマンス最適化: 最新100,000件に制限
    sql.push_str(" ORDER BY cm.timestamp DESC LIMIT 100000");

    let mut stmt = conn.prepare(&sql)?;
    let data: Vec<(i32, Option<String>)> =
        utils::query_map_with_params(&mut stmt, &params, |row| Ok((row.get(0)?, row.get(1).ok())))?
            .collect::<Result<Vec<_>, _>>()?;

    if data.is_empty() {
        return Ok(MessageLengthStats {
            avg_length: 0.0,
            median_length: 0.0,
            std_dev: 0.0,
            min_length: 0,
            max_length: 0,
            distribution: vec![],
            by_segment: vec![],
        });
    }

    // Calculate statistics
    let lengths: Vec<i32> = data.iter().map(|(len, _)| *len).collect();
    let sum: i32 = lengths.iter().sum();
    let count = lengths.len() as f64;
    let avg = sum as f64 / count;

    let mut sorted_lengths = lengths.clone();
    sorted_lengths.sort();
    let median = sorted_lengths[sorted_lengths.len() / 2] as f64;

    let variance: f64 = lengths
        .iter()
        .map(|&l| (l as f64 - avg).powi(2))
        .sum::<f64>()
        / count;
    let std_dev = variance.sqrt();

    let min_length = *sorted_lengths.first().unwrap_or(&0);
    let max_length = *sorted_lengths.last().unwrap_or(&0);

    // Distribution buckets
    let buckets = [
        ("0-10", 0, 10),
        ("11-30", 11, 30),
        ("31-50", 31, 50),
        ("51-100", 51, 100),
        ("101-200", 101, 200),
        ("201+", 201, i32::MAX),
    ];

    let distribution: Vec<LengthDistribution> = buckets
        .iter()
        .map(|(label, min, max)| {
            let count = lengths.iter().filter(|&&l| l >= *min && l <= *max).count() as i64;
            LengthDistribution {
                bucket: label.to_string(),
                count,
                percentage: (count as f64 / lengths.len() as f64) * 100.0,
            }
        })
        .collect();

    // By segment
    let mut segment_data: HashMap<String, Vec<i32>> = HashMap::new();
    for (length, badges_opt) in data {
        let segment = if let Some(badges_str) = badges_opt {
            let badges = utils::parse_badges(&badges_str).unwrap_or_default();
            if badges.contains(&"broadcaster".to_string()) {
                "broadcaster"
            } else if badges.contains(&"moderator".to_string()) {
                "moderator"
            } else if badges.contains(&"vip".to_string()) {
                "vip"
            } else if badges.contains(&"subscriber".to_string()) {
                "subscriber"
            } else {
                "regular"
            }
        } else {
            "regular"
        };

        segment_data
            .entry(segment.to_string())
            .or_default()
            .push(length);
    }

    let by_segment: Vec<SegmentLengthStats> = segment_data
        .into_iter()
        .map(|(segment, lengths)| {
            let avg = lengths.iter().sum::<i32>() as f64 / lengths.len() as f64;
            SegmentLengthStats {
                segment,
                avg_length: avg,
                message_count: lengths.len() as i64,
            }
        })
        .collect();

    Ok(MessageLengthStats {
        avg_length: avg,
        median_length: median,
        std_dev,
        min_length,
        max_length,
        distribution,
        by_segment,
    })
}

/// Phase 2: Get viewer-chat correlation analysis
pub fn get_viewer_chat_correlation(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<CorrelationResult, duckdb::Error> {
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

    // Get time-bucketed data with viewers and chats
    let mut sql = String::from(
        r#"
        WITH time_buckets AS (
            SELECT
                time_bucket(INTERVAL '5 minutes', ss.collected_at) as bucket,
                AVG(ss.viewer_count) as avg_viewers
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            WHERE ss.viewer_count IS NOT NULL
              AND ss.collected_at IS NOT NULL
              AND ss.collected_at > '1971-01-01'
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ref ch_name) = filter_channel_name {
        sql.push_str(" AND ss.channel_name = ?");
        params.push(ch_name.clone());
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
        ),
        chat_buckets AS (
            SELECT 
                time_bucket(INTERVAL '5 minutes', cm.timestamp) as bucket,
                COUNT(*) as chat_count
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
    );

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
            GROUP BY bucket
        )
        SELECT 
            tb.bucket::VARCHAR as timestamp,
            tb.avg_viewers,
            COALESCE(cb.chat_count, 0) as chat_count
        FROM time_buckets tb
        LEFT JOIN chat_buckets cb ON tb.bucket = cb.bucket
        ORDER BY tb.bucket
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let data: Vec<(String, f64, i64)> = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    if data.is_empty() {
        return Ok(CorrelationResult {
            pearson_coefficient: 0.0,
            interpretation: "No data".to_string(),
            scatter_data: vec![],
            hourly_correlation: vec![],
        });
    }

    // Calculate Pearson correlation
    let viewers: Vec<f64> = data.iter().map(|(_, v, _)| *v).collect();
    let chats: Vec<f64> = data.iter().map(|(_, _, c)| *c as f64).collect();

    let pearson = calculate_pearson_correlation(&viewers, &chats);

    let interpretation = match pearson {
        x if x >= 0.7 => "Strong positive correlation",
        x if x >= 0.4 => "Moderate positive correlation",
        x if x >= 0.1 => "Weak positive correlation",
        x if x >= -0.1 => "No correlation",
        x if x >= -0.4 => "Weak negative correlation",
        x if x >= -0.7 => "Moderate negative correlation",
        _ => "Strong negative correlation",
    }
    .to_string();

    let scatter_data: Vec<ScatterPoint> = data
        .iter()
        .map(|(ts, v, c)| ScatterPoint {
            timestamp: ts.clone(),
            viewers: *v as i32,
            chats: *c,
        })
        .collect();

    // Calculate hourly correlation
    let hourly_correlation =
        calculate_hourly_correlation(conn, channel_id, stream_id, start_time, end_time)?;

    Ok(CorrelationResult {
        pearson_coefficient: pearson,
        interpretation,
        scatter_data,
        hourly_correlation,
    })
}

fn calculate_pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for i in 0..x.len() {
        let diff_x = x[i] - mean_x;
        let diff_y = y[i] - mean_y;
        sum_xy += diff_x * diff_y;
        sum_x2 += diff_x * diff_x;
        sum_y2 += diff_y * diff_y;
    }

    if sum_x2 == 0.0 || sum_y2 == 0.0 {
        return 0.0;
    }

    sum_xy / (sum_x2 * sum_y2).sqrt()
}

fn calculate_hourly_correlation(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<Vec<HourlyCorrelation>, duckdb::Error> {
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

    let mut hourly_data: HashMap<i32, (Vec<f64>, Vec<f64>)> = HashMap::new();

    // Get hourly data
    let mut sql = String::from(
        r#"
        WITH hourly_viewers AS (
            SELECT
                EXTRACT(HOUR FROM ss.collected_at) as hour,
                time_bucket(INTERVAL '5 minutes', ss.collected_at) as bucket,
                AVG(ss.viewer_count) as avg_viewers
            FROM stream_stats ss
            LEFT JOIN streams s ON ss.stream_id = s.id
            WHERE ss.viewer_count IS NOT NULL
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ref ch_name) = filter_channel_name {
        sql.push_str(" AND ss.channel_name = ?");
        params.push(ch_name.clone());
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
            GROUP BY hour, bucket
        ),
        hourly_chats AS (
            SELECT 
                EXTRACT(HOUR FROM cm.timestamp) as hour,
                time_bucket(INTERVAL '5 minutes', cm.timestamp) as bucket,
                COUNT(*) as chat_count
            FROM chat_messages cm
            LEFT JOIN streams s ON cm.stream_id = s.id
            WHERE 1=1
        "#,
    );

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
            GROUP BY hour, bucket
        )
        SELECT 
            hv.hour,
            hv.avg_viewers,
            COALESCE(hc.chat_count, 0) as chat_count
        FROM hourly_viewers hv
        LEFT JOIN hourly_chats hc ON hv.bucket = hc.bucket
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let data: Vec<(i32, f64, i64)> = utils::query_map_with_params(&mut stmt, &params, |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    for (hour, viewers, chats) in data {
        let entry = hourly_data.entry(hour).or_insert((Vec::new(), Vec::new()));
        entry.0.push(viewers);
        entry.1.push(chats as f64);
    }

    let mut result: Vec<HourlyCorrelation> = hourly_data
        .into_iter()
        .map(|(hour, (viewers, chats))| HourlyCorrelation {
            hour,
            correlation: calculate_pearson_correlation(&viewers, &chats),
            sample_count: viewers.len() as i64,
        })
        .collect();

    result.sort_by_key(|h| h.hour);

    Ok(result)
}

/// Phase 2: Get category change impact analysis
pub fn get_category_change_impact(
    conn: &Connection,
    channel_id: i64,
    start_time: Option<&str>,
    end_time: Option<&str>,
) -> Result<CategoryImpactResult, duckdb::Error> {
    // channel_id を channel_name に変換してフィルター
    let filter_channel_name = conn
        .query_row(
            "SELECT channel_name FROM channels WHERE id = ? AND platform = 'twitch'",
            [channel_id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .ok();

    if filter_channel_name.is_none() {
        // channel_name が取得できない場合は空の結果を返す
        return Ok(CategoryImpactResult {
            changes: vec![],
            category_performance: vec![],
        });
    }

    // Get category changes
    let mut sql = String::from(
        r#"
        WITH ordered_stats AS (
            SELECT
                ss.collected_at,
                ss.category,
                ss.viewer_count,
                LAG(ss.category) OVER (ORDER BY ss.collected_at) as prev_category,
                LAG(ss.viewer_count) OVER (ORDER BY ss.collected_at) as prev_viewers,
                LEAD(ss.viewer_count, 5) OVER (ORDER BY ss.collected_at) as after_viewers
            FROM stream_stats ss
            WHERE ss.channel_name = ?
                AND ss.category IS NOT NULL
                AND ss.viewer_count IS NOT NULL
        "#,
    );

    let mut params = vec![filter_channel_name.clone().unwrap()];

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
            strftime(collected_at::TIMESTAMP, '%Y-%m-%dT%H:%M:%S.000Z') as timestamp,
            prev_category,
            category,
            prev_viewers,
            COALESCE(after_viewers, viewer_count) as after_viewers
        FROM ordered_stats
        WHERE prev_category IS NOT NULL
            AND prev_category != category
            AND prev_viewers > 0
        ORDER BY collected_at DESC
        LIMIT 50
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let changes_data: Vec<(String, String, String, i32, i32)> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let changes: Vec<CategoryChange> = changes_data
        .into_iter()
        .map(|(ts, from, to, before, after)| {
            let viewer_change = if before > 0 {
                ((after - before) as f64 / before as f64) * 100.0
            } else {
                0.0
            };

            CategoryChange {
                timestamp: ts,
                from_category: from,
                to_category: to,
                before_viewers: before,
                after_viewers: after,
                viewer_change_percent: viewer_change,
                chat_change_percent: 0.0, // Simplified for now
            }
        })
        .collect();

    // Get category performance
    let filter_channel_name_ref = filter_channel_name.as_ref().unwrap();
    let mut perf_sql = String::from(
        r#"
        WITH stats_with_interval AS (
            SELECT
                ss.category,
                ss.viewer_count,
                COALESCE((
                    SELECT COUNT(*)
                    FROM chat_messages cm
                    WHERE cm.stream_id = ss.stream_id
                      AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                      AND cm.timestamp < ss.collected_at
                ), 0) AS chat_rate_1min,
                EXTRACT(EPOCH FROM (
                    LEAD(ss.collected_at) OVER (ORDER BY ss.collected_at) - ss.collected_at
                )) / 60.0 AS interval_minutes
            FROM stream_stats ss
            WHERE ss.channel_name = ?
                AND ss.category IS NOT NULL
                AND ss.viewer_count IS NOT NULL
        "#,
    );

    let mut perf_params = vec![filter_channel_name_ref.clone()];

    if let Some(start) = start_time {
        perf_sql.push_str(" AND ss.collected_at >= ?");
        perf_params.push(start.to_string());
    }

    if let Some(end) = end_time {
        perf_sql.push_str(" AND ss.collected_at <= ?");
        perf_params.push(end.to_string());
    }

    perf_sql.push_str(
        r#"
        )
        SELECT 
            category,
            AVG(viewer_count) as avg_viewers,
            AVG(chat_rate_1min) as avg_chat_rate,
            SUM(COALESCE(interval_minutes, 1)) as total_minutes
        FROM stats_with_interval
        GROUP BY category
        ORDER BY avg_viewers DESC
        "#,
    );

    let mut perf_stmt = conn.prepare(&perf_sql)?;
    let category_performance: Vec<CategoryPerformance> =
        utils::query_map_with_params(&mut perf_stmt, &perf_params, |row| {
            Ok(CategoryPerformance {
                category: row.get(0)?,
                avg_viewers: row.get(1)?,
                avg_chat_rate: row.get(2)?,
                total_time_minutes: row.get(3)?,
                change_count: 0, // Calculated separately if needed
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CategoryImpactResult {
        changes,
        category_performance,
    })
}

/// Phase 3: Get chatter activity scores
pub fn get_chatter_activity_scores(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    limit: i32,
) -> Result<ChatterScoreResult, duckdb::Error> {
    // Get chatter data with badges - N+1クエリを避けるため一度に取得
    let mut sql = format!(
        r#"
        WITH user_badges AS (
            SELECT
                user_id,
                {},
                ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY timestamp DESC) as rn
            FROM chat_messages
            WHERE badges IS NOT NULL
        ),
        "#,
        chat_query::badges_select("chat_messages")
    );

    sql.push_str(
        r#"
        chatter_stats AS (
            SELECT
                cm.user_id,
                cm.user_name,
                COUNT(*) as message_count,
                COUNT(DISTINCT cm.stream_id) as stream_count,
                COUNT(DISTINCT DATE(cm.timestamp)) as active_days
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
            GROUP BY cm.user_id, cm.user_name
        )
        SELECT
            cs.user_name,
            cs.message_count,
            cs.stream_count,
            cs.active_days,
            ub.badges
        FROM chatter_stats cs
        LEFT JOIN user_badges ub ON cs.user_id = ub.user_id AND ub.rn = 1
        ORDER BY cs.message_count DESC
        "#,
    );

    let mut stmt = conn.prepare(&sql)?;
    let data: Vec<(String, i64, i64, i64, Option<String>)> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4).ok(),
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Calculate scores and cache badges
    let mut scores_with_users: Vec<(String, f64, i64, i64, Vec<String>)> = Vec::new();

    for (user_name, message_count, stream_count, active_days, badges_str) in data {
        let badges = if let Some(b) = badges_str {
            utils::parse_badges(&b).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Calculate badge weight
        let badge_weight = if badges.contains(&"broadcaster".to_string()) {
            3.0
        } else if badges.contains(&"moderator".to_string()) {
            2.5
        } else if badges.contains(&"vip".to_string()) {
            2.0
        } else if badges.contains(&"subscriber".to_string()) {
            1.5
        } else {
            1.0
        };

        // Activity Score calculation
        let normalized_messages = (message_count as f64).ln() * 10.0;
        let stream_participation = stream_count as f64 * 5.0;
        let consistency = active_days as f64 * 3.0;

        let score = (normalized_messages * 0.3)
            + (stream_participation * 0.3)
            + (consistency * 0.2)
            + (badge_weight * 10.0 * 0.2);

        scores_with_users.push((user_name, score, message_count, stream_count, badges));
    }

    // Sort by score
    scores_with_users.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Get top N - バッジはキャッシュされているため追加クエリ不要
    let top_scores: Vec<ChatterActivityScore> = scores_with_users
        .iter()
        .take(limit as usize)
        .enumerate()
        .map(
            |(idx, (user, score, msg_count, str_count, badges))| ChatterActivityScore {
                user_name: user.clone(),
                score: *score,
                message_count: *msg_count,
                stream_count: *str_count,
                badges: badges.clone(),
                rank: (idx + 1) as i32,
            },
        )
        .collect();

    // Score distribution
    let distribution_buckets = [
        ("0-20", 0.0, 20.0),
        ("21-40", 21.0, 40.0),
        ("41-60", 41.0, 60.0),
        ("61-80", 61.0, 80.0),
        ("81-100", 81.0, 100.0),
        ("100+", 100.0, f64::MAX),
    ];

    let total_users = scores_with_users.len() as f64;
    let score_distribution: Vec<ScoreDistribution> = distribution_buckets
        .iter()
        .map(|(label, min, max)| {
            let count = scores_with_users
                .iter()
                .filter(|(_, score, _, _, _)| *score >= *min && *score < *max)
                .count() as i64;
            ScoreDistribution {
                score_range: label.to_string(),
                user_count: count,
                percentage: if total_users > 0.0 {
                    (count as f64 / total_users) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Segment average scores (simplified)
    let segment_avg_scores = vec![]; // Would need to group by segment

    Ok(ChatterScoreResult {
        scores: top_scores,
        score_distribution,
        segment_avg_scores,
    })
}

/// Phase 4: Detect anomalies using Modified Z-Score (MAD-based)
/// This method is:
/// - Statistically robust (not affected by outliers)
/// - Scale-independent (works for both small and large streams)
/// - Correlated with trend detection (uses same statistical base)
pub fn detect_anomalies(
    conn: &Connection,
    channel_id: Option<i64>,
    stream_id: Option<i64>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    z_threshold: f64,
) -> Result<AnomalyResult, duckdb::Error> {
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

    // Get viewer data with stream information
    let mut viewer_sql = String::from(
        r#"
        SELECT
            strftime(ss.collected_at::TIMESTAMP, '%Y-%m-%dT%H:%M:%S') as timestamp,
            ss.viewer_count,
            ss.stream_id,
            strftime(s.started_at::TIMESTAMP, '%Y-%m-%dT%H:%M:%S') as stream_started_at
        FROM stream_stats ss
        LEFT JOIN streams s ON ss.stream_id = s.id
        WHERE ss.viewer_count IS NOT NULL
          AND ss.collected_at IS NOT NULL
          AND ss.collected_at > TIMESTAMP '1971-01-01'
          AND ss.viewer_count > 0
        "#,
    );

    let mut params: Vec<String> = Vec::new();

    if let Some(ref ch_name) = filter_channel_name {
        viewer_sql.push_str(" AND ss.channel_name = ?");
        params.push(ch_name.clone());
    }

    if let Some(st_id) = stream_id {
        viewer_sql.push_str(" AND ss.stream_id = ?");
        params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        viewer_sql.push_str(" AND ss.collected_at >= ?");
        params.push(start.to_string());
    }

    if let Some(end) = end_time {
        viewer_sql.push_str(" AND ss.collected_at <= ?");
        params.push(end.to_string());
    }

    viewer_sql.push_str(" ORDER BY ss.collected_at");

    let mut stmt = conn.prepare(&viewer_sql)?;
    let viewer_data_raw: Vec<(String, i32, Option<i64>, Option<String>)> =
        utils::query_map_with_params(&mut stmt, &params, |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2).ok(), row.get(3).ok()))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if viewer_data_raw.len() < 10 {
        return Ok(create_empty_anomaly_result());
    }

    // Remove consecutive duplicates (Twitch viewer count updates in intervals, not real-time)
    // This prevents normal platform updates from being flagged as anomalies
    let viewer_data = deduplicate_consecutive_values_with_metadata(&viewer_data_raw);

    if viewer_data.len() < 5 {
        return Ok(create_empty_anomaly_result());
    }

    let n = viewer_data.len();
    let viewer_values: Vec<f64> = viewer_data.iter().map(|(_, v, _, _)| *v as f64).collect();

    // Calculate robust statistics (median and MAD)
    let (viewer_median, viewer_mad) = calculate_median_and_mad(&viewer_values);

    // Also calculate mean and std dev for reference
    let viewer_avg = viewer_values.iter().sum::<f64>() / n as f64;
    let viewer_variance = viewer_values
        .iter()
        .map(|v| (v - viewer_avg).powi(2))
        .sum::<f64>()
        / n as f64;
    let viewer_std_dev = viewer_variance.sqrt();

    // Calculate stream duration and position for each point
    // Group data by stream_id to calculate total stream duration
    let mut stream_info: HashMap<i64, (i64, i64)> = HashMap::new(); // stream_id -> (start_ts, end_ts)

    for (ts, _, stream_id_opt, started_at_opt) in &viewer_data {
        if let (Some(stream_id), Some(started_at)) = (stream_id_opt, started_at_opt) {
            let current_ts = parse_timestamp(ts);
            let start_ts = parse_timestamp(started_at);

            if current_ts > 0 && start_ts > 0 {
                stream_info
                    .entry(*stream_id)
                    .and_modify(|(start, end)| {
                        *start = (*start).min(start_ts);
                        *end = (*end).max(current_ts);
                    })
                    .or_insert((start_ts, current_ts));
            }
        }
    }

    // Calculate position in stream for each point
    let stream_positions: Vec<Option<(i64, f64)>> = viewer_data
        .iter()
        .map(|(ts, _, stream_id_opt, started_at_opt)| {
            if let (Some(stream_id), Some(started_at)) = (stream_id_opt, started_at_opt) {
                let current_ts = parse_timestamp(ts);
                let start_ts = parse_timestamp(started_at);

                if current_ts > 0 && start_ts > 0 {
                    let minutes_from_start = (current_ts - start_ts) / 60;
                    let position_ratio = stream_info.get(stream_id).and_then(|(stream_start, stream_end)| {
                        let total = stream_end - stream_start;
                        if total > 0 {
                            Some((current_ts - stream_start) as f64 / total as f64)
                        } else {
                            None
                        }
                    });
                    // When we have valid timestamps, we always have at least minutes_from_start.
                    // If duration is 0 (single point) or stream not in stream_info, treat as early (ratio 0).
                    return Some((minutes_from_start, position_ratio.unwrap_or(0.0)));
                }
            }
            None
        })
        .collect();

    // Detect anomalies using Modified Z-Score with time interval consideration
    let mut viewer_anomalies: Vec<Anomaly> = Vec::new();

    // Skip first and last point to avoid stream start/end effects
    for i in 1..(n - 1) {
        let current = viewer_values[i];
        let previous = viewer_values[i - 1];

        // Skip only the stream start period (first 10 minutes or 10%)
        // Stream end periods are NOT skipped because:
        // 1. viewer_count > 0 filter already excludes actual stream end (0 viewers)
        // 2. Spikes/drops during late stream are valuable insights (e.g., surprise announcements, viewer retention)
        if let Some((minutes_from_start, position_ratio)) = stream_positions[i] {
            // Skip first 10 minutes (absolute) OR first 10% (relative)
            // Using OR ensures we skip early period regardless of stream length
            let is_early = minutes_from_start < 10 || position_ratio < 0.10;

            if is_early {
                continue; // Skip stream start period only
            }
        } else {
            // If we can't determine stream position (no stream_id or started_at), skip to be safe
            continue;
        }

        // Parse timestamps to calculate time interval
        let current_ts = parse_timestamp(&viewer_data[i].0);
        let previous_ts = parse_timestamp(&viewer_data[i - 1].0);
        let time_interval_minutes = ((current_ts - previous_ts) as f64 / 60.0).max(1.0);

        // Calculate Modified Z-Score for current value
        // Modified Z-Score = 0.6745 * (value - median) / MAD
        // Values with |Modified Z-Score| > threshold are considered outliers
        let modified_z = calculate_modified_z_score(current, viewer_median, viewer_mad);

        // Detect anomaly if Modified Z-Score exceeds threshold
        // Additional filter: for rapid changes (< 5 min interval), require higher Z-score
        // This accounts for Twitch's delayed update behavior
        let effective_threshold = if time_interval_minutes < 5.0 {
            z_threshold * 1.5 // Stricter threshold for rapid changes
        } else {
            z_threshold
        };

        if modified_z.abs() >= effective_threshold {
            let change_amount = current - previous;
            let change_rate = if previous > 0.0 {
                (change_amount / previous) * 100.0
            } else {
                0.0
            };

            // Determine stream phase based on relative position in stream
            let (stream_phase, minutes_from_start) =
                if let Some((mins, ratio)) = stream_positions[i] {
                    let phase = if ratio < 0.33 {
                        "early" // First 1/3 of stream
                    } else if ratio < 0.67 {
                        "mid" // Middle 1/3 of stream
                    } else {
                        "late" // Last 1/3 of stream
                    };
                    (phase.to_string(), Some(mins))
                } else {
                    ("unknown".to_string(), None)
                };

            viewer_anomalies.push(Anomaly {
                timestamp: viewer_data[i].0.clone(),
                value: current,
                previous_value: previous,
                change_amount,
                change_rate,
                modified_z_score: modified_z,
                is_positive: current > previous, // Spike if current > previous, drop otherwise
                minutes_from_stream_start: minutes_from_start,
                stream_phase,
                stream_id: viewer_data[i].2,
            });
        }
    }

    // Sort by absolute Modified Z-Score (most significant first)
    viewer_anomalies.sort_by(|a, b| {
        b.modified_z_score
            .abs()
            .partial_cmp(&a.modified_z_score.abs())
            .unwrap()
    });

    // Limit to top 50 most significant anomalies
    viewer_anomalies.truncate(50);

    // ========================================================================
    // Chat anomaly detection
    // ========================================================================

    // Get chat rate data with stream information (using same pattern as viewer data)
    let mut chat_sql = String::from(
        r#"
        SELECT
            strftime(ss.collected_at::TIMESTAMP, '%Y-%m-%dT%H:%M:%S') as timestamp,
            COALESCE((
                SELECT COUNT(*)
                FROM chat_messages cm
                WHERE cm.stream_id = ss.stream_id
                  AND cm.timestamp >= ss.collected_at - INTERVAL '1 minute'
                  AND cm.timestamp < ss.collected_at
            ), 0) AS chat_rate_1min,
            ss.stream_id,
            strftime(s.started_at::TIMESTAMP, '%Y-%m-%dT%H:%M:%S') as stream_started_at
        FROM stream_stats ss
        LEFT JOIN streams s ON ss.stream_id = s.id
        WHERE ss.collected_at IS NOT NULL
          AND ss.collected_at > TIMESTAMP '1971-01-01'
        "#,
    );

    let mut chat_params: Vec<String> = Vec::new();

    if let Some(ref ch_name) = filter_channel_name {
        chat_sql.push_str(" AND ss.channel_name = ?");
        chat_params.push(ch_name.clone());
    }

    if let Some(st_id) = stream_id {
        chat_sql.push_str(" AND ss.stream_id = ?");
        chat_params.push(st_id.to_string());
    }

    if let Some(start) = start_time {
        chat_sql.push_str(" AND ss.collected_at >= ?");
        chat_params.push(start.to_string());
    }

    if let Some(end) = end_time {
        chat_sql.push_str(" AND ss.collected_at <= ?");
        chat_params.push(end.to_string());
    }

    chat_sql.push_str(" ORDER BY ss.collected_at");

    let mut chat_stmt = conn.prepare(&chat_sql)?;
    let chat_data_raw: Vec<(String, i64, Option<i64>, Option<String>)> =
        utils::query_map_with_params(&mut chat_stmt, &chat_params, |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2).ok(), row.get(3).ok()))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut chat_anomalies = vec![];
    let mut chat_trend = "N/A".to_string();
    let mut chat_median = 0.0;
    let mut chat_mad = 0.0;
    let mut chat_avg = 0.0;
    let mut chat_std_dev = 0.0;

    // Only process chat anomalies if we have enough data
    if chat_data_raw.len() >= 10 {
        // Convert i64 to i32 for compatibility with deduplicate function
        let chat_data_i32: Vec<(String, i32, Option<i64>, Option<String>)> = chat_data_raw
            .iter()
            .map(|(ts, val, sid, sa)| (ts.clone(), *val as i32, *sid, sa.clone()))
            .collect();

        // Remove consecutive duplicates
        let chat_data = deduplicate_consecutive_values_with_metadata(&chat_data_i32);

        if chat_data.len() >= 5 {
            let chat_n = chat_data.len();
            let chat_values: Vec<f64> = chat_data.iter().map(|(_, v, _, _)| *v as f64).collect();

            // Calculate robust statistics for chat
            let (chat_median_calc, chat_mad_calc) = calculate_median_and_mad(&chat_values);
            chat_median = chat_median_calc;
            chat_mad = chat_mad_calc;

            // Calculate mean and std dev for chat
            chat_avg = chat_values.iter().sum::<f64>() / chat_n as f64;
            let chat_variance = chat_values
                .iter()
                .map(|v| (v - chat_avg).powi(2))
                .sum::<f64>()
                / chat_n as f64;
            chat_std_dev = chat_variance.sqrt();

            // Reuse stream position calculations from viewer data
            // (stream_info HashMap is already calculated above)

            // Calculate position in stream for each chat data point
            let chat_stream_positions: Vec<Option<(i64, f64)>> = chat_data
                .iter()
                .map(|(ts, _, stream_id_opt, started_at_opt)| {
                    if let (Some(stream_id), Some(started_at)) = (stream_id_opt, started_at_opt) {
                        let current_ts = parse_timestamp(ts);
                        let start_ts = parse_timestamp(started_at);

                        if current_ts > 0 && start_ts > 0 {
                            if let Some((stream_start, stream_end)) = stream_info.get(stream_id) {
                                let minutes_from_start = (current_ts - start_ts) / 60;
                                let total_duration = (stream_end - stream_start) / 60;

                                if total_duration > 0 {
                                    let position_ratio = (current_ts - stream_start) as f64
                                        / (stream_end - stream_start) as f64;
                                    return Some((minutes_from_start, position_ratio));
                                }
                            }
                        }
                    }
                    None
                })
                .collect();

            // Detect chat anomalies using same algorithm
            for i in 1..(chat_n - 1) {
                let current = chat_values[i];
                let previous = chat_values[i - 1];

                // Skip stream start period
                if let Some((minutes_from_start, position_ratio)) = chat_stream_positions[i] {
                    let is_early = minutes_from_start < 10 || position_ratio < 0.10;

                    if is_early {
                        continue;
                    }
                } else {
                    continue;
                }

                // Calculate time interval
                let current_ts = parse_timestamp(&chat_data[i].0);
                let previous_ts = parse_timestamp(&chat_data[i - 1].0);
                let time_interval_minutes = ((current_ts - previous_ts) as f64 / 60.0).max(1.0);

                // Calculate Modified Z-Score
                let modified_z = calculate_modified_z_score(current, chat_median, chat_mad);

                // Apply threshold with time interval consideration
                let effective_threshold = if time_interval_minutes < 5.0 {
                    z_threshold * 1.5
                } else {
                    z_threshold
                };

                if modified_z.abs() >= effective_threshold {
                    let change_amount = current - previous;
                    let change_rate = if previous > 0.0 {
                        (change_amount / previous) * 100.0
                    } else {
                        0.0
                    };

                    let (stream_phase, minutes_from_start) =
                        if let Some((mins, ratio)) = chat_stream_positions[i] {
                            let phase = if ratio < 0.33 {
                                "early"
                            } else if ratio < 0.67 {
                                "mid"
                            } else {
                                "late"
                            };
                            (phase.to_string(), Some(mins))
                        } else {
                            ("unknown".to_string(), None)
                        };

                    chat_anomalies.push(Anomaly {
                        timestamp: chat_data[i].0.clone(),
                        value: current,
                        previous_value: previous,
                        change_amount,
                        change_rate,
                        modified_z_score: modified_z,
                        is_positive: current > previous,
                        minutes_from_stream_start: minutes_from_start,
                        stream_phase,
                        stream_id: chat_data[i].2,
                    });
                }
            }

            // Sort by absolute Modified Z-Score
            chat_anomalies.sort_by(|a, b| {
                b.modified_z_score
                    .abs()
                    .partial_cmp(&a.modified_z_score.abs())
                    .unwrap()
            });

            // Limit to top 50
            chat_anomalies.truncate(50);

            // Calculate chat trend
            chat_trend = calculate_trend(&chat_values, chat_median, chat_mad);
        }
    }

    // Determine trend using MAD-based approach
    let viewer_trend = calculate_trend(&viewer_values, viewer_median, viewer_mad);

    Ok(AnomalyResult {
        viewer_anomalies,
        chat_anomalies,
        trend_stats: TrendStats {
            viewer_trend,
            viewer_median,
            viewer_mad,
            viewer_avg,
            viewer_std_dev,
            chat_trend,
            chat_median,
            chat_mad,
            chat_avg,
            chat_std_dev,
        },
    })
}

/// Calculate median and MAD (Median Absolute Deviation)
fn calculate_median_and_mad(values: &[f64]) -> (f64, f64) {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted.len();
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    // Calculate MAD: median of absolute deviations from median
    let mut abs_deviations: Vec<f64> = values.iter().map(|&v| (v - median).abs()).collect();
    abs_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mad = if n % 2 == 0 {
        (abs_deviations[n / 2 - 1] + abs_deviations[n / 2]) / 2.0
    } else {
        abs_deviations[n / 2]
    };

    // MAD can be 0 if data is very stable, use a minimum value
    let mad = mad.max(1.0);

    (median, mad)
}

/// Calculate Modified Z-Score
/// Modified Z-Score = 0.6745 * (value - median) / MAD
/// The constant 0.6745 is the 75th percentile of the standard normal distribution
fn calculate_modified_z_score(value: f64, median: f64, mad: f64) -> f64 {
    0.6745 * (value - median) / mad
}

/// Calculate trend based on MAD-normalized change
fn calculate_trend(values: &[f64], _median: f64, mad: f64) -> String {
    let n = values.len();
    if n < 10 {
        return "insufficient data".to_string();
    }

    // Calculate medians of first and second half
    let first_half = &values[..n / 2];
    let second_half = &values[n / 2..];

    let (first_median, _) = calculate_median_and_mad(first_half);
    let (second_median, _) = calculate_median_and_mad(second_half);

    // Calculate change in terms of MAD
    let change_in_mad = (second_median - first_median) / mad;

    // Trend detection thresholds (in terms of MAD)
    // > 1.5 MAD = significant increase
    // < -1.5 MAD = significant decrease
    if change_in_mad > 1.5 {
        "increasing".to_string()
    } else if change_in_mad < -1.5 {
        "decreasing".to_string()
    } else {
        "stable".to_string()
    }
}

/// Remove consecutive duplicate values with metadata
/// Twitch viewer count updates every few minutes, not in real-time
/// This function keeps only the points where the value actually changed
fn deduplicate_consecutive_values_with_metadata(
    data: &[(String, i32, Option<i64>, Option<String>)],
) -> Vec<(String, i32, Option<i64>, Option<String>)> {
    if data.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    result.push(data[0].clone());

    for i in 1..data.len() {
        // Only keep if value changed from previous
        if data[i].1 != data[i - 1].1 {
            result.push(data[i].clone());
        }
    }

    result
}

/// Parse timestamp to get Unix timestamp in seconds
fn parse_timestamp(ts: &str) -> i64 {
    use chrono::prelude::*;

    // Try parsing as RFC3339 first (with timezone)
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return dt.timestamp();
    }

    // Try parsing as local time without timezone (new format)
    if let Ok(naive) = NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S") {
        if let Some(local) = Local.from_local_datetime(&naive).single() {
            return local.timestamp();
        }
    }

    0
}

fn create_empty_anomaly_result() -> AnomalyResult {
    AnomalyResult {
        viewer_anomalies: vec![],
        chat_anomalies: vec![],
        trend_stats: TrendStats {
            viewer_trend: "no data".to_string(),
            chat_trend: "N/A".to_string(),
            viewer_median: 0.0,
            viewer_mad: 0.0,
            viewer_avg: 0.0,
            viewer_std_dev: 0.0,
            chat_median: 0.0,
            chat_mad: 0.0,
            chat_avg: 0.0,
            chat_std_dev: 0.0,
        },
    }
}
