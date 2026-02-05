import { invoke } from '@tauri-apps/api/core';
import type {
  BroadcasterAnalytics,
  GameAnalytics,
  DailyStats,
  DataAvailability,
  ChatEngagementStats,
  ChatSpike,
  UserSegmentStats,
  TopChatter,
  TimePatternStats,
  ChatterBehaviorStats,
  ChatAnalyticsQuery,
  WordFrequencyResult,
  EmoteAnalysisResult,
  MessageLengthStats,
  CorrelationResult,
  CategoryImpactResult,
  ChatterScoreResult,
  AnomalyResult,
  ChatMessage,
} from '../types';

// ========== Broadcaster & Game Analytics ==========

export const getBroadcasterAnalytics = async (params: {
  channelId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<BroadcasterAnalytics[]> => {
  return await invoke<BroadcasterAnalytics[]>('get_broadcaster_analytics', {
    channelId: params.channelId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getGameAnalytics = async (params: {
  category?: string;
  startTime?: string;
  endTime?: string;
}): Promise<GameAnalytics[]> => {
  return await invoke<GameAnalytics[]>('get_game_analytics', {
    category: params.category,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const listGameCategories = async (params: {
  startTime?: string;
  endTime?: string;
}): Promise<string[]> => {
  return await invoke<string[]>('list_game_categories', {
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getDataAvailability = async (): Promise<DataAvailability> => {
  return await invoke<DataAvailability>('get_data_availability');
};

export const getGameDailyStats = async (params: {
  category: string;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  return await invoke<DailyStats[]>('get_game_daily_stats', {
    category: params.category,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getChannelDailyStats = async (params: {
  channelId: number;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  return await invoke<DailyStats[]>('get_channel_daily_stats', {
    channelId:params.channelId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

// ========== Chat Analytics ==========

export const getChatEngagementTimeline = async (
  query: ChatAnalyticsQuery
): Promise<ChatEngagementStats[]> => {
  return await invoke<ChatEngagementStats[]>('get_chat_engagement_timeline', {
    channelId: query.channelId,
    streamId: query.streamId,
    startTime: query.startTime,
    endTime: query.endTime,
    intervalMinutes: query.intervalMinutes ?? 5,
  });
};

export const detectChatSpikes = async (
  query: ChatAnalyticsQuery
): Promise<ChatSpike[]> => {
  return await invoke<ChatSpike[]>('detect_chat_spikes', {
    channelId:query.channelId,
    streamId:query.streamId,
    startTime:query.startTime,
    endTime:query.endTime,
    minSpikeRatio:query.minSpikeRatio ?? 2.0,
  });
};

export const getUserSegmentStats = async (
  query: ChatAnalyticsQuery
): Promise<UserSegmentStats[]> => {
  return await invoke<UserSegmentStats[]>('get_user_segment_stats', {
    channelId:query.channelId,
    streamId:query.streamId,
    startTime:query.startTime,
    endTime:query.endTime,
  });
};

export const getTopChatters = async (
  query: ChatAnalyticsQuery
): Promise<TopChatter[]> => {
  return await invoke<TopChatter[]>('get_top_chatters', {
    channelId:query.channelId,
    streamId:query.streamId,
    startTime:query.startTime,
    endTime:query.endTime,
    limit: query.limit ?? 50,
  });
};

export const getTimePatternStats = async (
  query: ChatAnalyticsQuery
): Promise<TimePatternStats[]> => {
  return await invoke<TimePatternStats[]>('get_time_pattern_stats', {
    channelId:query.channelId,
    startTime:query.startTime,
    endTime:query.endTime,
    groupByDay:query.groupByDay ?? false,
  });
};

export const getChatterBehaviorStats = async (
  query: ChatAnalyticsQuery
): Promise<ChatterBehaviorStats> => {
  return await invoke<ChatterBehaviorStats>('get_chatter_behavior_stats', {
    channelId:query.channelId,
    startTime:query.startTime,
    endTime:query.endTime,
  });
};

// ========== Data Science Analytics ==========

export const getWordFrequencyAnalysis = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
  limit?: number;
}): Promise<WordFrequencyResult> => {
  return await invoke<WordFrequencyResult>('get_word_frequency_analysis', {
    channelId: params.channelId,
    streamId: params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
    limit: params.limit,
  });
};

export const getEmoteAnalysis = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
}): Promise<EmoteAnalysisResult> => {
  return await invoke<EmoteAnalysisResult>('get_emote_analysis', {
    channelId:params.channelId,
    streamId:params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getMessageLengthStats = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
}): Promise<MessageLengthStats> => {
  return await invoke<MessageLengthStats>('get_message_length_stats', {
    channelId:params.channelId,
    streamId:params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getViewerChatCorrelation = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
}): Promise<CorrelationResult> => {
  return await invoke<CorrelationResult>('get_viewer_chat_correlation', {
    channelId:params.channelId,
    streamId:params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getCategoryChangeImpact = async (params: {
  channelId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<CategoryImpactResult> => {
  return await invoke<CategoryImpactResult>('get_category_change_impact', {
    channelId:params.channelId,
    startTime: params.startTime,
    endTime: params.endTime,
  });
};

export const getChatterActivityScores = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
  limit?: number;
}): Promise<ChatterScoreResult> => {
  return await invoke<ChatterScoreResult>('get_chatter_activity_scores', {
    channelId:params.channelId,
    streamId:params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
    limit: params.limit,
  });
};

export const detectAnomalies = async (params: {
  channelId?: number | null;
  streamId?: number | null;
  startTime?: string;
  endTime?: string;
  zThreshold?: number;
}): Promise<AnomalyResult> => {
  return await invoke<AnomalyResult>('detect_anomalies', {
    channelId:params.channelId,
    streamId:params.streamId,
    startTime: params.startTime,
    endTime: params.endTime,
    zThreshold:params.zThreshold,
  });
};

// ========== Anomaly Chat Messages ==========

export const getChatMessagesAroundTimestamp = async (params: {
  streamId: number;
  timestamp: string;
  windowMinutes?: number;
}): Promise<ChatMessage[]> => {
  return await invoke<ChatMessage[]>('get_chat_messages_around_timestamp', {
    query: {
      streamId: params.streamId,
      timestamp: params.timestamp,
      windowMinutes: params.windowMinutes ?? 2,
    }
  });
};

// ========== Realtime Stats ==========

export const getRealtimeChatRate = async (): Promise<number> => {
  return await invoke<number>('get_realtime_chat_rate');
};
