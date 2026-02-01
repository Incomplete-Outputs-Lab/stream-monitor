use crate::database::models::{ChatMessage, StreamStats};
use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 集計されたストリーム統計データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStreamStats {
    pub timestamp: String,     // 集計期間の開始時刻
    pub interval_minutes: i32, // 集計間隔（分）
    pub avg_viewer_count: Option<f64>,
    pub max_viewer_count: Option<i32>,
    pub min_viewer_count: Option<i32>,
    pub chat_rate_avg: f64,
    pub data_points: i32,
}

/// 集計されたチャット統計データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedChatStats {
    pub timestamp: String,     // 集計期間の開始時刻
    pub interval_minutes: i32, // 集計間隔（分）
    pub message_count: i64,
    pub unique_users: i64,
    pub messages_per_minute: f64,
}

pub struct DataAggregator;

impl DataAggregator {
    /// ストリーム統計データを指定した間隔で集計
    pub fn aggregate_stream_stats(
        stats: &[StreamStats],
        interval_minutes: i32,
    ) -> Vec<AggregatedStreamStats> {
        if stats.is_empty() {
            return vec![];
        }

        // タイムスタンプでソート（念のため）
        let mut sorted_stats = stats.to_vec();
        sorted_stats.sort_by(|a, b| a.collected_at.cmp(&b.collected_at));

        // 間隔ごとにグループ化
        let mut grouped_data: HashMap<String, Vec<&StreamStats>> = HashMap::new();

        for stat in &sorted_stats {
            let interval_start = Self::get_interval_start(&stat.collected_at, interval_minutes);
            grouped_data.entry(interval_start).or_default().push(stat);
        }

        // 各グループを集計
        let mut result: Vec<AggregatedStreamStats> = grouped_data
            .into_iter()
            .map(|(timestamp, group_stats)| {
                Self::aggregate_stream_stats_group(&timestamp, interval_minutes, &group_stats)
            })
            .collect();

        // タイムスタンプでソート
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        result
    }

    /// チャットメッセージデータを指定した間隔で集計
    ///
    /// # 注意
    /// この関数は現在、テストでのみ使用されています。
    /// 実際のコマンド（`chat.rs`など）では直接SQLで集計を行っています。
    /// 将来的にチャット集計機能を実装する際に使用予定です。
    #[allow(dead_code)]
    pub fn aggregate_chat_messages(
        messages: &[ChatMessage],
        interval_minutes: i32,
    ) -> Vec<AggregatedChatStats> {
        if messages.is_empty() {
            return vec![];
        }

        // タイムスタンプでソート（念のため）
        let mut sorted_messages = messages.to_vec();
        sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // 間隔ごとにグループ化
        let mut grouped_data: HashMap<String, Vec<&ChatMessage>> = HashMap::new();

        for message in &sorted_messages {
            let interval_start = Self::get_interval_start(&message.timestamp, interval_minutes);
            grouped_data
                .entry(interval_start)
                .or_default()
                .push(message);
        }

        // 各グループを集計
        let mut result: Vec<AggregatedChatStats> = grouped_data
            .into_iter()
            .map(|(timestamp, group_messages)| {
                Self::aggregate_chat_messages_group(&timestamp, interval_minutes, &group_messages)
            })
            .collect();

        // タイムスタンプでソート
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        result
    }

