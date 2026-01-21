export function MultiView() {
  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">マルチビュー</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">複数のストリームを同時に監視・分析</p>
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
              d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
            />
          </svg>
        </div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
          実装予定の機能
        </h3>
        <p className="text-gray-600 dark:text-gray-400 mb-4">
          マルチビュー機能は現在開発中です。複数のストリームを同時に監視・分析できる機能を予定しています。
        </p>
        <div className="text-sm text-yellow-600 dark:text-yellow-400 font-medium">
          🔄 開発中 - 今後のアップデートをお待ちください
        </div>
      </div>
    </div>
  );
}