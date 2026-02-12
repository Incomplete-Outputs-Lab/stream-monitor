import { create } from 'zustand';

interface AppStateStore {
  backendReady: boolean;
  setBackendReady: (ready: boolean) => void;
}

export const useAppStateStore = create<AppStateStore>((set) => ({
  backendReady: false,
  setBackendReady: (ready: boolean) => set({ backendReady: ready }),
}));
