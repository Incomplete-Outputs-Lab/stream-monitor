import { invoke } from '@tauri-apps/api/core';

export interface OAuthConfig {
  client_id: string | null;
  client_secret: string | null;
}

/**
 * トークンを保存
 */
export const saveToken = async (platform: string, token: string): Promise<void> => {
  return await invoke('save_token', { platform, token });
};

/**
 * トークンを削除
 */
export const deleteToken = async (platform: string): Promise<void> => {
  return await invoke('delete_token', { platform });
};

/**
 * トークンを検証
 */
export const verifyToken = async (platform: string): Promise<boolean> => {
  return await invoke<boolean>('verify_token', { platform });
};

/**
 * OAuth設定を取得
 */
export const getOAuthConfig = async (platform: string): Promise<OAuthConfig> => {
  return await invoke<OAuthConfig>('get_oauth_config', { platform });
};

/**
 * OAuth設定を保存
 */
export const saveOAuthConfig = async (
  platform: string,
  clientId: string,
  clientSecret?: string
): Promise<void> => {
  return await invoke('save_oauth_config', {
    platform,
    clientId,
    clientSecret: clientSecret || null,
  });
};

/**
 * OAuth設定を削除
 */
export const deleteOAuthConfig = async (platform: string): Promise<void> => {
  return await invoke('delete_oauth_config', { platform });
};

/**
 * OAuth設定が存在するか確認
 */
export const hasOAuthConfig = async (platform: string): Promise<boolean> => {
  return await invoke<boolean>('has_oauth_config', { platform });
};
