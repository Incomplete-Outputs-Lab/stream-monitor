export interface Channel {
  id?: number;
  platform: 'twitch' | 'youtube';
  channel_id: string; // Twitch: login, YouTube: channel_id
  channel_name: string;
  display_name?: string;
  profile_image_url?: string;
  enabled: boolean;
  poll_interval: number;
  follower_count?: number;
  broadcaster_type?: string;
  view_count?: number;
  is_auto_discovered?: boolean;
  discovered_at?: string;
  twitch_user_id?: number; // Twitchの不変なuser ID（内部識別子）
  created_at?: string;
  updated_at?: string;
}

export interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
  category?: string;
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

export interface SqlQueryResult {
  columns: string[];
  rows: any[][];
  affected_rows?: number;
  execution_time_ms: number;
}

export interface SqlTemplate {
  id?: number;
  name: string;
  description?: string;
  query: string;
  created_at?: string;
  updated_at?: string;
}

export interface SaveTemplateRequest {
  id?: number;
  name: string;
  description?: string;
  query: string;
}

export interface TableInfo {
  table_name: string;
  column_count: number;
}

export interface BroadcasterAnalytics {
  channel_id: number;
  channel_name: string;
  minutes_watched: number;
  hours_broadcasted: number;
  average_ccu: number;
  main_played_title: string | null;
  main_title_mw_percent: number | null;
  peak_ccu: number;
  stream_count: number;
  total_chat_messages: number;
  avg_chat_rate: number;
  unique_chatters: number;
  engagement_rate: number;
  category_count: number;
}

export interface GameAnalytics {
  category: string;
  minutes_watched: number;
  hours_broadcasted: number;
  average_ccu: number;
  unique_broadcasters: number;
  top_channel: string | null;
}

export interface DataAvailability {
  first_record: string;
  last_record: string;
  total_days_with_data: number;
  total_records: number;
}

export interface DailyStats {
  date: string;
  minutes_watched: number;
  hours_broadcasted: number;
  average_ccu: number;
  collection_hours: number;
}

export interface TwitchRateLimitStatus {
  /** 直近1分間で消費したポイント数 */
  points_used: number;
  /** バケット容量（800） */
  bucket_capacity: number;
  /** 推定残りポイント数 */
  points_remaining: number;
  /** 最古エントリが期限切れになるまでの秒数 */
  oldest_entry_expires_in_seconds: number | null;
  /** 使用率（0-100） */
  usage_percent: number;
  /** 直近1分間のリクエスト数 */
  request_count: number;
}

export interface AutoDiscoveryFilters {
  game_ids: string[];
  languages: string[];
  min_viewers?: number;
}

export interface AutoDiscoverySettings {
  enabled: boolean;
  poll_interval: number;
  max_streams: number;
  filters: AutoDiscoveryFilters;
}

export interface DiscoveredStreamInfo {
  id: number;
  twitch_user_id: number;  // 不変なTwitch user ID（内部識別子）
  channel_id: string;       // login（表示用）
  channel_name: string;
  display_name?: string;
  profile_image_url?: string;
  discovered_at?: string;
  title?: string;
  category?: string;
  viewer_count?: number;
  follower_count?: number;
  broadcaster_type?: string;
}

export interface TwitchGame {
  id: string;
  name: string;
  box_art_url: string;
}
