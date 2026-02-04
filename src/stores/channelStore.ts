import { create } from 'zustand';
import { Channel } from '../types';
import * as channelsApi from '../api/channels';

interface ChannelStore {
  channels: Channel[];
  loading: boolean;
  error: string | null;
  fetchChannels: () => Promise<void>;
  addChannel: (channel: Omit<Channel, 'id' | 'created_at' | 'updated_at'>) => Promise<void>;
  removeChannel: (id: number) => Promise<void>;
  updateChannel: (id: number, updates: Partial<Channel>) => Promise<void>;
  toggleChannel: (id: number) => Promise<void>;
}

export const useChannelStore = create<ChannelStore>((set) => ({
  channels: [],
  loading: false,
  error: null,

  fetchChannels: async () => {
    set({ loading: true, error: null });
    try {
      const channels = await channelsApi.listChannels();
      set({ channels, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  addChannel: async (channel) => {
    try {
      const newChannel = await channelsApi.addChannel({
        platform: channel.platform,
        channel_id: channel.channel_id,
        channel_name: channel.channel_name,
        poll_interval: channel.poll_interval,
      });
      set((state) => ({
        channels: [...state.channels, newChannel],
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  removeChannel: async (id) => {
    try {
      await channelsApi.removeChannel(id);
      set((state) => ({
        channels: state.channels.filter((ch) => ch.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateChannel: async (id, updates) => {
    try {
      const updated = await channelsApi.updateChannel({
        id,
        channel_name: updates.channel_name,
        poll_interval: updates.poll_interval,
        enabled: updates.enabled,
      });
      set((state) => ({
        channels: state.channels.map((ch) => (ch.id === id ? updated : ch)),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  toggleChannel: async (id) => {
    try {
      const updated = await channelsApi.toggleChannel(id);
      set((state) => ({
        channels: state.channels.map((ch) => (ch.id === id ? updated : ch)),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
