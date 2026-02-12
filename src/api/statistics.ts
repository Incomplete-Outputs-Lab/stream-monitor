import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  BroadcasterAnalyticsSchema,
  GameAnalyticsSchema,
  DailyStatsSchema,
  DataAvailabilitySchema,
  ChatEngagementStatsSchema,
  ChatSpikeSchema,
  UserSegmentStatsSchema,
  TopChatterSchema,
  TimePatternStatsSchema,
  ChatterBehaviorStatsSchema,
  ChatAnalyticsQuerySchema,
  WordFrequencyResultSchema,
  EmoteAnalysisResultSchema,
  MessageLengthStatsSchema,
  CorrelationResultSchema,
  CategoryImpactResultSchema,
  ChatterScoreResultSchema,
  AnomalyResultSchema,
  ChatMessageSchema,
  type BroadcasterAnalytics,
  type GameAnalytics,
  type DailyStats,
  type DataAvailability,
  type ChatEngagementStats,
  type ChatSpike,
  type UserSegmentStats,
  type TopChatter,
  type TimePatternStats,
  type ChatterBehaviorStats,
  type ChatAnalyticsQuery,
  type WordFrequencyResult,
  type EmoteAnalysisResult,
  type MessageLengthStats,
  type CorrelationResult,
  type CategoryImpactResult,
  type ChatterScoreResult,
  type AnomalyResult,
  type ChatMessage,
} from '../schemas';

// ========== Broadcaster & Game Analytics ==========

export const getBroadcasterAnalytics = async (params: {
  channelId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<BroadcasterAnalytics[]> => {
  const result = await invoke<unknown>('get_broadcaster_analytics', {
    channelId: params.channelId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
  return z.array(BroadcasterAnalyticsSchema).parse(result);
};

export const getGameAnalytics = async (params: {
  category?: string;
  startTime?: string;
  endTime?: string;
}): Promise<GameAnalytics[]> => {
  const result = await invoke<unknown>('get_game_analytics', {
    category: params.category,
    startTime: params.startTime,
    endTime: params.endTime,
  });
  return z.array(GameAnalyticsSchema).parse(result);
};

export const listGameCategories = async (params: {
  startTime?: string;
  endTime?: string;
}): Promise<string[]> => {
  const result = await invoke<unknown>('list_game_categories', {
    startTime: params.startTime,
    endTime: params.endTime,
  });
  return z.array(z.string()).parse(result);
};

export const getDataAvailability = async (): Promise<DataAvailability> => {
  const result = await invoke<unknown>('get_data_availability');
  return DataAvailabilitySchema.parse(result);
};

export const getGameDailyStats = async (params: {
  category: string;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  const result = await invoke<unknown>('get_game_daily_stats', {
    category: params.category,
    startTime: params.startTime,
    endTime: params.endTime,
  });
  return z.array(DailyStatsSchema).parse(result);
};

export const getChannelDailyStats = async (params: {
  channelId: number;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  const result = await invoke<unknown>('get_channel_daily_stats', {
    channelId: params.channelId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
  return z.array(DailyStatsSchema).parse(result);
};

// ========== Chat Analytics ==========

export const getChatEngagementTimeline = async (
  query: ChatAnalyticsQuery
): Promise<ChatEngagementStats[]> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('get_chat_engagement_timeline', {
    channelId: validatedQuery.channelId,
    streamId: validatedQuery.streamId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
    intervalMinutes: validatedQuery.intervalMinutes ?? 5,
  });
  return z.array(ChatEngagementStatsSchema).parse(result);
};

export const detectChatSpikes = async (
  query: ChatAnalyticsQuery
): Promise<ChatSpike[]> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('detect_chat_spikes', {
    channelId: validatedQuery.channelId,
    streamId: validatedQuery.streamId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
    minSpikeRatio: validatedQuery.minSpikeRatio ?? 2.0,
  });
  return z.array(ChatSpikeSchema).parse(result);
};

export const getUserSegmentStats = async (
  query: ChatAnalyticsQuery
): Promise<UserSegmentStats[]> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('get_user_segment_stats', {
    channelId: validatedQuery.channelId,
    streamId: validatedQuery.streamId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
  });
  return z.array(UserSegmentStatsSchema).parse(result);
};

export const getTopChatters = async (
  query: ChatAnalyticsQuery
): Promise<TopChatter[]> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('get_top_chatters', {
    channelId: validatedQuery.channelId,
    streamId: validatedQuery.streamId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
    limit: validatedQuery.limit ?? 50,
  });
  return z.array(TopChatterSchema).parse(result);
};

export const getTimePatternStats = async (
  query: ChatAnalyticsQuery
): Promise<TimePatternStats[]> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('get_time_pattern_stats', {
    channelId: validatedQuery.channelId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
    groupByDay: validatedQuery.groupByDay ?? false,
  });
  return z.array(TimePatternStatsSchema).parse(result);
};

export const getChatterBehaviorStats = async (
  query: ChatAnalyticsQuery
): Promise<ChatterBehaviorStats> => {
  const validatedQuery = ChatAnalyticsQuerySchema.parse(query);
  const result = await invoke<unknown>('get_chatter_behavior_stats', {
    channelId: validatedQuery.channelId,
    startTime: validatedQuery.startTime,
    endTime: validatedQuery.endTime,
  });
  return ChatterBehaviorStatsSchema.parse(result);
};

export const getChatMessages = async (params: {
  streamId?: number;
  channelId?: number;
  startTime?: string;
  endTime?: string;
  limit?: number;
  offset?: number;
}): Promise<ChatMessage[]> => {
  const result = await invoke<unknown>('get_chat_messages', { query: params });
  return z.array(ChatMessageSchema).parse(result);
};

// ========== Data Science APIs ==========

export const getWordFrequency = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  limit?: number;
}): Promise<WordFrequencyResult> => {
  const result = await invoke<unknown>('get_word_frequency', { query: params });
  return WordFrequencyResultSchema.parse(result);
};

export const getEmoteAnalysis = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  limit?: number;
}): Promise<EmoteAnalysisResult> => {
  const result = await invoke<unknown>('get_emote_analysis', { query: params });
  return EmoteAnalysisResultSchema.parse(result);
};

