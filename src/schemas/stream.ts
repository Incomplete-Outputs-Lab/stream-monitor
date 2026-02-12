import { z } from 'zod';

/**
 * Stream stats schema
 */
export const StreamStatsSchema = z.object({
  id: z.number().optional(),
  stream_id: z.number(),
  collected_at: z.string(),
  viewer_count: z.number().optional(),
  chat_rate_1min: z.number(),
  category: z.string().optional(),
});

/**
 * Stream stats query schema
 */
export const StreamStatsQuerySchema = z.object({
  stream_id: z.number().optional(),
  channel_id: z.number().optional(),
  start_time: z.string().optional(),
  end_time: z.string().optional(),
});

/**
 * Aggregated stream stats schema
 */
export const AggregatedStreamStatsSchema = z.object({
  timestamp: z.string(),
  interval_minutes: z.number(),
  avg_viewer_count: z.number().optional(),
  max_viewer_count: z.number().optional(),
  min_viewer_count: z.number().optional(),
  chat_rate_avg: z.number(),
  data_points: z.number(),
});

/**
 * Stream info schema
 */
export const StreamInfoSchema = z.object({
  id: z.number(),
  stream_id: z.string(),
  channel_id: z.number(),
  channel_name: z.string(),
  platform: z.enum(['twitch', 'youtube']).optional(),
  title: z.string(),
  category: z.string(),
  started_at: z.string(),
  ended_at: z.string(),
  peak_viewers: z.number(),
  avg_viewers: z.number(),
  duration_minutes: z.number(),
  minutes_watched: z.number(),
  follower_gain: z.number(),
  total_chat_messages: z.number(),
  engagement_rate: z.number(),
  last_collected_at: z.string(),
});

/**
 * Timeline point schema
 */
export const TimelinePointSchema = z.object({
  collected_at: z.string(),
  viewer_count: z.number(),
  chat_rate_1min: z.number(),
  category: z.string(),
  title: z.string(),
  follower_count: z.number(),
});

/**
 * Category change schema
 */
export const CategoryChangeSchema = z.object({
  timestamp: z.string(),
  from_category: z.string(),
  to_category: z.string(),
});

/**
 * Title change schema
 */
export const TitleChangeSchema = z.object({
  timestamp: z.string(),
  from_title: z.string(),
  to_title: z.string(),
});

/**
 * Stream timeline data schema
 */
export const StreamTimelineDataSchema = z.object({
  stream_info: StreamInfoSchema,
  stats: z.array(TimelinePointSchema),
  category_changes: z.array(CategoryChangeSchema),
  title_changes: z.array(TitleChangeSchema),
});

/**
 * Normalized timeline point schema (for comparison)
 */
export const NormalizedTimelinePointSchema = z.object({
  timestamp: z.string(),
  timestampMs: z.number(),
  viewer_count: z.number(),
  chat_rate_1min: z.number(),
  streamId: z.number(),
  streamLabel: z.string(),
});

/**
 * Comparison event schema
 */
export const ComparisonEventSchema = z.object({
  timestamp: z.string(),
  timestampMs: z.number(),
  eventType: z.enum(['category', 'title']),
  streamId: z.number(),
  streamLabel: z.string(),
  description: z.string(),
  color: z.string(),
});

/**
 * Selected stream schema
 */
export const SelectedStreamSchema = z.object({
  streamId: z.number(),
  channelName: z.string(),
  streamTitle: z.string(),
  startedAt: z.string(),
  color: z.string(),
});

// Export types
export type StreamStats = z.infer<typeof StreamStatsSchema>;
export type StreamStatsQuery = z.infer<typeof StreamStatsQuerySchema>;
export type AggregatedStreamStats = z.infer<typeof AggregatedStreamStatsSchema>;
export type StreamInfo = z.infer<typeof StreamInfoSchema>;
export type TimelinePoint = z.infer<typeof TimelinePointSchema>;
export type CategoryChange = z.infer<typeof CategoryChangeSchema>;
export type TitleChange = z.infer<typeof TitleChangeSchema>;
export type StreamTimelineData = z.infer<typeof StreamTimelineDataSchema>;
export type NormalizedTimelinePoint = z.infer<typeof NormalizedTimelinePointSchema>;
export type ComparisonEvent = z.infer<typeof ComparisonEventSchema>;
export type SelectedStream = z.infer<typeof SelectedStreamSchema>;
