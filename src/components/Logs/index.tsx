import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

interface GetLogsQuery {
  level?: string;
  search?: string;
  limit?: number;
}

export function Logs() {
  const [selectedLevel, setSelectedLevel] = useState<string>("");
  const [searchText, setSearchText] = useState<string>("");

  const { data: logs, isLoading, refetch } = useQuery({
    queryKey: ["logs", selectedLevel, searchText],
    queryFn: async () => {
      const query: GetLogsQuery = {
        limit: 500,
      };
      if (selectedLevel) query.level = selectedLevel;
      if (searchText) query.search = searchText;
      
      return await invoke<LogEntry[]>("get_logs", { query });
    },
    refetchInterval: 10000, // 10秒ごとに自動更新
  });

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case "ERROR":
        return "text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20";
      case "WARN":
        return "text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20";
      case "INFO":
        return "text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20";
      default:
        return "text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/20";
    }
  };

  const getLevelIcon = (level: string) => {
    switch (level.toUpperCase()) {
      case "ERROR":
        return "❌";
      case "WARN":
        return "⚠️";
      case "INFO":
        return "ℹ️";
      default:
        return "📝";
    }
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">ログビューア</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">アプリケーションのログを確認</p>
      </div>

      {/* フィルター */}
      <div className="card p-4">
        <div className="flex flex-col md:flex-row gap-4">
          <div className="flex-1">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              検索
            </label>
            <input
              type="text"
              placeholder="ログメッセージを検索..."
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              レベル
            </label>
            <select
              value={selectedLevel}
              onChange={(e) => setSelectedLevel(e.target.value)}
              className="px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">すべて</option>
              <option value="INFO">INFO</option>
              <option value="WARN">WARN</option>
              <option value="ERROR">ERROR</option>
            </select>
          </div>
          <div className="flex items-end">
            <button
              onClick={() => refetch()}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              更新
            </button>
          </div>
        </div>
      </div>

      {/* ログ表示 */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">ログエントリ</h3>
          {logs && (
            <span className="text-xs font-medium text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-slate-700 px-2 py-1 rounded-full">
              {logs.length}件
            </span>
          )}
        </div>

        {isLoading ? (
          <div className="text-center py-12">
            <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600 mx-auto"></div>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-3 font-medium">読み込み中...</p>
          </div>
        ) : logs && logs.length > 0 ? (
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {logs.slice().reverse().map((log, index) => (
              <div
                key={index}
                className="border border-gray-200 dark:border-slate-700 rounded-lg p-3 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors"
              >
                <div className="flex items-start gap-3">
                  <span className="text-lg">{getLevelIcon(log.level)}</span>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className={`text-xs font-semibold px-2 py-1 rounded ${getLevelColor(log.level)}`}>
                        {log.level}
                      </span>
                      <span className="text-xs text-gray-500 dark:text-gray-400">
                        {log.timestamp}
                      </span>
                    </div>
                    <p className="text-sm text-gray-700 dark:text-gray-300 break-words">
                      {log.message}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
              <svg
                className="w-8 h-8 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                />
              </svg>
            </div>
            <p className="text-gray-500 dark:text-gray-400 font-medium">ログエントリが見つかりません</p>
            <p className="text-sm text-gray-400 dark:text-gray-500 mt-1">
              {searchText || selectedLevel ? "フィルター条件を変更してください" : "ログファイルが空です"}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}