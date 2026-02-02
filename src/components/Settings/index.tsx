import { useState, useEffect } from 'react';
import { useQuery, useQueryClient, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { OAuthConfigForm } from './OAuthConfigForm';
import { TokenForm } from './TokenForm';
import { TwitchAuthPanel } from './TwitchAuthPanel';
import { AutoDiscoveryForm } from './AutoDiscoveryForm';
import { useConfigStore } from '../../stores/configStore';
import { useThemeStore } from '../../stores/themeStore';

interface BuildInfo {
  version: string;
  commit_hash?: string;
  build_date?: string;
  developer: string;
  repository_url: string;
}

export function Settings() {
  const [twitchAuthMethod, setTwitchAuthMethod] = useState<'auth' | 'config' | null>(null);
  const [youtubeAuthMethod, setYoutubeAuthMethod] = useState<'token' | 'config' | null>(null);

  const { hasTwitchToken, hasYouTubeToken, hasTwitchOAuth, checkTokens } = useConfigStore();
  const { theme, setTheme } = useThemeStore();
  const queryClient = useQueryClient();

  // 自動起動状態を取得
  const { data: autostartEnabled, isLoading: autostartLoading } = useQuery({
    queryKey: ['autostart-enabled'],
    queryFn: isEnabled,
  });

  // 自動起動トグル
  const autostartMutation = useMutation({
    mutationFn: async () => {
      if (await isEnabled()) {
        await disable();
      } else {
        await enable();
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['autostart-enabled'] });
    },
  });

  // デバッグ: 状態変化を監視
  useEffect(() => {
    console.log('[Settings] Token state changed:', {
      hasTwitchToken,
      hasYouTubeToken,
      hasTwitchOAuth,
    });
  }, [hasTwitchToken, hasYouTubeToken, hasTwitchOAuth]);

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

  // 認証パネルが閉じられたときにトークン状態を再確認
  useEffect(() => {
    if (twitchAuthMethod === null) {
      // Twitchの認証パネルが閉じられた後、トークン状態を再確認
      checkTokens();
    }
  }, [twitchAuthMethod, checkTokens]);

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

        {/* アプリ設定 */}
        <section className="card p-4 space-y-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">アプリ設定</h2>
          <div>
            <label className="flex items-center justify-between cursor-pointer">
              <div>
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  OS起動時に自動起動
                </span>
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                  Windowsの起動時にアプリを自動的に起動します
                </p>
              </div>
              <button
                onClick={() => autostartMutation.mutate()}
                disabled={autostartLoading || autostartMutation.isPending}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed ${
                  autostartEnabled
                    ? 'bg-purple-600'
                    : 'bg-gray-200 dark:bg-slate-600'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    autostartEnabled ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </label>
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
              onClick={() => setTwitchAuthMethod(twitchAuthMethod === 'config' ? null : 'config')}
              className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                twitchAuthMethod === 'config'
                  ? 'bg-purple-600 text-white shadow-sm'
                  : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
              }`}
            >
              OAuth設定
            </button>
            {hasTwitchOAuth && (
              <button
                onClick={() => setTwitchAuthMethod(twitchAuthMethod === 'auth' ? null : 'auth')}
                className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
                  twitchAuthMethod === 'auth'
                    ? 'bg-purple-600 text-white shadow-sm'
                    : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
                }`}
              >
                🔐 認証
              </button>
            )}
          </div>
          {twitchAuthMethod === 'config' && (
            <OAuthConfigForm
              platform="twitch"
              onClose={() => setTwitchAuthMethod(null)}
            />
          )}
          {twitchAuthMethod === 'auth' && (
            <div className="mt-4">
              <TwitchAuthPanel
                onClose={() => setTwitchAuthMethod(null)}
                onSuccess={() => {
                  // 認証成功時にトークン状態を更新
                  checkTokens();
                }}
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
        </section>

        {/* Twitch自動発見設定 */}
        <AutoDiscoveryForm />
      </div>

      {/* ビルド情報 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
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
                  <a
                    href={`${buildInfo.repository_url}/commit/${buildInfo.commit_hash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-purple-600 dark:text-purple-400 hover:text-purple-700 dark:hover:text-purple-300 font-mono transition-colors underline"
                  >
                    {buildInfo.commit_hash.substring(0, 8)}
                  </a>
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
              <div className="flex justify-between">
                <span className="text-gray-600 dark:text-gray-400">リポジトリ:</span>
                <a
                  href={buildInfo.repository_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-purple-600 dark:text-purple-400 hover:text-purple-700 dark:hover:text-purple-300 transition-colors underline"
                >
                  GitHub
                </a>
              </div>
            </div>
          </section>
        )}

        {/* サポート */}
        <section className="card p-4 space-y-3">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            開発を支援する
          </h2>
          
          <div className="text-xs text-gray-600 dark:text-gray-400 space-y-2">
            <p>
              Stream Monitorは個人で開発・運営しているオープンソースプロジェクトです。
            </p>
            <p>
              サポートいただいた資金は、サーバー維持費や新機能の開発に充てさせていただきます。
            </p>
          </div>
          
          <div className="pt-2">
            <h3 className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
              サポート方法
            </h3>
            <a
              href="http://subs.twitch.tv/flowingspdg"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-purple-600 hover:bg-purple-700 text-white text-sm font-medium transition-colors"
            >
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z"/>
              </svg>
              Twitchでサブスクライブ
            </a>
          </div>
        </section>
      </div>
    </div>
  );
}
