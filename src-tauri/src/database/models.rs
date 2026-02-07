use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Option<i64>,
    pub platform: String,
    pub channel_id: String, // Twitch: login (変更可能), YouTube: channel_id
    pub channel_name: String,
    pub display_name: Option<String>,
    pub profile_image_url: Option<String>,
    pub enabled: bool,
    pub poll_interval: i32,
    pub follower_count: Option<i32>,
    pub broadcaster_type: Option<String>,
    pub view_count: Option<i32>,
    pub is_auto_discovered: Option<bool>,
    pub discovered_at: Option<String>,
    pub twitch_user_id: Option<i64>, // Twitchの不変なuser ID（内部識別子）
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: Option<i64>,
    pub channel_id: i64,
    pub stream_id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub thumbnail_url: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub id: Option<i64>,
    pub stream_id: i64,
    pub collected_at: String,
    pub viewer_count: Option<i32>,
    pub chat_rate_1min: Option<i64>, // Chat messages in the last 1 minute
    pub category: Option<String>,
    pub game_id: Option<String>,
    pub title: Option<String>,
    pub follower_count: Option<i32>,
    pub twitch_user_id: Option<String>,
    pub channel_name: Option<String>,
}

/// Combined stream data returned by collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamData {
    pub stream_id: String, // Platform-specific stream ID
    pub title: Option<String>,
    pub category: Option<String>,
    pub game_id: Option<String>, // Platform-specific game/category ID
    pub thumbnail_url: Option<String>,
    pub started_at: String,
    pub viewer_count: Option<i32>,
    pub follower_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelWithStats {
    #[serde(flatten)]
    pub channel: Channel,
    pub is_live: bool,
    pub current_viewers: Option<i32>,
    pub current_title: Option<String>,
}

/// Event payload for channel stats updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatsEvent {
    pub channel_id: i64,
    pub is_live: bool,
    pub viewer_count: Option<i32>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub channel_id: Option<i64>,
    pub stream_id: Option<i64>,
    pub timestamp: String,
    pub platform: String,
    pub user_id: Option<String>,
    pub user_name: String,
    pub display_name: Option<String>, // Twitch表示名（ユーザーが設定した名前）
    pub message: String,
    pub message_type: String,
    pub badges: Option<Vec<String>>,
    pub badge_info: Option<String>, // サブスク月数等の詳細情報 (例: "subscriber:24")
}

/// ゲームカテゴリ（Twitch game/category）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameCategory {
    pub game_id: String,              // Twitch game ID（プライマリキー）
    pub game_name: String,            // カテゴリ名（表示用、言語ごとに異なる可能性あり）
    pub box_art_url: Option<String>,  // ボックスアート画像URL
    pub last_updated: Option<String>, // 最終更新日時
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_serialization() {
        let channel = Channel {
            id: Some(1),
            platform: crate::constants::database::PLATFORM_TWITCH.to_string(),
            channel_id: "test_channel".to_string(),
            channel_name: "Test Channel".to_string(),
            display_name: None,
            profile_image_url: None,
            follower_count: None,
            broadcaster_type: None,
            view_count: None,
            is_auto_discovered: None,
            discovered_at: None,
            twitch_user_id: Some(123456789),
            enabled: true,
            poll_interval: 60,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&channel).unwrap();
        let deserialized: Channel = serde_json::from_str(&json).unwrap();

        assert_eq!(channel.id, deserialized.id);
        assert_eq!(channel.platform, deserialized.platform);
        assert_eq!(channel.channel_id, deserialized.channel_id);
        assert_eq!(channel.channel_name, deserialized.channel_name);
        assert_eq!(channel.twitch_user_id, deserialized.twitch_user_id);
    }

    #[test]
    fn test_stream_stats_serialization() {
        let stats = StreamStats {
            id: Some(1),
            stream_id: 1,
            collected_at: "2024-01-01T00:00:00Z".to_string(),
            viewer_count: Some(100),
            chat_rate_1min: Some(5),
            category: Some("Just Chatting".to_string()),
            game_id: Some("509658".to_string()),
            title: Some("Test Stream Title".to_string()),
            follower_count: Some(5000),
            twitch_user_id: Some("123456789".to_string()),
            channel_name: Some("test_channel".to_string()),
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: StreamStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.viewer_count, deserialized.viewer_count);
        assert_eq!(stats.category, deserialized.category);
        assert_eq!(stats.title, deserialized.title);
        assert_eq!(stats.follower_count, deserialized.follower_count);
        assert_eq!(stats.twitch_user_id, deserialized.twitch_user_id);
        assert_eq!(stats.channel_name, deserialized.channel_name);
    }
}