export const getMessageLengthStats = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<MessageLengthStats> => {
  const result = await invoke<unknown>('get_message_length_stats', { query: params });
  return MessageLengthStatsSchema.parse(result);
};

export const getViewerChatCorrelation = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<CorrelationResult> => {
  const result = await invoke<unknown>('get_viewer_chat_correlation', { query: params });
  return CorrelationResultSchema.parse(result);
};

export const getCategoryImpact = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<CategoryImpactResult> => {
  const result = await invoke<unknown>('get_category_impact', { query: params });
  return CategoryImpactResultSchema.parse(result);
};

export const getChatterActivityScores = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  limit?: number;
}): Promise<ChatterScoreResult> => {
  const result = await invoke<unknown>('get_chatter_activity_scores', { query: params });
  return ChatterScoreResultSchema.parse(result);
};

export const detectAnomalies = async (params: {
  channelId?: number;
  streamId?: number;
  startTime?: string;
  endTime?: string;
  zThreshold?: number;
}): Promise<AnomalyResult> => {
  const result = await invoke<unknown>('detect_anomalies', { query: params });
  return AnomalyResultSchema.parse(result);
};

// ========== Real-time Statistics ==========

export const getRealtimeChatRate = async (): Promise<
  { channel_id: number; stream_id: number; chat_rate_1min: number }[]
> => {
  const RealtimeChatRateItemSchema = z.object({
    channel_id: z.number(),
    stream_id: z.number(),
    chat_rate_1min: z.number(),
  });
  const result = await invoke<unknown>('get_realtime_chat_rate');
  return z.array(RealtimeChatRateItemSchema).parse(result);
};

export const getChatMessagesAroundTimestamp = async (params: {
  streamId: number;
  timestamp: string;
  beforeMinutes?: number;
  afterMinutes?: number;
  limit?: number;
}): Promise<ChatMessage[]> => {
  const result = await invoke<unknown>('get_chat_messages_around_timestamp', params);
  return z.array(ChatMessageSchema).parse(result);
};
