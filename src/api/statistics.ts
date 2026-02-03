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
} from '../types';

// ========== Broadcaster & Game Analytics ==========

export const getBroadcasterAnalytics = async (params: {
  channelId?: number;
  startTime?: string;
  endTime?: string;
}): Promise<BroadcasterAnalytics[]> => {
  return await invoke<BroadcasterAnalytics[]>('get_broadcaster_analytics', {
    channel_id: params.channelId,
    start_time: params.startTime,
    end_time: params.endTime,
  });
};

export const getGameAnalytics = async (params: {
  category?: string;
  startTime?: string;
  endTime?: string;
}): Promise<GameAnalytics[]> => {
  return await invoke<GameAnalytics[]>('get_game_analytics', {
    category: params.category,
    start_time: params.startTime,
    end_time: params.endTime,
  });
};

export const listGameCategories = async (params: {
  startTime?: string;
  endTime?: string;
}): Promise<string[]> => {
  return await invoke<string[]>('list_game_categories', {
    start_time: params.startTime,
    end_time: params.endTime,
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
    start_time: params.startTime,
    end_time: params.endTime,
  });
};

export const getChannelDailyStats = async (params: {
  channelId: number;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  return await invoke<DailyStats[]>('get_channel_daily_stats', {
    channel_id: params.channelId,
    start_time: params.startTime,
    end_time: params.endTime,
  });
};

// ========== Chat Analytics ==========

export const getChatEngagementTimeline = async (
  query: ChatAnalyticsQuery
): Promise<ChatEngagementStats[]> => {
  return await invoke<ChatEngagementStats[]>('get_chat_engagement_timeline', {
    channel_id: query.channelId,
    stream_id: query.streamId,
    start_time: query.startTime,
    end_time: query.endTime,
    interval_minutes: query.intervalMinutes ?? 5,
  });
};

export const detectChatSpikes = async (
  query: ChatAnalyticsQuery
): Promise<ChatSpike[]> => {
  return await invoke<ChatSpike[]>('detect_chat_spikes', {
    channel_id: query.channelId,
    stream_id: query.streamId,
    start_time: query.startTime,
    end_time: query.endTime,
    min_spike_ratio: query.minSpikeRatio ?? 2.0,
  });
};

export const getUserSegmentStats = async (
  query: ChatAnalyticsQuery
): Promise<UserSegmentStats[]> => {
  return await invoke<UserSegmentStats[]>('get_user_segment_stats', {
    channel_id: query.channelId,
    stream_id: query.streamId,
    start_time: query.startTime,
    end_time: query.endTime,
  });
};

export const getTopChatters = async (
  query: ChatAnalyticsQuery
): Promise<TopChatter[]> => {
  return await invoke<TopChatter[]>('get_top_chatters', {
    channel_id: query.channelId,
    stream_id: query.streamId,
    start_time: query.startTime,
    end_time: query.endTime,
    limit: query.limit ?? 50,
  });
};

export const getTimePatternStats = async (
  query: ChatAnalyticsQuery
): Promise<TimePatternStats[]> => {
  return await invoke<TimePatternStats[]>('get_time_pattern_stats', {
    channel_id: query.channelId,
    start_time: query.startTime,
    end_time: query.endTime,
    group_by_day: query.groupByDay ?? false,
  });
};

export const getChatterBehaviorStats = async (
  query: ChatAnalyticsQuery
): Promise<ChatterBehaviorStats> => {
  return await invoke<ChatterBehaviorStats>('get_chatter_behavior_stats', {
    channel_id: query.channelId,
    start_time: query.startTime,
    end_time: query.endTime,
  });
};
