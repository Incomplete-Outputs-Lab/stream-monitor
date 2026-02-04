import { invoke } from '@tauri-apps/api/core';
import type { DiscoveredStreamInfo, AutoDiscoverySettings, TwitchGame } from '../types';

/**
 * 自動発見された配信一覧を取得
 */
export const getDiscoveredStreams = async (): Promise<DiscoveredStreamInfo[]> => {
  return await invoke<DiscoveredStreamInfo[]>('get_discovered_streams');
};

/**
 * 自動発見チャンネルを手動登録に昇格
 */
export const promoteDiscoveredChannel = async (channelId: string): Promise<void> => {
  return await invoke('promote_discovered_channel', { channelId });
};

/**
 * 自動発見機能の有効/無効を切り替え
 */
export const toggleAutoDiscovery = async (): Promise<boolean> => {
  return await invoke<boolean>('toggle_auto_discovery');
};

/**
 * 自動発見設定を取得
 */
export const getAutoDiscoverySettings = async (): Promise<AutoDiscoverySettings | null> => {
  return await invoke<AutoDiscoverySettings | null>('get_auto_discovery_settings');
};

/**
 * 自動発見設定を保存
 */
export const saveAutoDiscoverySettings = async (settings: AutoDiscoverySettings): Promise<void> => {
  return await invoke('save_auto_discovery_settings', { settings });
};

/**
 * Twitchゲームを検索
 */
export const searchTwitchGames = async (query: string): Promise<TwitchGame[]> => {
  return await invoke<TwitchGame[]>('search_twitch_games', { query });
};

/**
 * ゲームIDからゲーム情報を取得
 */
export const getGamesByIds = async (gameIds: string[]): Promise<TwitchGame[]> => {
  return await invoke<TwitchGame[]>('get_games_by_ids', { gameIds });
};