    /// タイムスタンプを指定した間隔の開始時刻に丸める
    fn get_interval_start(timestamp: &str, interval_minutes: i32) -> String {
        // RFC3339形式のタイムスタンプをパース
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            let dt_utc = dt.with_timezone(&chrono::Utc);

            // 分単位で丸める
            let minutes_since_epoch = dt_utc.timestamp() / 60;
            let interval_start_minutes =
                (minutes_since_epoch / interval_minutes as i64) * interval_minutes as i64;
            let interval_start = chrono::Utc
                .timestamp_opt(interval_start_minutes * 60, 0)
                .unwrap();

            // RFC3339形式で出力（+00:00をZに変換）
            interval_start.to_rfc3339().replace("+00:00", "Z")
        } else {
            // パース失敗時は元のタイムスタンプを返す
            timestamp.to_string()
        }
    }

    /// ストリーム統計のグループを集計
    fn aggregate_stream_stats_group(
        timestamp: &str,
        interval_minutes: i32,
        stats: &[&StreamStats],
    ) -> AggregatedStreamStats {
        let data_points = stats.len() as i32;

        // 視聴者数の統計
        let viewer_counts: Vec<i32> = stats.iter().filter_map(|stat| stat.viewer_count).collect();

        let avg_viewer_count = if viewer_counts.is_empty() {
            None
        } else {
            Some(viewer_counts.iter().map(|&v| v as f64).sum::<f64>() / viewer_counts.len() as f64)
        };

        let max_viewer_count = viewer_counts.iter().max().copied();
        let min_viewer_count = viewer_counts.iter().min().copied();

        // チャットレートの平均
        let chat_rates: Vec<f64> = stats
            .iter()
            .map(|stat| stat.chat_rate_1min as f64)
            .collect();
        let chat_rate_avg = if chat_rates.is_empty() {
            0.0
        } else {
            chat_rates.iter().sum::<f64>() / chat_rates.len() as f64
        };

        AggregatedStreamStats {
            timestamp: timestamp.to_string(),
            interval_minutes,
            avg_viewer_count,
            max_viewer_count,
            min_viewer_count,
            chat_rate_avg,
            data_points,
        }
    }

    /// チャットメッセージのグループを集計
    ///
    /// # 注意
    /// この関数は`aggregate_chat_messages`の内部関数です。
    /// 将来的にチャット集計機能を実装する際に使用予定です。
    #[allow(dead_code)]
    fn aggregate_chat_messages_group(
        timestamp: &str,
        interval_minutes: i32,
        messages: &[&ChatMessage],
    ) -> AggregatedChatStats {
        let message_count = messages.len() as i64;

        // ユニークユーザー数
        let mut unique_users = std::collections::HashSet::new();
        for message in messages {
            unique_users.insert(&message.user_name);
        }
        let unique_users_count = unique_users.len() as i64;

        // 1分あたりのメッセージ数（間隔で割る）
        let messages_per_minute = message_count as f64 / interval_minutes as f64;

        AggregatedChatStats {
            timestamp: timestamp.to_string(),
            interval_minutes,
            message_count,
            unique_users: unique_users_count,
            messages_per_minute,
        }
    }

    /// 1分間隔の集計（便利関数）
    pub fn aggregate_to_1min(stats: &[StreamStats]) -> Vec<AggregatedStreamStats> {
        Self::aggregate_stream_stats(stats, 1)
    }

    /// 5分間隔の集計（便利関数）
    pub fn aggregate_to_5min(stats: &[StreamStats]) -> Vec<AggregatedStreamStats> {
        Self::aggregate_stream_stats(stats, 5)
    }

    /// 1時間間隔の集計（便利関数）
    pub fn aggregate_to_1hour(stats: &[StreamStats]) -> Vec<AggregatedStreamStats> {
        Self::aggregate_stream_stats(stats, 60)
    }

    /// チャットメッセージを1分間隔で集計（便利関数）
    ///
    /// # 注意
    /// この関数は将来使用予定です。現在はテストでのみ使用されています。
    #[allow(dead_code)]
    pub fn aggregate_chat_to_1min(messages: &[ChatMessage]) -> Vec<AggregatedChatStats> {
        Self::aggregate_chat_messages(messages, 1)
    }

    /// チャットメッセージを5分間隔で集計（便利関数）
    ///
    /// # 注意
    /// この関数は将来使用予定です。現在はテストでのみ使用されています。
    #[allow(dead_code)]
    pub fn aggregate_chat_to_5min(messages: &[ChatMessage]) -> Vec<AggregatedChatStats> {
        Self::aggregate_chat_messages(messages, 5)
    }

    /// チャットメッセージを1時間間隔で集計（便利関数）
    ///
    /// # 注意
    /// この関数は将来使用予定です。現在はテストでのみ使用されています。
    #[allow(dead_code)]
    pub fn aggregate_chat_to_1hour(messages: &[ChatMessage]) -> Vec<AggregatedChatStats> {
        Self::aggregate_chat_messages(messages, 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{ChatMessage, StreamStats};

    #[test]
    fn test_aggregate_stream_stats_empty() {
        let stats: Vec<StreamStats> = vec![];
        let result = DataAggregator::aggregate_stream_stats(&stats, 1);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_aggregate_stream_stats_single_point() {
        let stats = vec![StreamStats {
            id: Some(1),
            stream_id: 1,
            collected_at: "2024-01-01T12:00:00Z".to_string(),
            viewer_count: Some(100),
            chat_rate_1min: 10,
            category: None,
            twitch_user_id: None,
            channel_name: None,
        }];

        let result = DataAggregator::aggregate_stream_stats(&stats, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].avg_viewer_count, Some(100.0));
        assert_eq!(result[0].max_viewer_count, Some(100));
        assert_eq!(result[0].min_viewer_count, Some(100));
        assert_eq!(result[0].chat_rate_avg, 10.0);
        assert_eq!(result[0].data_points, 1);
    }

    #[test]
    fn test_aggregate_stream_stats_multiple_points() {
        let stats = vec![
            StreamStats {
                id: Some(1),
                stream_id: 1,
                collected_at: "2024-01-01T12:00:00Z".to_string(),
                viewer_count: Some(100),
                chat_rate_1min: 10,
                category: None,
                twitch_user_id: None,
                channel_name: None,
            },
            StreamStats {
                id: Some(2),
                stream_id: 1,
                collected_at: "2024-01-01T12:01:00Z".to_string(),
                viewer_count: Some(150),
                chat_rate_1min: 15,
                category: None,
                twitch_user_id: None,
                channel_name: None,
            },
        ];

        let result = DataAggregator::aggregate_stream_stats(&stats, 5); // 5分間隔
        assert_eq!(result.len(), 1); // 同じ5分間隔に含まれる
        assert_eq!(result[0].avg_viewer_count, Some(125.0)); // (100 + 150) / 2
        assert_eq!(result[0].max_viewer_count, Some(150));
        assert_eq!(result[0].min_viewer_count, Some(100));
        assert_eq!(result[0].chat_rate_avg, 12.5); // (10 + 15) / 2
        assert_eq!(result[0].data_points, 2);
    }

    #[test]
    fn test_aggregate_chat_messages() {
        let messages = vec![
            ChatMessage {
                id: Some(1),
                stream_id: 1,
                timestamp: "2024-01-01T12:00:00Z".to_string(),
                platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
                user_id: Some("user1".to_string()),
                user_name: "User1".to_string(),
                message: "Hello".to_string(),
                message_type: "normal".to_string(),
            },
            ChatMessage {
                id: Some(2),
                stream_id: 1,
                timestamp: "2024-01-01T12:01:00Z".to_string(),
                platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
                user_id: Some("user2".to_string()),
                user_name: "User2".to_string(),
                message: "Hi".to_string(),
                message_type: "normal".to_string(),
            },
        ];

        let result = DataAggregator::aggregate_chat_messages(&messages, 5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message_count, 2);
        assert_eq!(result[0].unique_users, 2);
        assert_eq!(result[0].messages_per_minute, 2.0 / 5.0); // 2 messages / 5 minutes
    }

    #[test]
    fn test_get_interval_start() {
        // 1分間隔
        assert_eq!(
            DataAggregator::get_interval_start("2024-01-01T12:34:56Z", 1),
            "2024-01-01T12:34:00Z"
        );

        // 5分間隔
        assert_eq!(
            DataAggregator::get_interval_start("2024-01-01T12:34:56Z", 5),
            "2024-01-01T12:30:00Z"
        );

        // 60分（1時間）間隔
        assert_eq!(
            DataAggregator::get_interval_start("2024-01-01T12:34:56Z", 60),
            "2024-01-01T12:00:00Z"
        );
    }
}
