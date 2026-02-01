import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';

interface BroadcasterAnalyticsFilters {
  channelId?: number;
  gameId?: string;
}

interface GameAnalyticsFilters {
  category?: string;
}

/**
 * 配信者別統計を取得するフック
 */
export function useBroadcasterAnalytics(
  startDate: string,
  endDate: string,
  filters?: BroadcasterAnalyticsFilters
) {
  return useQuery({
    queryKey: ['broadcasterAnalytics', startDate, endDate, filters],
    queryFn: () =>
      invoke('get_broadcaster_analytics', {
        channelId: filters?.channelId,
        startTime: startDate,
        endTime: endDate,
      }),
  });
}

/**
 * ゲーム別統計を取得するフック
 */
export function useGameAnalytics(
  startDate: string,
  endDate: string,
  filters?: GameAnalyticsFilters
) {
  return useQuery({
    queryKey: ['gameAnalytics', startDate, endDate, filters],
    queryFn: () =>
      invoke('get_game_analytics', {
        category: filters?.category,
        startTime: startDate,
        endTime: endDate,
      }),
  });
}

/**
 * ゲーム別日次統計を取得するフック
 */
export function useGameDailyStats(
  category: string,
  startDate: string,
  endDate: string
) {
  return useQuery({
    queryKey: ['gameDailyStats', category, startDate, endDate],
    queryFn: () =>
      invoke('get_game_daily_stats', {
        category,
        startTime: startDate,
        endTime: endDate,
      }),
  });
}

/**
 * チャンネル別日次統計を取得するフック
 */
export function useChannelDailyStats(
  channelId: number,
  startDate: string,
  endDate: string
) {
  return useQuery({
    queryKey: ['channelDailyStats', channelId, startDate, endDate],
    queryFn: () =>
      invoke('get_channel_daily_stats', {
        channelId,
        startTime: startDate,
        endTime: endDate,
      }),
  });
}

/**
 * データ可用性情報を取得するフック
 */
export function useDataAvailability() {
  return useQuery({
    queryKey: ['dataAvailability'],
    queryFn: () => invoke('get_data_availability'),
  });
}
