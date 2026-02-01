import { LoadingSpinner } from './LoadingSpinner';

interface LoadingStateProps {
  message?: string;
}

/**
 * ローディング状態を表示する共通コンポーネント
 */
export function LoadingState({ message = 'Loading...' }: LoadingStateProps) {
  return (
    <div className="flex items-center justify-center p-8">
      <LoadingSpinner />
      <span className="ml-2 text-gray-600 dark:text-gray-300">{message}</span>
    </div>
  );
}
