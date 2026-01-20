import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface ConfigStore {
  hasTwitchToken: boolean;
  hasYouTubeToken: boolean;
  loading: boolean;
  error: string | null;
  checkTokens: () => Promise<void>;
  saveToken: (platform: string, token: string) => Promise<void>;
  deleteToken: (platform: string) => Promise<void>;
  verifyToken: (platform: string) => Promise<boolean>;
}

export const useConfigStore = create<ConfigStore>((set) => ({
  hasTwitchToken: false,
  hasYouTubeToken: false,
  loading: false,
  error: null,

  checkTokens: async () => {
    set({ loading: true });
    try {
      const [hasTwitch, hasYouTube] = await Promise.all([
        invoke<boolean>('has_token', { platform: 'twitch' }),
        invoke<boolean>('has_token', { platform: 'youtube' }),
      ]);
      set({
        hasTwitchToken: hasTwitch,
        hasYouTubeToken: hasYouTube,
        loading: false,
      });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  saveToken: async (platform, token) => {
    try {
      await invoke('save_token', { platform, token });
      if (platform === 'twitch') {
        set({ hasTwitchToken: true });
      } else if (platform === 'youtube') {
        set({ hasYouTubeToken: true });
      }
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteToken: async (platform) => {
    try {
      await invoke('delete_token', { platform });
      if (platform === 'twitch') {
        set({ hasTwitchToken: false });
      } else if (platform === 'youtube') {
        set({ hasYouTubeToken: false });
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
}));
