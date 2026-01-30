import { useState, useEffect } from 'react';
import { useConfigStore } from '../../stores/configStore';

interface OAuthConfigFormProps {
  platform: 'twitch' | 'youtube';
  onClose?: () => void;
}

export function OAuthConfigForm({ platform, onClose }: OAuthConfigFormProps) {
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const { getOAuthConfig, saveOAuthConfig, deleteOAuthConfig } = useConfigStore();

  const platformName = platform === 'twitch' ? 'Twitch' : 'YouTube';

  // コンポーネントマウント時に既存設定を読み込み
  useEffect(() => {
    const loadExistingConfig = async () => {
      try {
        const config = await getOAuthConfig(platform);
        if (config.client_id) {
          setClientId(config.client_id);
        }
        if (config.client_secret) {
          setClientSecret(config.client_secret);
        }
      } catch (err) {
        // 設定が存在しない場合は何もしない（新規設定時）
        console.log(`${platformName} OAuth config not found, starting fresh`);
      }
    };

    loadExistingConfig();
  }, [platform, getOAuthConfig, platformName]);

  const handleSave = async () => {
    // Client IDは必須
    if (!clientId.trim()) {
      setError('Client ID は必須です');
      return;
    }

    // YouTubeの場合、Client Secretは必須
    if (platform === 'youtube' && !clientSecret.trim()) {
      setError('YouTube OAuth では Client ID と Client Secret の両方が必要です');
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      // Client Secretが空でない場合のみ送信
      const secret = clientSecret.trim() || undefined;
      await saveOAuthConfig(platform, clientId.trim(), secret);
      setSuccess(`${platformName} OAuth設定を保存しました`);
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      const errorMessage = String(err);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`${platformName}のOAuth設定を削除しますか？\n\n注意: この操作により、保存されたClient IDとClient Secretが完全に削除されます。`)) {
      return;
    }

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await deleteOAuthConfig(platform);
      setSuccess(`${platformName} OAuth設定を削除しました`);
      setClientId('');
      setClientSecret('');
      if (onClose) {
        setTimeout(() => onClose(), 2000); // 2秒後に閉じる
      }
    } catch (err) {
      const errorMessage = String(err);
      setError(errorMessage);
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
          {platformName} OAuth設定
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

      <div className="text-sm text-gray-600 dark:text-gray-400 space-y-2">
        {platform === 'twitch' ? (
          <>
            <p>
              Twitch APIを使用するには、Twitch Developer ConsoleでOAuthアプリケーションを作成し、Client IDを取得してください。
            </p>
            <div className="text-xs space-y-2 p-3 bg-blue-50 dark:bg-blue-900/10 rounded border border-blue-200 dark:border-blue-800">
              <p className="font-semibold text-blue-700 dark:text-blue-400">📝 設定手順：</p>
              <div className="space-y-1 pl-2">
                <p>1. <a href="https://dev.twitch.tv/console/apps" target="_blank" rel="noopener noreferrer" className="underline hover:text-blue-600 font-medium">Twitch Developer Console</a> でアプリを作成または編集</p>
                <p>2. <span className="font-medium">Category</span> を選択（例：Application Integration）</p>
                <p>3. <span className="font-medium">Client ID</span> をコピーして下記に貼り付け</p>
              </div>
            </div>
            <div className="text-xs space-y-1 p-2 bg-green-50 dark:bg-green-900/10 rounded border border-green-200 dark:border-green-800">
              <p className="font-semibold text-green-700 dark:text-green-400">✅ Device Code認証について：</p>
              <p className="text-green-700 dark:text-green-400">
                TwitchはDevice Code Grant Flowを使用するため、<span className="font-semibold">Client Secretは不要</span>です。
                Client IDのみ設定すれば、安全に認証できます。
              </p>
            </div>
          </>
        ) : (
          `${platformName} APIを使用するには、${platformName} Developer ConsoleでOAuthアプリケーションを作成し、Client IDとClient Secretを取得してください。`
        )}
      </div>

      <div className="space-y-4">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Client ID
          <input
            type="text"
            value={clientId}
            onChange={(e) => setClientId(e.target.value)}
            onBlur={(e) => setClientId(e.target.value.trim())}
            className="input-field mt-1"
            placeholder={`${platformName} Client IDを入力`}
            disabled={loading}
          />
          {clientId && (
            <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              現在の設定: {clientId.substring(0, 8)}... (長さ: {clientId.length}文字)
            </div>
          )}
        </label>
        {platform !== 'twitch' && (
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Client Secret
            <input
              type="password"
              value={clientSecret}
              onChange={(e) => setClientSecret(e.target.value)}
              onBlur={(e) => setClientSecret(e.target.value.trim())}
              className="input-field mt-1"
              placeholder={`${platformName} Client Secretを入力`}
              disabled={loading}
            />
          </label>
        )}

        <div className="flex space-x-3">
          <button
            onClick={handleSave}
            disabled={loading || !clientId.trim() || (platform === 'youtube' && !clientSecret.trim())}
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