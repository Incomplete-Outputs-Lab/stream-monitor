import { z } from 'zod';

/**
 * Chat message schema
 */
export const ChatMessageSchema = z.object({
  // Rust側では Option<> フィールドが null になる場合があるため、
  // Zod では undefined / null の両方を許容する nullish() を使用する
  id: z.number().nullish(),
  channel_id: z.number().nullish(),
  // stream_id は一部のメッセージで NULL になる場合があるため nullish にする
  stream_id: z.number().nullish(),
  timestamp: z.string(),
  platform: z.string(),
  user_id: z.string().nullish(),
  user_name: z.string(),
  display_name: z.string().nullish(),
  message: z.string(),
  message_type: z.string(),
  badges: z.array(z.string()).nullish(),
  badge_info: z.string().nullish(),
});

/**
 * Chat messages query schema
 */
export const ChatMessagesQuerySchema = z.object({
  stream_id: z.number().optional(),
  channel_id: z.number().optional(),
  start_time: z.string().optional(),
  end_time: z.string().optional(),
  limit: z.number().optional(),
  offset: z.number().optional(),
});

/**
 * Aggregated chat stats schema
 */
export const AggregatedChatStatsSchema = z.object({
  timestamp: z.string(),
  interval_minutes: z.number(),
  message_count: z.number(),
  unique_users: z.number(),
  messages_per_minute: z.number(),
});

/**
 * Chat engagement stats schema
 */
export const ChatEngagementStatsSchema = z.object({
  timestamp: z.string(),
  chatCount: z.number(),
  uniqueChatters: z.number(),
  viewerCount: z.number(),
  engagementRate: z.number(),
});

/**
 * Chat spike schema
 */
export const ChatSpikeSchema = z.object({
  timestamp: z.string(),
  chatCount: z.number(),
  spikeRatio: z.number(),
  prevCount: z.number(),
});

/**
 * User segment enum
 */
export const UserSegmentSchema = z.enum(['subscriber', 'vip', 'moderator', 'broadcaster', 'regular']);

/**
 * User segment stats schema
 */
export const UserSegmentStatsSchema = z.object({
  segment: UserSegmentSchema,
  messageCount: z.number(),
  userCount: z.number(),
  avgMessagesPerUser: z.number(),
  percentage: z.number(),
});

/**
 * Top chatter schema
 */
export const TopChatterSchema = z.object({
  userId: z.string().optional(),
  userName: z.string(),
  displayName: z.string().optional(),
  messageCount: z.number(),
  badges: z.array(z.string()),
  firstSeen: z.string(),
  lastSeen: z.string(),
  streamCount: z.number(),
});

/**
 * Time pattern stats schema
 */
export const TimePatternStatsSchema = z.object({
  hour: z.number(),
  dayOfWeek: z.number().nullish(), // null when groupByDay=false
  avgChatRate: z.number(),
  avgEngagement: z.number(),
  totalMessages: z.number(),
});

/**
 * Chatter behavior stats schema
 */
export const ChatterBehaviorStatsSchema = z.object({
  totalUniqueChatters: z.number(),
  repeaterCount: z.number(),
  newChatterCount: z.number(),
  repeaterPercentage: z.number(),
  avgParticipationRate: z.number(),
});

/**
 * Chat analytics query schema
 */
export const ChatAnalyticsQuerySchema = z.object({
  channelId: z.number().optional(),
  streamId: z.number().optional(),
  startTime: z.string().optional(),
  endTime: z.string().optional(),
  intervalMinutes: z.number().optional(),
  minSpikeRatio: z.number().optional(),
  limit: z.number().optional(),
  groupByDay: z.boolean().optional(),
});

// Export types
export type ChatMessage = z.infer<typeof ChatMessageSchema>;
export type ChatMessagesQuery = z.infer<typeof ChatMessagesQuerySchema>;
export type AggregatedChatStats = z.infer<typeof AggregatedChatStatsSchema>;
export type ChatEngagementStats = z.infer<typeof ChatEngagementStatsSchema>;
export type ChatSpike = z.infer<typeof ChatSpikeSchema>;
export type UserSegment = z.infer<typeof UserSegmentSchema>;
export type UserSegmentStats = z.infer<typeof UserSegmentStatsSchema>;
export type TopChatter = z.infer<typeof TopChatterSchema>;
export type TimePatternStats = z.infer<typeof TimePatternStatsSchema>;
export type ChatterBehaviorStats = z.infer<typeof ChatterBehaviorStatsSchema>;
export type ChatAnalyticsQuery = z.infer<typeof ChatAnalyticsQuerySchema>;
