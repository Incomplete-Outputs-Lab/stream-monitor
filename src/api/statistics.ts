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
  return await invoke<BroadcasterAnalytics[]>('get_broadcaster_analytics', params);
};

export const getGameAnalytics = async (params: {
  category?: string;
  startTime?: string;
  endTime?: string;
}): Promise<GameAnalytics[]> => {
  return await invoke<GameAnalytics[]>('get_game_analytics', params);
};

export const listGameCategories = async (params: {
  startTime?: string;
  endTime?: string;
}): Promise<string[]> => {
  return await invoke<string[]>('list_game_categories', params);
};

export const getDataAvailability = async (): Promise<DataAvailability> => {
  return await invoke<DataAvailability>('get_data_availability');
};

export const getGameDailyStats = async (params: {
  category: string;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  return await invoke<DailyStats[]>('get_game_daily_stats', params);
};

export const getChannelDailyStats = async (params: {
  channelId: number;
  startTime: string;
  endTime: string;
}): Promise<DailyStats[]> => {
  return await invoke<DailyStats[]>('get_channel_daily_stats', params);
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
    channelId: query.channelId,
    streamId: query.streamId,
    startTime: query.startTime,
    endTime: query.endTime,
    minSpikeRatio: query.minSpikeRatio ?? 2.0,
  });
};

export const getUserSegmentStats = async (
  query: ChatAnalyticsQuery
): Promise<UserSegmentStats[]> => {
  return await invoke<UserSegmentStats[]>('get_user_segment_stats', {
    channelId: query.channelId,
    streamId: query.streamId,
    startTime: query.startTime,
    endTime: query.endTime,
  });
};

export const getTopChatters = async (
  query: ChatAnalyticsQuery
): Promise<TopChatter[]> => {
  return await invoke<TopChatter[]>('get_top_chatters', {
    channelId: query.channelId,
    streamId: query.streamId,
    startTime: query.startTime,
    endTime: query.endTime,
    limit: query.limit ?? 50,
  });
};

export const getTimePatternStats = async (
  query: ChatAnalyticsQuery
): Promise<TimePatternStats[]> => {
  return await invoke<TimePatternStats[]>('get_time_pattern_stats', {
    channelId: query.channelId,
    startTime: query.startTime,
    endTime: query.endTime,
    groupByDay: query.groupByDay ?? false,
  });
};

export const getChatterBehaviorStats = async (
  query: ChatAnalyticsQuery
): Promise<ChatterBehaviorStats> => {
  return await invoke<ChatterBehaviorStats>('get_chatter_behavior_stats', {
    channelId: query.channelId,
    startTime: query.startTime,
    endTime: query.endTime,
  });
};
