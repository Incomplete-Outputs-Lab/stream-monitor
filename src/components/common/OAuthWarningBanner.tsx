import { useEffect } from 'react';
import { useConfigStore } from '../../stores/configStore';
import { useNavigation } from '../../contexts/NavigationContext';

export function OAuthWarningBanner() {
  const { hasTwitchOAuth, hasTwitchToken, checkTokens } = useConfigStore();
  const { setActiveTab } = useNavigation();

  useEffect(() => {
    checkTokens();
  }, [checkTokens]);

  const handleNavigateToSettings = () => {
    setActiveTab('settings');
  };

  // OAuth設定とトークンの両方が揃っている場合は警告を表示しない
  if (hasTwitchOAuth && hasTwitchToken) {
    return null;
  }

  return (
    <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4 mb-4">
      <div className="flex items-start gap-3">
        <div className="flex-shrink-0">
          <svg
            className="w-6 h-6 text-yellow-600 dark:text-yellow-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        </div>
        <div className="flex-1">
          <h3 className="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-1">
            {!hasTwitchOAuth && !hasTwitchToken
              ? 'Twitch API設定とOAuth認証が必要です'
              : !hasTwitchOAuth
              ? 'Twitch API設定が必要です'
              : 'Twitch OAuth認証が必要です'}
          </h3>
          <p className="text-xs text-yellow-700 dark:text-yellow-400 mb-3">
            {!hasTwitchOAuth && !hasTwitchToken
              ? 'Twitchの配信データを収集するには、Client IDの設定とOAuth認証を完了してください。'
              : !hasTwitchOAuth
              ? 'Twitchの配信データを収集するには、Client IDを設定してください。'
              : 'Twitchの配信データを収集するには、OAuth認証を完了してください。'}
          </p>
          <button
            onClick={handleNavigateToSettings}
            className="inline-flex items-center px-3 py-1.5 text-xs font-medium text-yellow-800 dark:text-yellow-200 bg-yellow-100 dark:bg-yellow-900/40 hover:bg-yellow-200 dark:hover:bg-yellow-900/60 rounded-md transition-colors"
          >
            <svg
              className="w-4 h-4 mr-1.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
            設定ページへ移動
          </button>
        </div>
      </div>
    </div>
  );
}
