import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { hasToken as hasTokenInStronghold, isVaultUnlocked } from '../utils/stronghold';

interface OAuthConfig {
  client_id: string | null;
  client_secret: string | null;
}

interface ConfigStore {
  hasTwitchToken: boolean;
  hasYouTubeToken: boolean;
  hasTwitchOAuth: boolean;
  hasYouTubeOAuth: boolean;
  loading: boolean;
  error: string | null;
  checkTokens: () => Promise<void>;
  saveToken: (platform: string, token: string) => Promise<void>;
  deleteToken: (platform: string) => Promise<void>;
  verifyToken: (platform: string) => Promise<boolean>;
  getOAuthConfig: (platform: string) => Promise<OAuthConfig>;
  saveOAuthConfig: (platform: string, clientId: string, clientSecret?: string) => Promise<void>;
  deleteOAuthConfig: (platform: string) => Promise<void>;
  hasOAuthConfig: (platform: string) => Promise<boolean>;
  checkOAuthConfigs: () => Promise<void>;
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  hasTwitchToken: false,
  hasYouTubeToken: false,
  hasTwitchOAuth: false,
  hasYouTubeOAuth: false,
  loading: false,
  error: null,

  checkTokens: async () => {
    set({ loading: true });
    try {
      // Check if vault is unlocked to directly query Stronghold
      const vaultUnlocked = isVaultUnlocked();
      
      let hasTwitch = false;
      let hasYouTube = false;
      
      if (vaultUnlocked) {
        // Vault is unlocked - check tokens directly from Stronghold
        console.log('[ConfigStore] Vault is unlocked, checking tokens from Stronghold');
        [hasTwitch, hasYouTube] = await Promise.all([
          hasTokenInStronghold('twitch'),
          hasTokenInStronghold('youtube'),
        ]);
      } else {
        // Vault is locked - fall back to backend check (will return false)
        console.log('[ConfigStore] Vault is locked, falling back to backend check');
        [hasTwitch, hasYouTube] = await Promise.all([
          invoke<boolean>('has_token', { platform: 'twitch' }),
          invoke<boolean>('has_token', { platform: 'youtube' }),
        ]);
      }
      
      const [hasTwitchOAuth, hasYouTubeOAuth] = await Promise.all([
        invoke<boolean>('has_oauth_config', { platform: 'twitch' }),
        invoke<boolean>('has_oauth_config', { platform: 'youtube' }),
      ]);
      
      console.log('[ConfigStore] Token status updated:', {
        hasTwitchToken: hasTwitch,
        hasYouTubeToken: hasYouTube,
        hasTwitchOAuth,
        hasYouTubeOAuth,
      });
      set({
        hasTwitchToken: hasTwitch,
        hasYouTubeToken: hasYouTube,
        hasTwitchOAuth,
        hasYouTubeOAuth,
        loading: false,
      });
    } catch (error) {
      console.error('[ConfigStore] Failed to check tokens:', error);
      set({ error: String(error), loading: false });
    }
  },

  saveToken: async (platform, token) => {
    try {
      await invoke('save_token', { platform, token });
      
      // After saving, check token status from Stronghold
      const vaultUnlocked = isVaultUnlocked();
      if (vaultUnlocked) {
        const hasToken = await hasTokenInStronghold(platform);
        if (platform === 'twitch') {
          set({ hasTwitchToken: hasToken });
        } else if (platform === 'youtube') {
          set({ hasYouTubeToken: hasToken });
        }
      } else {
        // Vault not unlocked, optimistically set to true
        if (platform === 'twitch') {
          set({ hasTwitchToken: true });
        } else if (platform === 'youtube') {
          set({ hasYouTubeToken: true });
        }
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteToken: async (platform) => {
    try {
      await invoke('delete_token', { platform });
      
      // After deleting, check token status from Stronghold
      const vaultUnlocked = isVaultUnlocked();
      if (vaultUnlocked) {
        const hasToken = await hasTokenInStronghold(platform);
        if (platform === 'twitch') {
          set({ hasTwitchToken: hasToken });
        } else if (platform === 'youtube') {
          set({ hasYouTubeToken: hasToken });
        }
      } else {
        // Vault not unlocked, optimistically set to false
        if (platform === 'twitch') {
          set({ hasTwitchToken: false });
        } else if (platform === 'youtube') {
          set({ hasYouTubeToken: false });
        }
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  verifyToken: async (platform) => {
    try {
      return await invoke<boolean>('verify_token', { platform });
    } catch (error) {
      set({ error: String(error) });
      return false;
    }
  },

  getOAuthConfig: async (platform) => {
    try {
      return await invoke<OAuthConfig>('get_oauth_config', { platform });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  saveOAuthConfig: async (platform, clientId, clientSecret) => {
    try {
      await invoke('save_oauth_config', {
        platform,
        clientId,
        clientSecret: clientSecret || null,
      });
      // OAuth設定が存在することを反映
      if (platform === 'twitch') {
        set({ hasTwitchOAuth: true });
      } else if (platform === 'youtube') {
        set({ hasYouTubeOAuth: true });
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteOAuthConfig: async (platform) => {
    try {
      await invoke('delete_oauth_config', { platform });
      // OAuth設定が削除されたことを反映
      if (platform === 'twitch') {
        set({ hasTwitchOAuth: false });
      } else if (platform === 'youtube') {
        set({ hasYouTubeOAuth: false });
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  hasOAuthConfig: async (platform) => {
    try {
      return await invoke<boolean>('has_oauth_config', { platform });
    } catch (error) {
      set({ error: String(error) });
      return false;
    }
  },

  checkOAuthConfigs: async () => {
    try {
      const [hasTwitchOAuth, hasYouTubeOAuth] = await Promise.all([
        get().hasOAuthConfig('twitch'),
        get().hasOAuthConfig('youtube'),
      ]);
      set({
        hasTwitchOAuth,
        hasYouTubeOAuth,
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
