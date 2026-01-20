import { useState } from 'react';
import { useConfigStore } from '../../stores/configStore';

interface TokenFormProps {
  platform: 'twitch' | 'youtube';
  onClose?: () => void;
}

export function TokenForm({ platform, onClose }: TokenFormProps) {
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const { saveToken, deleteToken, checkTokens } = useConfigStore();

  const platformName = platform === 'twitch' ? 'Twitch' : 'YouTube';

  const handleSave = async () => {
    if (!token.trim()) {
      setError('トークンを入力してください');
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await saveToken(platform, token.trim());
      setSuccess(`${platformName}トークンを保存しました`);
      await checkTokens();
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await deleteToken(platform);
      setSuccess(`${platformName}トークンを削除しました`);
      setToken('');
      await checkTokens();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setToken('');
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
          {platformName} トークン設定
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

      <div className="space-y-3">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          アクセストークン
          <input
            type="password"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            className="input-field mt-1"
            placeholder={`${platformName}のアクセストークンを入力`}
            disabled={loading}
          />
        </label>

        <div className="flex space-x-3">
          <button
            onClick={handleSave}
            disabled={loading || !token.trim()}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-medium"
          >
            {loading ? '保存中...' : '保存'}
          </button>
          <button
            onClick={handleDelete}
            disabled={loading}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-gray-400 text-white rounded-lg transition-colors text-sm font-medium"
          >
            {loading ? '削除中...' : '削除'}
          </button>
        </div>

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