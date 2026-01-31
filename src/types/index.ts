export interface Channel {
  id?: number;
  platform: 'twitch' | 'youtube';
  channel_id: string;
  channel_name: string;
  display_name?: string;
  profile_image_url?: string;
  enabled: boolean;
  poll_interval: number;
  follower_count?: number;
  broadcaster_type?: string;
  view_count?: number;
  created_at?: string;
  updated_at?: string;
}

export interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
}

export interface ChannelWithStats extends Channel {
  is_live: boolean;
  current_viewers?: number;
  current_title?: string;
}

export interface StreamStatsQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
}

export interface ExportQuery {
  channel_id?: number;
  start_time?: string;
  end_time?: string;
  aggregation?: string;
  include_chat?: boolean;
}

export interface ChatMessage {
  id?: number;
  stream_id: number;
  timestamp: string;
  platform: string;
  user_id?: string;
  user_name: string;
  message: string;
  message_type: string;
}

export interface ChatMessagesQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
  limit?: number;
  offset?: number;
}

export interface ChatStats {
  total_messages: number;
  unique_users: number;
  messages_per_minute: number;
  top_users: UserMessageCount[];
  message_types: MessageTypeCount[];
  hourly_distribution: HourlyStats[];
}

export interface UserMessageCount {
  user_name: string;
  message_count: number;
}

export interface MessageTypeCount {
  message_type: string;
  count: number;
}

export interface HourlyStats {
  hour: number;
  message_count: number;
}

export interface ChatStatsQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
}

export interface ChatRateQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
  interval_minutes?: number;
}

export interface ChatRateData {
  timestamp: string;
  message_count: number;
  interval_minutes: number;
}

export interface AggregatedStreamStats {
  timestamp: string;
  interval_minutes: number;
  avg_viewer_count?: number;
  max_viewer_count?: number;
  min_viewer_count?: number;
  chat_rate_avg: number;
  data_points: number;
}

export interface AggregatedChatStats {
  timestamp: string;
  interval_minutes: number;
  message_count: number;
  unique_users: number;
  messages_per_minute: number;
}

export interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

export interface DbInitStatus {
  initialized: boolean;
  message: string;
}

export interface DeviceAuthStatus {
  user_code: string;
  verification_uri: string;
  expires_in: number;
  device_code: string;
  interval: number;
}

export interface CollectorStatus {
  channel_id: number;
  channel_name: string;
  platform: string;
  is_running: boolean;
  last_poll_at?: string;
  last_success_at?: string;
  last_error?: string;
  poll_count: number;
  error_count: number;
}
