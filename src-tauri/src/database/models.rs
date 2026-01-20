use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Option<i64>,
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub poll_interval: i32,
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
    pub chat_rate_1min: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelWithStats {
    #[serde(flatten)]
    pub channel: Channel,
    pub is_live: bool,
    pub current_viewers: Option<i32>,
    pub current_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub stream_id: i64,
    pub timestamp: String,
    pub platform: String,
    pub user_id: Option<String>,
    pub user_name: String,
    pub message: String,
    pub message_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_serialization() {
        let channel = Channel {
            id: Some(1),
            platform: "twitch".to_string(),
            channel_id: "test_channel".to_string(),
            channel_name: "Test Channel".to_string(),
            display_name: None,
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
    }

    #[test]
    fn test_stream_stats_serialization() {
        let stats = StreamStats {
            id: Some(1),
            stream_id: 1,
            collected_at: "2024-01-01T00:00:00Z".to_string(),
            viewer_count: Some(100),
            chat_rate_1min: 10,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: StreamStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.viewer_count, deserialized.viewer_count);
        assert_eq!(stats.chat_rate_1min, deserialized.chat_rate_1min);
    }
}
