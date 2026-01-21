export function Statistics() {
  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">統計閲覧</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">過去の統計データを閲覧・分析</p>
      </div>

      <div className="card p-12 text-center">
        <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
          <svg
            className="w-10 h-10 text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            />
          </svg>
        </div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
          実装予定の機能
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          統計閲覧機能は現在開発中です。過去の統計データを閲覧・分析できる機能を予定しています。
        </p>
        <div className="text-sm text-yellow-600 dark:text-yellow-400 font-medium">
          🔄 開発中 - 今後のアップデートをお待ちください
        </div>
      </div>
    </div>
  );
}