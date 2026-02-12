import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  ChannelWithStatsSchema,
  ChannelSchema,
  AddChannelRequestSchema,
  UpdateChannelRequestSchema,
  type ChannelWithStats,
  type Channel,
  type AddChannelRequest,
  type UpdateChannelRequest,
} from '../schemas';

/**
 * チャンネル一覧を取得
 */
export const listChannels = async (): Promise<ChannelWithStats[]> => {
  const result = await invoke<unknown>('list_channels');
  return z.array(ChannelWithStatsSchema).parse(result);
};

/**
 * 軽量なチャンネル一覧を取得（Twitch API なし）
 *
 * - DBの `channels` テーブルのみを参照
 * - タイムラインなど、配信者リストだけ必要な画面で利用
 */
export const listChannelsBasic = async (): Promise<Channel[]> => {
  const result = await invoke<unknown>('list_channels_basic');
  return z.array(ChannelSchema).parse(result);
};

/**
 * チャンネルを追加
 */
export const addChannel = async (request: AddChannelRequest): Promise<Channel> => {
  const validatedRequest = AddChannelRequestSchema.parse(request);
  const result = await invoke<unknown>('add_channel', { request: validatedRequest });
  return ChannelSchema.parse(result);
};

/**
 * チャンネルを削除
 */
export const removeChannel = async (id: number): Promise<void> => {
  await invoke('remove_channel', { id });
};

/**
 * チャンネル情報を更新
 */
export const updateChannel = async (request: UpdateChannelRequest): Promise<Channel> => {
  const validatedRequest = UpdateChannelRequestSchema.parse(request);
  const result = await invoke<unknown>('update_channel', {
    id: validatedRequest.id,
    channel_name: validatedRequest.channel_name,
    poll_interval: validatedRequest.poll_interval,
    enabled: validatedRequest.enabled,
  });
  return ChannelSchema.parse(result);
};

/**
 * チャンネルの有効/無効を切り替え
 */
export const toggleChannel = async (id: number): Promise<Channel> => {
  const result = await invoke<unknown>('toggle_channel', { id });
  return ChannelSchema.parse(result);
};
