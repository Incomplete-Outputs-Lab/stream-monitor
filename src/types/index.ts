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
  delimiter?: string;
}

export interface ChatMessage {
  id?: number;
  channel_id?: number;
  stream_id: number;
  timestamp: string;
  platform: string;
  user_id?: string;
  user_name: string;
  display_name?: string; // Twitch表示名
  message: string;
  message_type: string;
  badges?: string[];
  badge_info?: string;
}

export interface ChatMessagesQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
  limit?: number;
  offset?: number;
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
  login_name: string;
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
  top_channel_login: string | null;
  total_chat_messages: number;
  avg_chat_rate: number;
  engagement_rate: number;
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

export interface TwitchGame {
  id: string;
  name: string;
  box_art_url: string;
}

export interface SelectedGame {
  id: string;
  name: string;
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

export interface StreamInfo {
  id: number;
  stream_id: string;
  channel_id: number;
  channel_name: string;
  title: string;
  category: string;
  started_at: string;
  ended_at?: string;
  peak_viewers: number;
  avg_viewers: number;
  duration_minutes: number;
  minutes_watched: number;
  follower_gain: number;
  total_chat_messages: number;
  engagement_rate: number;
  last_collected_at?: string;
}

export interface TimelinePoint {
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
  category?: string;
  title?: string;
  follower_count?: number;
}

export interface CategoryChange {
  timestamp: string;
  from_category?: string;
  to_category: string;
}

export interface TitleChange {
  timestamp: string;
  from_title?: string;
  to_title: string;
}

export interface StreamTimelineData {
  stream_info: StreamInfo;
  stats: TimelinePoint[];
  category_changes: CategoryChange[];
  title_changes: TitleChange[];
}

// ========== Timeline Comparison Types ==========

// 比較用に正規化されたデータポイント
export interface NormalizedTimelinePoint {
  timestamp: string; // ISO8601形式の絶対時刻
  timestampMs: number; // ミリ秒のUnix timestamp
  viewer_count: number;
  chat_rate_1min: number;
  streamId: number; // どの配信のデータか識別
  streamLabel: string; // 配信者名と配信タイトル
}

// 比較用イベントマーカー
export interface ComparisonEvent {
  timestamp: string; // ISO8601形式の絶対時刻
  timestampMs: number; // ミリ秒のUnix timestamp
  eventType: 'category' | 'title';
  streamId: number;
  streamLabel: string;
  description: string;
  color: string; // イベントマーカーの色
}

// 複数配信の選択状態
export interface SelectedStream {
  streamId: number;
  channelName: string;
  streamTitle: string;
  startedAt: string;
  color: string; // グラフ表示用の色
}

// ========== Chat Analytics Types ==========

export interface ChatEngagementStats {
  timestamp: string;
  chatCount: number;
  uniqueChatters: number;
  viewerCount: number;
  engagementRate: number;
}

export interface ChatSpike {
  timestamp: string;
  chatCount: number;
  spikeRatio: number;
  prevCount: number;
}

export interface UserSegmentStats {
  segment: 'subscriber' | 'vip' | 'moderator' | 'broadcaster' | 'regular';
  messageCount: number;
  userCount: number;
  avgMessagesPerUser: number;
  percentage: number;
}

export interface TopChatter {
  userId?: string;          // Twitch user_id（プライマリ識別子）
  userName: string;         // Twitchログイン名
  displayName?: string;     // Twitch表示名
  messageCount: number;
  badges: string[];
  firstSeen: string;
  lastSeen: string;
  streamCount: number;
}

export interface TimePatternStats {
  hour: number;
  dayOfWeek?: number;
  avgChatRate: number;
  avgEngagement: number;
  totalMessages: number;
}

export interface ChatterBehaviorStats {
  totalUniqueChatters: number;
  repeaterCount: number;
  newChatterCount: number;
  repeaterPercentage: number;
  avgParticipationRate: number;
}

export interface ChatAnalyticsQuery {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  intervalMinutes?: number;
  minSpikeRatio?: number;
  limit?: number;
  groupByDay?: boolean;
}

// ========== Data Science Types ==========

// Phase 1: Text Analysis

export interface WordFrequency {
  word: string;
  count: number;
  percentage: number;
}

export interface WordFrequencyResult {
  words: WordFrequency[];
  totalWords: number;
  uniqueWords: number;
  avgWordsPerMessage: number;
  totalMessages: number;
}

export interface EmoteUsage {
  name: string;
  count: number;
  users: number;
  percentage: number;
}

export interface HourlyEmotePattern {
  hour: number;
  count: number;
}

export interface EmoteAnalysisResult {
  emotes: EmoteUsage[];
  totalEmoteUses: number;
  emotePerMessageRate: number;
  hourlyPattern: HourlyEmotePattern[];
}

export interface LengthDistribution {
  bucket: string; // "0-10", "11-20", etc.
  count: number;
  percentage: number;
}

export interface SegmentLengthStats {
  segment: string;
  avgLength: number;
  messageCount: number;
}

export interface MessageLengthStats {
  avgLength: number;
  medianLength: number;
  stdDev: number;
  minLength: number;
  maxLength: number;
  distribution: LengthDistribution[];
  bySegment: SegmentLengthStats[];
}

// Phase 2: Correlation Analysis

export interface ScatterPoint {
  viewers: number;
  chats: number;
  timestamp: string;
}

export interface HourlyCorrelation {
  hour: number;
  correlation: number;
  sampleCount: number;
}

export interface CorrelationResult {
  pearsonCoefficient: number;
  interpretation: string;
  scatterData: ScatterPoint[];
  hourlyCorrelation: HourlyCorrelation[];
}

export interface CategoryChange {
  timestamp: string;
  fromCategory: string;
  toCategory: string;
  beforeViewers: number;
  afterViewers: number;
  viewerChangePercent: number;
  chatChangePercent: number;
}

export interface CategoryPerformance {
  category: string;
  avgViewers: number;
  avgChatRate: number;
  totalTimeMinutes: number;
  changeCount: number;
}

export interface CategoryImpactResult {
  changes: CategoryChange[];
  categoryPerformance: CategoryPerformance[];
}

// Phase 3: User Behavior Analysis

export interface ChatterActivityScore {
  userName: string;
  score: number;
  messageCount: number;
  streamCount: number;
  badges: string[];
  rank: number;
}

export interface ScoreDistribution {
  scoreRange: string;
  userCount: number;
  percentage: number;
}

export interface SegmentAvgScore {
  segment: string;
  avgScore: number;
  userCount: number;
}

export interface ChatterScoreResult {
  scores: ChatterActivityScore[];
  scoreDistribution: ScoreDistribution[];
  segmentAvgScores: SegmentAvgScore[];
}


// Phase 4: Anomaly Detection

export interface Anomaly {
  timestamp: string;
  value: number;
  previousValue: number;
  changeAmount: number;
  changeRate: number;
  modifiedZScore: number;
  isPositive: boolean;
  minutesFromStreamStart?: number;
  streamPhase: string;
  streamId?: number;
}

export interface TrendStats {
  viewerTrend: string;
  viewerMedian: number;
  viewerMad: number;
  viewerAvg: number;
  viewerStdDev: number;
  chatTrend: string;
  chatMedian: number;
  chatMad: number;
  chatAvg: number;
  chatStdDev: number;
}

export interface AnomalyResult {
  viewerAnomalies: Anomaly[];
  chatAnomalies: Anomaly[];
  trendStats: TrendStats;
}

// Data Science Query Parameters

export interface DataScienceQuery {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  limit?: number;
  zThreshold?: number;
}

// Game Category

export interface GameCategory {
  gameId: string;         // Twitch game ID（プライマリキー）
  gameName: string;       // カテゴリ名（表示用）
  boxArtUrl?: string;     // ボックスアート画像URL
  lastUpdated?: string;   // 最終更新日時
}

export interface UpsertGameCategoryRequest {
  gameId: string;
  gameName: string;
  boxArtUrl?: string;
}
