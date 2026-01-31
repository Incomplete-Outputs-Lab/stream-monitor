import { useState } from 'react';
import { initializeStronghold } from '../../utils/stronghold';

interface VaultUnlockDialogProps {
  onUnlock: () => void;
  onCancel?: () => void;
}

export function VaultUnlockDialog({ onUnlock, onCancel }: VaultUnlockDialogProps) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleUnlock = async () => {
    setError('');
    setIsLoading(true);

    try {
      await initializeStronghold(password);
      onUnlock();
    } catch (err) {
      console.error('Vault unlock failed:', err);
      setError('パスワードが正しくありません');
      setPassword('');
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && password) {
      handleUnlock();
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
        <h2 className="text-2xl font-bold mb-4 text-gray-900 dark:text-white">
          Vaultのアンロック
        </h2>
        
        <p className="text-gray-700 dark:text-gray-300 mb-6">
          保存された認証情報にアクセスするため、Vaultのパスワードを入力してください。
        </p>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              パスワード
            </label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              onKeyPress={handleKeyPress}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="Vaultのパスワードを入力"
              disabled={isLoading}
              autoFocus
            />
          </div>

          {error && (
            <div className="text-red-600 dark:text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="flex gap-2">
            {onCancel && (
              <button
                onClick={onCancel}
                disabled={isLoading}
                className="flex-1 px-4 py-2 bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 text-gray-900 dark:text-white rounded-md transition-colors"
              >
                キャンセル
              </button>
            )}
            <button
              onClick={handleUnlock}
              disabled={isLoading || !password}
              className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-md transition-colors"
            >
              {isLoading ? 'アンロック中...' : 'アンロック'}
            </button>
          </div>
        </div>

        <div className="mt-4 text-xs text-gray-600 dark:text-gray-400">
          <p>
            パスワードを忘れた場合、保存されたデータにアクセスできなくなります。
            その場合は、Vaultを再作成する必要があります（既存のデータは失われます）。
          </p>
        </div>
      </div>
    </div>
  );
}
