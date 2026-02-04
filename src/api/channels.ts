import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../types';

export interface AddChannelRequest {
  platform: string;
  channel_id: string;
  channel_name: string;
  poll_interval: number;
}

export interface UpdateChannelRequest {
  id: number;
  channel_name?: string;
  poll_interval?: number;
  enabled?: boolean;
}

/**
 * チャンネル一覧を取得
 */
export const listChannels = async (): Promise<Channel[]> => {
  return await invoke<Channel[]>('list_channels');
};

/**
 * チャンネルを追加
 */
export const addChannel = async (request: AddChannelRequest): Promise<Channel> => {
  return await invoke<Channel>('add_channel', { request });
};

/**
 * チャンネルを削除
 */
export const removeChannel = async (id: number): Promise<void> => {
  return await invoke('remove_channel', { id });
};

/**
 * チャンネル情報を更新
 */
export const updateChannel = async (request: UpdateChannelRequest): Promise<Channel> => {
  return await invoke<Channel>('update_channel', request);
};

/**
 * チャンネルの有効/無効を切り替え
 */
export const toggleChannel = async (id: number): Promise<Channel> => {
  return await invoke<Channel>('toggle_channel', { id });
};
