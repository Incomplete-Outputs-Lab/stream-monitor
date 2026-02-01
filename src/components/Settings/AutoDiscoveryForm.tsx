import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AutoDiscoverySettings } from '../../types';

export function AutoDiscoveryForm() {
  const [settings, setSettings] = useState<AutoDiscoverySettings>({
    enabled: false,
    poll_interval: 300,
    max_streams: 20,
    filters: {
      game_ids: [],
      languages: [],
      min_viewers: undefined,
    },
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [languageInput, setLanguageInput] = useState('');
  const [gameIdInput, setGameIdInput] = useState('');
  const [showDetails, setShowDetails] = useState(false);

  // 設定を読み込み
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const result = await invoke<AutoDiscoverySettings | null>(
          'get_auto_discovery_settings'
        );
        if (result) {
          setSettings(result);
        }
      } catch (err) {
        console.error('Failed to load auto-discovery settings:', err);
      }
    };

    loadSettings();
  }, []);

  const handleSave = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await invoke('save_auto_discovery_settings', { settings });
      setSuccess('保存しました');
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const newEnabled = await invoke<boolean>('toggle_auto_discovery');
      setSettings((prev) => ({ ...prev, enabled: newEnabled }));
      setSuccess(newEnabled ? '有効化しました' : '無効化しました');
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleAddLanguage = () => {
    const lang = languageInput.trim().toLowerCase();
    if (lang && !settings.filters.languages.includes(lang)) {
      setSettings((prev) => ({
        ...prev,
        filters: {
          ...prev.filters,
          languages: [...prev.filters.languages, lang],
        },
      }));
      setLanguageInput('');
    }
  };

  const handleRemoveLanguage = (lang: string) => {
    setSettings((prev) => ({
      ...prev,
      filters: {
        ...prev.filters,
        languages: prev.filters.languages.filter((l) => l !== lang),
      },
    }));
  };

  const handleAddGameId = () => {
    const gameId = gameIdInput.trim();
    if (gameId && !settings.filters.game_ids.includes(gameId)) {
      setSettings((prev) => ({
        ...prev,
        filters: {
          ...prev.filters,
          game_ids: [...prev.filters.game_ids, gameId],
        },
      }));
      setGameIdInput('');
    }
  };

  const handleRemoveGameId = (gameId: string) => {
    setSettings((prev) => ({
      ...prev,
      filters: {
        ...prev.filters,
        game_ids: prev.filters.game_ids.filter((id) => id !== gameId),
      },
    }));
  };

  return (
    <section className="card p-4 space-y-3">
      <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Twitch自動発見</h2>
      <div className="flex items-center space-x-2">
        <div className={`w-2 h-2 rounded-full ${settings.enabled ? 'bg-green-500' : 'bg-gray-400'}`}></div>
        <span className="text-xs text-gray-600 dark:text-gray-400">
          {settings.enabled ? '有効' : '無効'}
        </span>
      </div>

      {error && (
        <div className="p-2 bg-red-100 dark:bg-red-900/30 border border-red-400 dark:border-red-700 text-red-800 dark:text-red-300 rounded text-xs">
          {error}
        </div>
      )}

      {success && (
        <div className="p-2 bg-green-100 dark:bg-green-900/30 border border-green-400 dark:border-green-700 text-green-800 dark:text-green-300 rounded text-xs">
          {success}
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        <button
          onClick={handleToggle}
          disabled={loading}
          className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
            settings.enabled
              ? 'bg-green-600 text-white shadow-sm'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          } disabled:opacity-50`}
        >
          {settings.enabled ? 'ON' : 'OFF'}
        </button>
        <button
          onClick={() => setShowDetails(!showDetails)}
          className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
            showDetails
              ? 'bg-purple-600 text-white shadow-sm'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          詳細設定
        </button>
      </div>

      {showDetails && (
        <div className="space-y-4 p-4 border border-gray-200 dark:border-slate-600 rounded-lg bg-gray-50 dark:bg-slate-800">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            視聴者数上位の配信を自動的に発見・監視します
          </div>

          {/* ポーリング間隔 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              ポーリング間隔（秒）
            </label>
            <input
              type="number"
              min="60"
              max="3600"
              value={settings.poll_interval}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  poll_interval: parseInt(e.target.value) || 300,
                }))
              }
              className="input-field"
            />
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              推奨: 300秒（5分）以上
            </p>
          </div>

          {/* 最大取得件数 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              最大取得件数
            </label>
            <input
              type="number"
              min="1"
              max="100"
              value={settings.max_streams}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  max_streams: parseInt(e.target.value) || 20,
                }))
              }
              className="input-field"
            />
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              最大100件まで指定可能
            </p>
          </div>

          {/* 最小視聴者数 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              最小視聴者数（任意）
            </label>
            <input
              type="number"
              min="0"
              value={settings.filters.min_viewers || ''}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  filters: {
                    ...prev.filters,
                    min_viewers: e.target.value
                      ? parseInt(e.target.value)
                      : undefined,
                  },
                }))
              }
              className="input-field"
              placeholder="指定しない場合は全て取得"
            />
          </div>

          {/* 言語フィルター */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              言語フィルター（任意）
            </label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={languageInput}
                onChange={(e) => setLanguageInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddLanguage()}
                placeholder="例: ja, en"
                className="input-field flex-1"
              />
              <button
                onClick={handleAddLanguage}
                className="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors text-sm font-medium"
              >
                追加
              </button>
            </div>
            <div className="flex flex-wrap gap-2">
              {settings.filters.languages.map((lang) => (
                <span
                  key={lang}
                  className="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 rounded-full text-sm"
                >
                  {lang}
                  <button
                    onClick={() => handleRemoveLanguage(lang)}
                    className="hover:text-blue-600 dark:hover:text-blue-400"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              ISO 639-1コード（例: ja=日本語, en=英語, ko=韓国語）
            </p>
          </div>

          {/* ゲームIDフィルター */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              ゲーム/カテゴリIDフィルター（任意）
            </label>
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={gameIdInput}
                onChange={(e) => setGameIdInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleAddGameId()}
                placeholder="TwitchゲームID"
                className="input-field flex-1"
              />
              <button
                onClick={handleAddGameId}
                className="px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors text-sm font-medium"
              >
                追加
              </button>
            </div>
            <div className="flex flex-wrap gap-2">
              {settings.filters.game_ids.map((gameId) => (
                <span
                  key={gameId}
                  className="inline-flex items-center gap-1 px-3 py-1 bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300 rounded-full text-sm"
                >
                  {gameId}
                  <button
                    onClick={() => handleRemoveGameId(gameId)}
                    className="hover:text-purple-600 dark:hover:text-purple-400"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Twitch API のゲームIDを指定（空の場合は全カテゴリ）
            </p>
          </div>

          <div className="flex space-x-3">
            <button
              onClick={handleSave}
              disabled={loading}
              className="flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-medium"
            >
              {loading ? '保存中...' : '保存'}
            </button>
          </div>
        </div>
      )}
    </section>
  );
}
