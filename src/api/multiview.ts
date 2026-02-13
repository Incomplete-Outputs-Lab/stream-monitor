import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import { MultiviewChannelStatsSchema, type MultiviewChannelStats } from '../schemas/multiview';

/**
 * マルチビュー用リアルタイム統計を取得
 */
export const getMultiviewRealtimeStats = async (
  channelIds: number[]
): Promise<MultiviewChannelStats[]> => {
  const result = await invoke<unknown>('get_multiview_realtime_stats', {
    channelIds,
  });
  return z.array(MultiviewChannelStatsSchema).parse(result);
};
