import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';

interface OAuthLoginProps {
  platform: 'twitch' | 'youtube';
  onClose?: () => void;
}

export function OAuthLogin({ platform, onClose }: OAuthLoginProps) {
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const { checkTokens } = useConfigStore();

  const platformName = platform === 'twitch' ? 'Twitch' : 'YouTube';

  const handleLogin = async () => {
    if (!clientId.trim() || !clientSecret.trim()) {
      setError('Client ID と Client Secret を設定してください');
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      // ブラウザが自動で開かれることを通知
      setSuccess(`ブラウザで${platformName}認証ページを開いています...`);

      const command = platform === 'twitch' ? 'login_with_twitch' : 'login_with_youtube';
      await invoke<string>(command, {
        config: {
          client_id: clientId.trim(),
          client_secret: clientSecret.trim(),
        },
        port: platform === 'twitch' ? 8080 : 8081,
      });

      // トークンの存在を確認
      await checkTokens();
      setSuccess(`${platformName}に接続しました！`);
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      const errorMessage = String(err);
      if (errorMessage.includes('timeout') || errorMessage.includes('cancelled')) {
        setError('認証がタイムアウトしました。再度お試しください。');
      } else {
        setError(errorMessage);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setClientId('');
    setClientSecret('');
    setError(null);
    setSuccess(null);
    if (onClose) {
      onClose();
    }
  };

  return (
    <div className="space-y-4 p-4 border border-gray-200 dark:border-slate-600 rounded-lg bg-gray-50 dark:bg-slate-800">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {platformName} OAuth認証
        </h3>
        {onClose && (
          <button
            onClick={handleClose}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            ✕
          </button>
        )}
      </div>

      <div className="space-y-4">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Client ID
          <input
            type="text"
            value={clientId}
            onChange={(e) => setClientId(e.target.value)}
            className="input-field mt-1"
            placeholder={`${platformName} Client IDを入力`}
            disabled={loading}
          />
        </label>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Client Secret
          <input
            type="password"
            value={clientSecret}
            onChange={(e) => setClientSecret(e.target.value)}
            className="input-field mt-1"
            placeholder={`${platformName} Client Secretを入力`}
            disabled={loading}
          />
        </label>

        <button
          onClick={handleLogin}
          disabled={loading || !clientId.trim() || !clientSecret.trim()}
          className="w-full px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-400 text-white rounded-lg transition-colors font-medium"
        >
          {loading ? `${platformName}に接続中...` : `${platformName}でログイン`}
        </button>

        {error && (
          <div className="text-red-500 text-sm p-2 bg-red-50 dark:bg-red-900/20 rounded">
            {error}
          </div>
        )}

        {success && (
          <div className="text-green-600 text-sm p-2 bg-green-50 dark:bg-green-900/20 rounded">
            {success}
          </div>
        )}
      </div>
    </div>
  );
}
