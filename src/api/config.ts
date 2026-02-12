import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  OAuthConfigSchema,
  type OAuthConfig,
} from '../schemas';

/**
 * トークンを保存
 */
export const saveToken = async (platform: string, token: string): Promise<void> => {
  await invoke('save_token', { platform, token });
};

/**
 * トークンを削除
 */
export const deleteToken = async (platform: string): Promise<void> => {
  await invoke('delete_token', { platform });
};

/**
 * トークンを検証
 */
export const verifyToken = async (platform: string): Promise<boolean> => {
  const result = await invoke<unknown>('verify_token', { platform });
  return z.boolean().parse(result);
};

/**
 * OAuth設定を取得
 */
export const getOAuthConfig = async (platform: string): Promise<OAuthConfig> => {
  const result = await invoke<unknown>('get_oauth_config', { platform });
  return OAuthConfigSchema.parse(result);
};

/**
 * Twitch Device Code認証を開始
 */
export const startTwitchDeviceAuth = async (): Promise<any> => {
  return await invoke('start_twitch_device_auth');
};

/**
 * Twitch Device Codeトークンをポーリング
 */
export const pollTwitchDeviceToken = async (
  deviceCode: string,
  interval: number,
  clientId: string
): Promise<string> => {
  return await invoke('poll_twitch_device_token', {
    deviceCode,
    interval,
    clientId,
  });
};

/**
 * OAuth設定を保存
 */
export const saveOAuthConfig = async (
  platform: string,
  clientId: string,
  clientSecret?: string
): Promise<void> => {
  await invoke('save_oauth_config', {
    platform,
    clientId,
    clientSecret: clientSecret ?? null,
  });
};

/**
 * OAuth設定を削除
 */
export const deleteOAuthConfig = async (platform: string): Promise<void> => {
  await invoke('delete_oauth_config', { platform });
};

/**
 * OAuth設定が存在するか確認
 */
export const hasOAuthConfig = async (platform: string): Promise<boolean> => {
  const result = await invoke<unknown>('has_oauth_config', { platform });
  return z.boolean().parse(result);
};
