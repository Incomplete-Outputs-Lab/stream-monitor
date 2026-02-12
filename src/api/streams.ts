import { invoke } from '@tauri-apps/api/core';
import { StreamInfoSchema, StreamTimelineDataSchema } from '../schemas';
import type { StreamInfo, StreamTimelineData } from '../types';

/**
 * チャンネルの配信一覧を取得
 */
export const getChannelStreams = async (params: {
  channel_id: number;
  limit?: number;
  offset?: number;
}): Promise<StreamInfo[]> => {
  const result = await invoke<unknown>('get_channel_streams', {
    channelId: params.channel_id,
    limit: params.limit ?? 50,
    offset: params.offset ?? 0,
  });
  return Array.isArray(result)
    ? result.map((r) => StreamInfoSchema.parse(r))
    : [];
};

/**
 * 配信のタイムラインデータを取得
 */
export const getStreamTimeline = async (
  streamId: number
): Promise<StreamTimelineData> => {
  const result = await invoke<unknown>('get_stream_timeline', {
    streamId,
  });
  return StreamTimelineDataSchema.parse(result);
};

/**
 * 日付範囲で配信一覧を取得（全チャンネル・カレンダー用）
 * dateFrom / dateTo は "YYYY-MM-DD" 形式
 */
export const getStreamsByDateRange = async (params: {
  date_from: string;
  date_to: string;
  limit?: number;
  offset?: number;
}): Promise<StreamInfo[]> => {
  const result = await invoke<unknown>('get_streams_by_date_range', {
    dateFrom: params.date_from,
    dateTo: params.date_to,
    limit: params.limit ?? 100,
    offset: params.offset ?? 0,
  });
  return Array.isArray(result)
    ? result.map((r) => StreamInfoSchema.parse(r))
    : [];
};

/**
 * 比較用：基準配信と時間帯・カテゴリが近い配信をサジェスト（全チャンネル）
 */
export const getSuggestedStreamsForComparison = async (params: {
  base_stream_id: number;
  limit?: number;
}): Promise<StreamInfo[]> => {
  const result = await invoke<unknown>('get_suggested_streams_for_comparison', {
    baseStreamId: params.base_stream_id,
    limit: params.limit ?? 50,
  });
  return Array.isArray(result)
    ? result.map((r) => StreamInfoSchema.parse(r))
    : [];
};
