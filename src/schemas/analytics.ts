import { z } from 'zod';

/**
 * Broadcaster analytics schema
 */
export const BroadcasterAnalyticsSchema = z.object({
  channel_id: z.number(),
  channel_name: z.string(),
  login_name: z.string(),
  minutes_watched: z.number(),
  hours_broadcasted: z.number(),
  average_ccu: z.number(),
  main_played_title: z.string(),
  main_title_mw_percent: z.number(),
  peak_ccu: z.number(),
  stream_count: z.number(),
  total_chat_messages: z.number(),
  avg_chat_rate: z.number(),
  unique_chatters: z.number(),
  engagement_rate: z.number(),
  category_count: z.number(),
});

/**
 * Game analytics schema
 */
export const GameAnalyticsSchema = z.object({
  game_id: z.string(),
  category: z.string(),
  minutes_watched: z.number(),
  hours_broadcasted: z.number(),
  average_ccu: z.number(),
  unique_broadcasters: z.number(),
  top_channel: z.string(),
  top_channel_login: z.string(),
  total_chat_messages: z.number(),
  avg_chat_rate: z.number(),
  engagement_rate: z.number(),
});

/**
 * Data availability schema
 */
export const DataAvailabilitySchema = z.object({
  first_record: z.string(),
  last_record: z.string(),
  total_days_with_data: z.number(),
  total_records: z.number(),
});

/**
 * Daily stats schema
 */
export const DailyStatsSchema = z.object({
  date: z.string(),
  minutes_watched: z.number(),
  hours_broadcasted: z.number(),
  average_ccu: z.number(),
  collection_hours: z.number(),
});

// Export types
export type BroadcasterAnalytics = z.infer<typeof BroadcasterAnalyticsSchema>;
export type GameAnalytics = z.infer<typeof GameAnalyticsSchema>;
export type DataAvailability = z.infer<typeof DataAvailabilitySchema>;
export type DailyStats = z.infer<typeof DailyStatsSchema>;
