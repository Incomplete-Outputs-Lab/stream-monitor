import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { OAuthLogin } from './OAuthLogin';
import { OAuthConfigForm } from './OAuthConfigForm';
import { TokenForm } from './TokenForm';
import { TwitchAuthPanel } from './TwitchAuthPanel';
import { useConfigStore } from '../../stores/configStore';
import { useThemeStore } from '../../stores/themeStore';

interface BuildInfo {
  version: string;
  commit_hash?: string;
  build_date?: string;
  developer: string;
}

export function Settings() {
  const [twitchAuthOpen, setTwitchAuthOpen] = useState(false);
  const [youtubeAuthMethod, setYoutubeAuthMethod] = useState<'token' | 'oauth' | 'config' | null>(null);

  const { hasTwitchToken, hasYouTubeToken, hasYouTubeOAuth, checkTokens } = useConfigStore();
  const { theme, setTheme } = useThemeStore();

  // ビルド情報取得
  const { data: buildInfo } = useQuery({
    queryKey: ["build-info"],
    queryFn: async () => {
      return await invoke<BuildInfo>("get_build_info");
    },
  });

  useEffect(() => {
    checkTokens();
  }, [checkTokens]);

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">設定</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">アプリケーションの各種設定</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 表示設定 */}
        <section className="card p-4 space-y-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">表示設定</h2>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              カラーテーマ
            </label>
            <div className="flex flex-wrap gap-2">
              <button
                onClick={() => setTheme('light')}
                className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                  theme === 'light'
                    ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-sm'
                    : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
                }`}
              >
                ライト
              </button>
              <button
                onClick={() => setTheme('dark')}
                className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                  theme === 'dark'
                    ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-sm'
                    : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
                }`}
              >
                ダーク
              </button>
              <button
                onClick={() => setTheme('system')}
                className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                  theme === 'system'
                    ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-sm'
                    : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
                }`}
              >
                システム
              </button>
            </div>
          </div>
        </section>

        {/* Twitch API設定 */}
        <section className="card p-4 space-y-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Twitch API</h2>
          <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${hasTwitchToken ? 'bg-green-500' : 'bg-gray-400'}`}></div>
            <span className="text-xs text-gray-600 dark:text-gray-400">
              {hasTwitchToken ? '接続済み' : '未接続'}
            </span>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => setTwitchAuthOpen(!twitchAuthOpen)}
              className={`px-4 py-2 rounded text-sm font-medium transition-all duration-200 ${
                twitchAuthOpen
                  ? 'bg-purple-600 text-white shadow-sm'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              🔐 認証
            </button>
          </div>
          {twitchAuthOpen && (
            <div className="mt-4">
              <TwitchAuthPanel
                onClose={() => setTwitchAuthOpen(false)}
              />
            </div>
          )}
        </section>

        {/* YouTube API設定 */}
        <section className="card p-4 space-y-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">YouTube API</h2>
          <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${hasYouTubeToken ? 'bg-green-500' : 'bg-gray-400'}`}></div>
            <span className="text-xs text-gray-600 dark:text-gray-400">
              {hasYouTubeToken ? '接続済み' : '未接続'}
            </span>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => setYoutubeAuthMethod(youtubeAuthMethod === 'token' ? null : 'token')}
              className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                youtubeAuthMethod === 'token'
                  ? 'bg-blue-600 text-white shadow-sm'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              トークン
            </button>
            {hasYouTubeOAuth ? (
              <button
                onClick={() => {
                  // OAuthログインを実行
                  const performLogin = async () => {
                    try {
                      await invoke('login_with_youtube');
                      // トークンの確認を更新
                      await checkTokens();
                    } catch (error) {
                      console.error('YouTube login failed:', error);
                      alert(`YouTubeログインに失敗しました: ${error}`);
                    }
                  };
                  performLogin();
                }}
                className="px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 bg-green-600 hover:bg-green-700 text-white shadow-sm"
              >
                YouTubeにログイン
              </button>
            ) : (
              <button
                onClick={() => setYoutubeAuthMethod(youtubeAuthMethod === 'config' ? null : 'config')}
                className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                  youtubeAuthMethod === 'config'
                    ? 'bg-purple-600 text-white shadow-sm'
                    : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
                }`}
              >
                OAuth設定
              </button>
            )}
            <button
              onClick={() => setYoutubeAuthMethod(youtubeAuthMethod === 'oauth' ? null : 'oauth')}
              className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                youtubeAuthMethod === 'oauth'
                  ? 'bg-orange-600 text-white shadow-sm'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              OAuth詳細
            </button>
          </div>
          {youtubeAuthMethod === 'token' && (
            <TokenForm
              platform="youtube"
              onClose={() => setYoutubeAuthMethod(null)}
            />
          )}
          {youtubeAuthMethod === 'config' && (
            <OAuthConfigForm
              platform="youtube"
              onClose={() => setYoutubeAuthMethod(null)}
            />
          )}
          {youtubeAuthMethod === 'oauth' && (
            <OAuthLogin
              platform="youtube"
              onClose={() => setYoutubeAuthMethod(null)}
            />
          )}
        </section>

        {/* ビルド情報 */}
        {buildInfo && (
          <section className="card p-4 space-y-3">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">ビルド情報</h2>
            <div className="space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">バージョン:</span>
                <span className="text-gray-900 dark:text-gray-100 font-mono">{buildInfo.version}</span>
              </div>
              {buildInfo.commit_hash && (
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">コミット:</span>
                  <span className="text-gray-900 dark:text-gray-100 font-mono">
                    {buildInfo.commit_hash.substring(0, 8)}
                  </span>
                </div>
              )}
              {buildInfo.build_date && (
                <div className="flex justify-between">
                  <span className="text-gray-600 dark:text-gray-400">ビルド日時:</span>
                  <span className="text-gray-900 dark:text-gray-100">{buildInfo.build_date}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">開発元:</span>
                <span className="text-gray-900 dark:text-gray-100">{buildInfo.developer}</span>
              </div>
            </div>
          </section>
        )}
      </div>
    </div>
  );
}
