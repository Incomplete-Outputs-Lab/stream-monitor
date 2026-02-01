interface ErrorStateProps {
  error: Error | string;
}

/**
 * エラー状態を表示する共通コンポーネント
 */
export function ErrorState({ error }: ErrorStateProps) {
  const errorMessage = typeof error === 'string' ? error : error.message;

  return (
    <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
      <p className="text-red-800 dark:text-red-200">
        <span className="font-semibold">Error:</span> {errorMessage}
      </p>
    </div>
  );
}
