import { useState, useEffect } from 'react';
import { OAuthLogin } from './OAuthLogin';
import { TokenForm } from './TokenForm';
import { useConfigStore } from '../../stores/configStore';
import { useThemeStore } from '../../stores/themeStore';

export function Settings() {
  const [twitchAuthMethod, setTwitchAuthMethod] = useState<'token' | 'oauth' | null>(null);
  const [youtubeAuthMethod, setYoutubeAuthMethod] = useState<'token' | 'oauth' | null>(null);

  const { hasTwitchToken, hasYouTubeToken, checkTokens } = useConfigStore();
  const { theme, setTheme } = useThemeStore();

  useEffect(() => {
    checkTokens();
  }, [checkTokens]);

  return (
    <div className="space-y-8 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">設定</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">アプリケーションの各種設定</p>
      </div>

      {/* テーマ設定 */}
      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">表示設定</h2>
        <div className="space-y-3">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            カラーテーマ
          </label>
          <div className="flex space-x-3">
            <button
              onClick={() => setTheme('light')}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                theme === 'light'
                  ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              ライト
            </button>
            <button
              onClick={() => setTheme('dark')}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                theme === 'dark'
                  ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              ダーク
            </button>
            <button
              onClick={() => setTheme('system')}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                theme === 'system'
                  ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              システムに合わせる
            </button>
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            現在のテーマ: {theme === 'system' ? 'システム設定に従う' : theme === 'dark' ? 'ダーク' : 'ライト'}
          </p>
        </div>
      </section>

      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">Twitch API設定</h2>
        {/* 認証状態表示 */}
        <div className="flex items-center space-x-2">
          <div className={`w-3 h-3 rounded-full ${hasTwitchToken ? 'bg-green-500' : 'bg-gray-400'}`}></div>
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {hasTwitchToken ? 'Twitchに接続済み' : 'Twitch未接続'}
          </span>
        </div>

        {/* 認証方法選択 */}
        <div className="flex space-x-3">
          <button
            onClick={() => setTwitchAuthMethod(twitchAuthMethod === 'token' ? null : 'token')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
              twitchAuthMethod === 'token'
                ? 'bg-blue-600 text-white shadow-md'
                : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
            }`}
          >
            トークン設定
          </button>
          <button
            onClick={() => setTwitchAuthMethod(twitchAuthMethod === 'oauth' ? null : 'oauth')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
              twitchAuthMethod === 'oauth'
                ? 'bg-purple-600 text-white shadow-md'
                : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
            }`}
          >
            OAuth認証
          </button>
        </div>

        {/* 展開フォーム */}
        {twitchAuthMethod === 'token' && (
          <TokenForm
            platform="twitch"
            onClose={() => setTwitchAuthMethod(null)}
          />
        )}
        {twitchAuthMethod === 'oauth' && (
          <OAuthLogin
            platform="twitch"
            onClose={() => setTwitchAuthMethod(null)}
          />
        )}
      </section>

      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">YouTube API設定</h2>
        {/* 認証状態表示 */}
        <div className="flex items-center space-x-2">
          <div className={`w-3 h-3 rounded-full ${hasYouTubeToken ? 'bg-green-500' : 'bg-red-400'}`}></div>
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {hasYouTubeToken ? 'YouTubeに接続済み' : 'YouTube未接続'}
          </span>
        </div>

        {/* 認証方法選択 */}
        <div className="flex space-x-3">
          <button
            onClick={() => setYoutubeAuthMethod(youtubeAuthMethod === 'token' ? null : 'token')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
              youtubeAuthMethod === 'token'
                ? 'bg-blue-600 text-white shadow-md'
                : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
            }`}
          >
            トークン設定
          </button>
          <button
            onClick={() => setYoutubeAuthMethod(youtubeAuthMethod === 'oauth' ? null : 'oauth')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
              youtubeAuthMethod === 'oauth'
                ? 'bg-purple-600 text-white shadow-md'
                : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
            }`}
          >
            OAuth認証
          </button>
        </div>

        {/* 展開フォーム */}
        {youtubeAuthMethod === 'token' && (
          <TokenForm
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
    </div>
  );
}
