import { useState, useEffect, useRef } from 'react';
import type { AutoDiscoverySettings, TwitchGame, SelectedGame } from '../../types';
import * as discoveryApi from '../../api/discovery';
import * as configApi from '../../api/config';
import { useAppStateStore } from '../../stores/appStateStore';

export function AutoDiscoveryForm() {
  const backendReady = useAppStateStore((state) => state.backendReady);
  const [settings, setSettings] = useState<AutoDiscoverySettings>({
    enabled: false,
    poll_interval: 300,
    max_streams: 20,
    filters: {
      game_ids: [],
      languages: [],
      min_viewers: 0,
    },
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [languageInput, setLanguageInput] = useState('');
  const [showDetails, setShowDetails] = useState(false);
  const [hasClientId, setHasClientId] = useState(false);

  // ゲーム検索UI用のstate
  const [gameSearchQuery, setGameSearchQuery] = useState('');
  const [gameSearchResults, setGameSearchResults] = useState<TwitchGame[]>([]);
  const [selectedGames, setSelectedGames] = useState<SelectedGame[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const searchTimeoutRef = useRef<number | null>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // 設定を読み込み（バックエンド準備完了後に実行し、最新の game_ids 等を反映）
  useEffect(() => {
    if (!backendReady) return;

    const loadSettings = async () => {
      try {
        // Client IDが設定されているかチェック
        const twitchOAuth = await configApi.getOAuthConfig('twitch');
        setHasClientId(!!twitchOAuth.client_id);

        const result = await discoveryApi.getAutoDiscoverySettings();
        
        if (result) {
          setSettings(result);

          // 既存のgame_idsからゲーム名を取得
          if (result.filters.game_ids.length > 0) {
            try {
              const games = await discoveryApi.getGamesByIds(result.filters.game_ids);
              setSelectedGames(games.map(g => ({ id: g.id, name: g.name })));
            } catch (err) {
              console.error('Failed to load game names:', err);
              // エラー時はIDのみ表示
              setSelectedGames(result.filters.game_ids.map(id => ({ id, name: id })));
            }
          } else {
            setSelectedGames([]);
          }
        }
      } catch (err) {
        console.error('Failed to load auto-discovery settings:', err);
      }
    };

    loadSettings();
  }, [backendReady]);

  // ドロップダウン外クリック時に閉じる
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // 検索クエリ変更時のdebounce処理
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }

    if (gameSearchQuery.trim().length === 0) {
      setGameSearchResults([]);
      setShowDropdown(false);
      return;
    }

    setIsSearching(true);
    searchTimeoutRef.current = window.setTimeout(async () => {
      try {
        const results = await discoveryApi.searchTwitchGames(gameSearchQuery);
        setGameSearchResults(results);
        setShowDropdown(results.length > 0);
      } catch (err) {
        console.error('Failed to search games:', err);
        setGameSearchResults([]);
        setShowDropdown(false);
      } finally {
        setIsSearching(false);
      }
    }, 300);

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [gameSearchQuery]);

  const handleSave = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await discoveryApi.saveAutoDiscoverySettings(settings);
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
      const newEnabled = await discoveryApi.toggleAutoDiscovery();
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

  const handleSelectGame = (game: TwitchGame) => {
    // 既に選択済みかチェック
    if (selectedGames.some(g => g.id === game.id)) {
      return;
    }

    // selectedGamesに追加
    setSelectedGames((prev) => [...prev, { id: game.id, name: game.name }]);

    // settingsのgame_idsに追加
    setSettings((prev) => ({
      ...prev,
      filters: {
        ...prev.filters,
        game_ids: [...prev.filters.game_ids, game.id],
      },
    }));

    // 検索UIをリセット
    setGameSearchQuery('');
    setGameSearchResults([]);
    setShowDropdown(false);
  };

  const handleRemoveGame = (gameId: string) => {
    // selectedGamesから削除
    setSelectedGames((prev) => prev.filter((g) => g.id !== gameId));

    // settingsのgame_idsから削除
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

      {/* Client ID未設定の警告 */}
      {!hasClientId && (
        <div className="p-3 bg-amber-100 dark:bg-amber-900/30 border border-amber-400 dark:border-amber-700 text-amber-800 dark:text-amber-300 rounded text-xs flex items-start gap-2">
          <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <div>
            <p className="font-semibold mb-1">Twitch Client IDが未設定です</p>
            <p>自動発見機能を使用するには、まず「Twitch API」セクションで「OAuth設定」からClient IDを設定してください。</p>
          </div>
        </div>
      )}

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
          disabled={loading || !hasClientId}
          className={`px-3 py-1.5 rounded text-xs font-medium transition-all duration-200 ${
            settings.enabled
              ? 'bg-green-600 text-white shadow-sm'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          } disabled:opacity-50 disabled:cursor-not-allowed`}
          title={!hasClientId ? 'Client IDを設定してください' : ''}
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
              value={settings.filters.min_viewers}
              onChange={(e) =>
                setSettings((prev) => ({
                  ...prev,
                  filters: {
                    ...prev.filters,
                    min_viewers: parseInt(e.target.value) || 0,
                  },
                }))
              }
              className="input-field"
              placeholder="0を指定すると全て取得"
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

          {/* ゲーム/カテゴリフィルター */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              ゲーム/カテゴリフィルター（任意）
            </label>
            <div className="relative" ref={dropdownRef}>
              <input
                type="text"
                value={gameSearchQuery}
                onChange={(e) => setGameSearchQuery(e.target.value)}
                placeholder="ゲーム名で検索..."
                className="input-field w-full"
              />
              {showDropdown && (
                <div className="absolute z-10 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg max-h-64 overflow-y-auto">
                  {isSearching ? (
                    <div className="p-3 text-center text-sm text-gray-500 dark:text-gray-400">
                      検索中...
                    </div>
                  ) : gameSearchResults.length > 0 ? (
                    gameSearchResults.map((game) => (
                      <button
                        key={game.id}
                        onClick={() => handleSelectGame(game)}
                        className="w-full flex items-center gap-3 p-2 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
                      >
                        <img
                          src={game.box_art_url.replace('{width}', '52').replace('{height}', '72')}
                          alt={game.name}
                          className="w-10 h-14 object-cover rounded"
                        />
                        <span className="text-sm text-gray-900 dark:text-gray-100">
                          {game.name}
                        </span>
                      </button>
                    ))
                  ) : (
                    <div className="p-3 text-center text-sm text-gray-500 dark:text-gray-400">
                      結果が見つかりませんでした
                    </div>
                  )}
                </div>
              )}
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {selectedGames.map((game) => (
                <span
                  key={game.id}
                  className="inline-flex items-center gap-1 px-3 py-1 bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300 rounded-full text-sm"
                >
                  {game.name}
                  <button
                    onClick={() => handleRemoveGame(game.id)}
                    className="hover:text-purple-600 dark:hover:text-purple-400"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              ゲーム名で検索して追加（空の場合は全カテゴリ）
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
