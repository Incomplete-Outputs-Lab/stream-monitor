import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { DeviceAuthStatus } from '../../types';
import { useConfigStore } from '../../stores/configStore';

interface TwitchAuthPanelProps {
  onClose?: () => void;
  onSuccess?: () => void;
}

export function TwitchAuthPanel({ onClose, onSuccess }: TwitchAuthPanelProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [deviceAuth, setDeviceAuth] = useState<DeviceAuthStatus | null>(null);
  const [timeRemaining, setTimeRemaining] = useState<number>(0);
  const [pollingActive, setPollingActive] = useState(false);
  const checkTokens = useConfigStore((state) => state.checkTokens);

  // タイマー管理
  useEffect(() => {
    if (timeRemaining > 0) {
      const timer = setTimeout(() => {
        setTimeRemaining(timeRemaining - 1);
      }, 1000);
      return () => clearTimeout(timer);
    } else if (timeRemaining === 0 && deviceAuth && pollingActive) {
      // タイムアウト
      setError('認証がタイムアウトしました。もう一度お試しください。');
      setPollingActive(false);
      setDeviceAuth(null);
    }
  }, [timeRemaining, deviceAuth, pollingActive]);

  // バックエンドからの認証成功イベントをリッスン
  useEffect(() => {
    let unlistenSuccess: (() => void) | undefined;
    let unlistenRequired: (() => void) | undefined;

    const setupListeners = async () => {
      // 認証成功イベント
      unlistenSuccess = await listen('twitch-auth-success', async () => {
        console.log('[TwitchAuthPanel] Received twitch-auth-success event');
        
        // Strongholdへの保存が完全に完了するまで少し待機
        await new Promise((resolve) => setTimeout(resolve, 800));
        
        console.log('[TwitchAuthPanel] Checking token status after auth success...');
        
        // トークンステータスを更新
        await checkTokens();
        
        // UIを更新
        setPollingActive(false);
        setDeviceAuth(null);
        setSuccess('Twitch認証に成功しました！');
        
        console.log('[TwitchAuthPanel] Token status updated successfully');
        
        // コールバックを実行
        if (onSuccess) {
          await onSuccess();
        }
        
        // 2秒後に画面を閉じる
        if (onClose) {
          setTimeout(() => onClose(), 2000);
        }
      });

      // 再認証が必要なイベント（リフレッシュトークンが無効になった場合）
      unlistenRequired = await listen('twitch-auth-required', async () => {
        console.log('[TwitchAuthPanel] Received twitch-auth-required event');
        
        // トークンステータスを更新
        await checkTokens();
        
        // UIを更新
        setError('Twitch認証の有効期限が切れました。再度認証してください。');
        setPollingActive(false);
        setDeviceAuth(null);
        setSuccess(null);
      });
    };

    setupListeners();

    return () => {
      if (unlistenSuccess) {
        unlistenSuccess();
      }
      if (unlistenRequired) {
        unlistenRequired();
      }
    };
  }, [checkTokens, onSuccess, onClose]);

  const handleStartAuth = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);
    setDeviceAuth(null);

    try {
      // Device Code Flow を開始
      const authStatus = await invoke<DeviceAuthStatus>('start_twitch_device_auth');
      
      setDeviceAuth(authStatus);
      setTimeRemaining(authStatus.expires_in);
      setLoading(false);

      // バックグラウンドでポーリング開始
      startPolling(authStatus);
    } catch (err) {
      setError(`認証開始エラー: ${String(err)}`);
      setLoading(false);
    }
  };

  const startPolling = async (authStatus: DeviceAuthStatus) => {
    setPollingActive(true);

    try {
      // ポーリングを開始（バックエンドで処理）
      // トークン取得成功後、バックエンドが 'twitch-auth-success' イベントをemitする
      // イベントリスナーが状態更新を処理するため、ここでは何もしない
      await invoke<string>('poll_twitch_device_token', {
        deviceCode: authStatus.device_code,
        interval: authStatus.interval,
        clientId: await getClientId(),
      });
      
      console.log('[TwitchAuthPanel] Token polling completed, waiting for event...');
    } catch (err) {
      // エラー時のみ処理
      setPollingActive(false);
      setDeviceAuth(null);
      setError(`認証エラー: ${String(err)}`);
    }
  };

  const getClientId = async (): Promise<string> => {
    // 設定からClient IDを取得
    const config = await invoke<{ client_id?: string }>('get_oauth_config', {
      platform: 'twitch',
    });
    return config.client_id || '';
  };

  const handleOpenBrowser = async () => {
    if (deviceAuth) {
      try {
        // Tauri opener pluginを使用してブラウザでURLを開く
        await openUrl(deviceAuth.verification_uri);
      } catch (err) {
        setError(`ブラウザを開けませんでした: ${String(err)}`);
      }
    }
  };

  const handleCopyCode = () => {
    if (deviceAuth) {
      navigator.clipboard.writeText(deviceAuth.user_code);
    }
  };

  const handleClose = () => {
    setError(null);
    setSuccess(null);
    setDeviceAuth(null);
    setPollingActive(false);
    if (onClose) {
      onClose();
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="space-y-4">
      {/* ヘッダー */}
      <div className="flex items-center justify-between pb-3 border-b border-gray-200 dark:border-slate-600">
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
          Twitch 認証
        </h2>
        {onClose && (
          <button
            onClick={handleClose}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 text-xl"
          >
            ✕
          </button>
        )}
      </div>

      {/* メッセージ表示 */}
      {error && (
        <div className="text-red-500 text-sm p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
          {error}
        </div>
      )}

      {success && (
        <div className="text-green-600 text-sm p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
          {success}
        </div>
      )}

      {/* デバイスコード表示 */}
      {deviceAuth && (
        <div className="p-5 border-2 border-purple-500 dark:border-purple-400 rounded-lg bg-purple-50 dark:bg-purple-900/20 space-y-4">
          <div className="text-center space-y-3">
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
              以下のコードをブラウザで入力してください：
            </p>
            
            <div className="relative">
              <div className="text-5xl font-bold text-purple-600 dark:text-purple-400 tracking-wider py-4 select-all">
                {deviceAuth.user_code}
              </div>
              <button
                onClick={handleCopyCode}
                className="absolute top-2 right-2 px-3 py-1 text-xs bg-purple-600 hover:bg-purple-700 text-white rounded transition-colors"
              >
                コピー
              </button>
            </div>

            <button
              onClick={handleOpenBrowser}
              className="w-full px-6 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors text-base font-semibold shadow-md hover:shadow-lg"
            >
              🌐 ブラウザで認証ページを開く
            </button>

            <p className="text-xs text-gray-600 dark:text-gray-400">
              または、手動で以下のURLにアクセス：<br />
              <a
                href={deviceAuth.verification_uri}
                target="_blank"
                rel="noopener noreferrer"
                className="text-purple-600 dark:text-purple-400 underline break-all"
              >
                {deviceAuth.verification_uri}
              </a>
            </p>

            {pollingActive && (
              <div className="flex flex-col items-center space-y-2 py-3">
                <div className="flex items-center space-x-2">
                  <span className="animate-spin text-2xl">⏳</span>
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    認証を待っています...
                  </span>
                </div>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  残り時間: {formatTime(timeRemaining)}
                </p>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 認証開始ボタン（デバイスコードがない場合） */}
      {!deviceAuth && (
        <div className="p-5 border border-gray-300 dark:border-slate-600 rounded-lg bg-gray-50 dark:bg-slate-800">
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="text-2xl">🎮</div>
              <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                Twitchアカウントでログイン
              </h3>
            </div>

            <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
              ブラウザでTwitchにログインして認証します。
              Device Code認証を使用し、Client Secret不要で安全に認証できます。
            </p>

            <button
              onClick={handleStartAuth}
              disabled={loading}
              className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-semibold shadow-md hover:shadow-lg"
            >
              {loading ? (
                <span className="flex items-center justify-center space-x-2">
                  <span className="animate-spin">⏳</span>
                  <span>準備中...</span>
                </span>
              ) : (
                'Twitchで認証'
              )}
            </button>

            <div className="text-xs text-gray-500 dark:text-gray-400 space-y-1">
              <p>✓ 安全なDevice Code認証</p>
              <p>✓ Client Secret不要</p>
              <p>✓ リフレッシュトークン対応（30日間有効）</p>
              <p>✓ 自動トークン更新</p>
            </div>
          </div>
        </div>
      )}

      {/* 補足説明 */}
      <div className="text-xs text-gray-500 dark:text-gray-400 p-3 bg-blue-50 dark:bg-blue-900/10 rounded-lg border border-blue-200 dark:border-blue-800">
        <p className="font-semibold text-blue-700 dark:text-blue-400 mb-1">💡 Device Code認証について</p>
        <p>
          Device Code Grant Flow は、デスクトップアプリケーション向けのTwitch公式推奨認証方式です。
          Client Secretを埋め込む必要がなく、セキュアに認証できます。
          ブラウザで認証コードを入力するだけで、アプリケーションが自動的にトークンを取得します。
        </p>
      </div>
    </div>
  );
}
