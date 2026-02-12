import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  DiscoveredStreamInfoSchema,
  AutoDiscoverySettingsSchema,
  TwitchGameSchema,
  type DiscoveredStreamInfo,
  type AutoDiscoverySettings,
  type TwitchGame,
} from '../schemas';

/**
 * 自動発見された配信一覧を取得
 */
export const getDiscoveredStreams = async (): Promise<DiscoveredStreamInfo[]> => {
  const result = await invoke<unknown>('get_discovered_streams');
  return z.array(DiscoveredStreamInfoSchema).parse(result);
};

/**
 * 自動発見チャンネルを手動登録に昇格（単一）
 */
export const promoteDiscoveredChannel = async (channelId: string): Promise<void> => {
  await invoke('promote_discovered_channel', { channelId });
};

/**
 * 自動発見チャンネルを手動登録に昇格（複数一括）
 * @returns 昇格に成功したチャンネルIDのリスト
 */
export const promoteDiscoveredChannels = async (
  channelIds: string[]
): Promise<string[]> => {
  const result = await invoke<unknown>('promote_discovered_channels', {
    channelIds,
  });
  return z.array(z.string()).parse(result);
};

/**
 * 自動発見機能の有効/無効を切り替え
 */
export const toggleAutoDiscovery = async (): Promise<boolean> => {
  const result = await invoke<unknown>('toggle_auto_discovery');
  return z.boolean().parse(result);
};

/**
 * 自動発見設定を取得
 */
export const getAutoDiscoverySettings = async (): Promise<AutoDiscoverySettings | null> => {
  const result = await invoke<unknown>('get_auto_discovery_settings');
  return result === null ? null : AutoDiscoverySettingsSchema.parse(result);
};

/**
 * 自動発見設定を保存
 */
export const saveAutoDiscoverySettings = async (settings: AutoDiscoverySettings): Promise<void> => {
  const validatedSettings = AutoDiscoverySettingsSchema.parse(settings);
  await invoke('save_auto_discovery_settings', { settings: validatedSettings });
};

/**
 * Twitchゲームを検索
 */
export const searchTwitchGames = async (query: string): Promise<TwitchGame[]> => {
  const result = await invoke<unknown>('search_twitch_games', { query });
  return z.array(TwitchGameSchema).parse(result);
};

/**
 * ゲームIDからゲーム情報を取得
 */
export const getGamesByIds = async (gameIds: string[]): Promise<TwitchGame[]> => {
  const result = await invoke<unknown>('get_games_by_ids', { gameIds });
  return z.array(TwitchGameSchema).parse(result);
};
