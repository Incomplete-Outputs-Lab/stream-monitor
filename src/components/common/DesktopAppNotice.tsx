export function DesktopAppNotice() {
  return (
    <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
      <p className="text-xs text-gray-600 dark:text-gray-400">
        ※ デスクトップアプリのため、アプリ起動中のみデータが収集されます。期間中の連続データではない場合があります。
      </p>
    </div>
  );
}
