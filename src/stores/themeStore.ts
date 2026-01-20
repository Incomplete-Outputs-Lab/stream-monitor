import { create } from 'zustand';

type Theme = 'light' | 'dark' | 'system';

interface ThemeStore {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  getEffectiveTheme: () => 'light' | 'dark';
}

const STORAGE_KEY = 'stream-monitor-theme';

function getStoredTheme(): Theme {
  if (typeof window === 'undefined') return 'system';
  const stored = localStorage.getItem(STORAGE_KEY);
  return (stored as Theme) || 'system';
}

function applyTheme(theme: Theme) {
  if (typeof window === 'undefined') return;
  const root = document.documentElement;
  const effectiveTheme = theme === 'system' 
    ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
    : theme;
  
  if (effectiveTheme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

export const useThemeStore = create<ThemeStore>((set, get) => {
  const initialTheme = getStoredTheme();
  
  // 初期化時にテーマを適用
  if (typeof window !== 'undefined') {
    applyTheme(initialTheme);
    
    // システムテーマ変更を監視
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      const { theme } = get();
      if (theme === 'system') {
        applyTheme('system');
      }
    });
  }
  
  return {
    theme: initialTheme,
    setTheme: (theme) => {
      set({ theme });
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY, theme);
        applyTheme(theme);
      }
    },
    getEffectiveTheme: () => {
      const { theme } = get();
      if (theme === 'system') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      }
      return theme;
    },
  };
});
