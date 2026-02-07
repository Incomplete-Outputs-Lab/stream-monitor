import { create } from 'zustand';
import { hasToken } from '../utils/keyring';
import * as configApi from '../api/config';

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
      // Check tokens directly from Keyring
      console.log('[ConfigStore] Checking tokens from Keyring');
      const [hasTwitch, hasYouTube] = await Promise.all([
        hasToken('twitch'),
        hasToken('youtube'),
      ]);
      
      const [hasTwitchOAuth, hasYouTubeOAuth] = await Promise.all([
        configApi.hasOAuthConfig('twitch'),
        configApi.hasOAuthConfig('youtube'),
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
      await configApi.saveToken(platform, token);

      // After saving, check token status from Keyring
      const tokenExists = await hasToken(platform);
      if (platform === 'twitch') {
        set({ hasTwitchToken: tokenExists });
      } else if (platform === 'youtube') {
        set({ hasYouTubeToken: tokenExists });
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteToken: async (platform) => {
    try {
      await configApi.deleteToken(platform);

      // After deleting, check token status from Keyring
      const tokenExists = await hasToken(platform);
      if (platform === 'twitch') {
        set({ hasTwitchToken: tokenExists });
      } else if (platform === 'youtube') {
        set({ hasYouTubeToken: tokenExists });
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  verifyToken: async (platform) => {
    try {
      return await configApi.verifyToken(platform);
    } catch (error) {
      set({ error: String(error) });
      return false;
    }
  },

  getOAuthConfig: async (platform) => {
    try {
      return await configApi.getOAuthConfig(platform);
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  saveOAuthConfig: async (platform, clientId, clientSecret) => {
    try {
      await configApi.saveOAuthConfig(platform, clientId, clientSecret);
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
      await configApi.deleteOAuthConfig(platform);
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
      return await configApi.hasOAuthConfig(platform);
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
