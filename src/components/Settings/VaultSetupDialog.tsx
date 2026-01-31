import { useState } from 'react';
import { initializeStronghold } from '../../utils/stronghold';

interface VaultSetupDialogProps {
  onComplete: () => void;
}

export function VaultSetupDialog({ onComplete }: VaultSetupDialogProps) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSetup = async () => {
    setError('');

    // Validation
    if (password.length < 8) {
      setError('パスワードは8文字以上である必要があります');
      return;
    }

    if (password !== confirmPassword) {
      setError('パスワードが一致しません');
      return;
    }

    setIsLoading(true);

    try {
      await initializeStronghold(password);
      onComplete();
    } catch (err) {
      console.error('Vault setup failed:', err);
      setError(`Vaultの初期化に失敗しました: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md">
        <h2 className="text-2xl font-bold mb-4 text-gray-900 dark:text-white">
          Vaultのセットアップ
        </h2>
        
        <p className="text-gray-700 dark:text-gray-300 mb-6">
          認証情報を安全に保存するため、Vaultのパスワードを設定してください。
          このパスワードは暗号化に使用され、アプリケーションには保存されません。
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
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="最低8文字"
              disabled={isLoading}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              パスワード（確認）
            </label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="再度入力してください"
              disabled={isLoading}
            />
          </div>

          {error && (
            <div className="text-red-600 dark:text-red-400 text-sm">
              {error}
            </div>
          )}

          <button
            onClick={handleSetup}
            disabled={isLoading || !password || !confirmPassword}
            className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-md transition-colors"
          >
            {isLoading ? 'セットアップ中...' : 'Vaultを作成'}
          </button>
        </div>

        <div className="mt-4 text-xs text-gray-600 dark:text-gray-400">
          <p>注意:</p>
          <ul className="list-disc list-inside mt-1 space-y-1">
            <li>パスワードを忘れると、保存されたデータにアクセスできなくなります</li>
            <li>パスワードは安全な場所に記録してください</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
