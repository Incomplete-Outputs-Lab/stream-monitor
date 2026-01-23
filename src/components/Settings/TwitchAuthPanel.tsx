import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';

interface TwitchAuthPanelProps {
  onClose?: () => void;
}

interface DeviceCodeResponse {
  user_code: string;
  verification_uri: string;
  device_code: string;
}

export function TwitchAuthPanel({ onClose }: TwitchAuthPanelProps) {
  const [loading, setLoading] = useState(false);
  const [deviceCode, setDeviceCode] = useState<DeviceCodeResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const { checkTokens } = useConfigStore();

  const handleStartAuth = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);
    setDeviceCode(null);

    try {
      // デバイスコードフローを開始
      const response = await invoke<DeviceCodeResponse>('start_twitch_device_flow');
      setDeviceCode(response);

      // ブラウザで認証ページを開く
      window.open(response.verification_uri, '_blank');

      // バックグラウンドでトークンをポーリング
      pollForToken(response.device_code);
    } catch (err) {
      setError(`認証開始エラー: ${String(err)}`);
      setLoading(false);
    }
  };

  const pollForToken = async (device_code: string) => {
    try {
      await invoke<string>('poll_twitch_token', { deviceCode: device_code });

      // 成功
      await checkTokens();
      setSuccess('Twitch認証に成功しました！');
      setLoading(false);
      setDeviceCode(null);

      if (onClose) {
        setTimeout(() => onClose(), 2000);
      }
    } catch (err) {
      setError(`認証エラー: ${String(err)}`);
      setLoading(false);
      setDeviceCode(null);
    }
  };

  const handleClose = () => {
    setDeviceCode(null);
    setError(null);
    setSuccess(null);
    if (onClose) {
      onClose();
    }
  };

  const handleCopyCode = () => {
    if (deviceCode) {
      navigator.clipboard.writeText(deviceCode.user_code);
      setSuccess('コードをコピーしました');
      setTimeout(() => setSuccess(null), 2000);
    }
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
      {deviceCode && (
        <div className="p-5 border-2 border-purple-500 dark:border-purple-600 rounded-lg bg-gradient-to-br from-purple-50 to-white dark:from-purple-900/10 dark:to-slate-800">
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="text-2xl">🔐</div>
              <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                認証コード
              </h3>
            </div>

            <div className="space-y-3">
              <div className="p-4 bg-white dark:bg-slate-700 rounded border border-gray-300 dark:border-slate-600">
                <p className="text-xs text-gray-600 dark:text-gray-400 mb-1">コード:</p>
                <div className="flex items-center justify-between">
                  <p className="text-2xl font-mono font-bold text-purple-600 dark:text-purple-400">
                    {deviceCode.user_code}
                  </p>
                  <button
                    onClick={handleCopyCode}
                    className="px-3 py-1 text-xs bg-gray-200 hover:bg-gray-300 dark:bg-slate-600 dark:hover:bg-slate-500 rounded transition-colors"
                  >
                    コピー
                  </button>
                </div>
              </div>

              <div className="text-sm text-gray-600 dark:text-gray-400 space-y-2">
                <p>1. ブラウザで以下のURLを開きます（自動で開きます）:</p>
                <a
                  href={deviceCode.verification_uri}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="block text-purple-600 dark:text-purple-400 underline break-all"
                >
                  {deviceCode.verification_uri}
                </a>
                <p>2. 上記のコードを入力してTwitchでログインしてください</p>
                <p>3. 認証が完了すると自動的にトークンが保存されます</p>
              </div>

              <div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400">
                <span className="animate-pulse">⏳</span>
                <span>認証を待っています...</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 認証開始ボタン */}
      {!deviceCode && (
        <div className="p-5 border border-gray-300 dark:border-slate-600 rounded-lg bg-gray-50 dark:bg-slate-800">
          <div className="space-y-4">
            <div className="flex items-center space-x-2">
              <div className="text-2xl">🎮</div>
              <h3 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                Twitchアカウントでログイン
              </h3>
            </div>

            <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
              ブラウザでTwitchにログインして、安全に認証します。
              クライアントシークレット不要のデバイスコードフローを使用します。
            </p>

            <button
              onClick={handleStartAuth}
              disabled={loading}
              className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-semibold shadow-md hover:shadow-lg"
            >
              {loading ? (
                <span className="flex items-center justify-center space-x-2">
                  <span className="animate-spin">⏳</span>
                  <span>認証開始中...</span>
                </span>
              ) : (
                'Twitchで認証'
              )}
            </button>

            <div className="text-xs text-gray-500 dark:text-gray-400 space-y-1">
              <p>✓ 安全なDevice Code Flow</p>
              <p>✓ Client Secret不要</p>
              <p>✓ リフレッシュトークン対応</p>
            </div>
          </div>
        </div>
      )}

      {/* 補足説明 */}
      <div className="text-xs text-gray-500 dark:text-gray-400 p-3 bg-blue-50 dark:bg-blue-900/10 rounded-lg border border-blue-200 dark:border-blue-800">
        <p className="font-semibold text-blue-700 dark:text-blue-400 mb-1">💡 Device Code Flowについて</p>
        <p>
          デスクトップアプリ向けの安全な認証方式です。
          クライアントシークレットを埋め込む必要がなく、ブラウザで認証を完結できます。
        </p>
      </div>
    </div>
  );
}
