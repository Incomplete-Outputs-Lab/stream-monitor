interface DateRangePickerProps {
  startDate: string;
  endDate: string;
  onChange: (start: string, end: string) => void;
}

export function DateRangePicker({ startDate, endDate, onChange }: DateRangePickerProps) {
  const handleStartChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value, endDate);
  };

  const handleEndChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(startDate, e.target.value);
  };

  // プリセット日付範囲
  const setDateRange = (days: number) => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);

    onChange(
      start.toISOString().split('T')[0],
      end.toISOString().split('T')[0]
    );
  };

  return (
    <div className="flex items-center space-x-4">
      {/* プリセットボタン */}
      <div className="flex space-x-2">
        <button
          onClick={() => setDateRange(1)}
          className="px-3 py-1 text-sm bg-gray-100 dark:bg-slate-700 hover:bg-gray-200 dark:hover:bg-slate-600 text-gray-900 dark:text-gray-100 rounded-md transition-colors"
        >
          今日
        </button>
        <button
          onClick={() => setDateRange(7)}
          className="px-3 py-1 text-sm bg-gray-100 dark:bg-slate-700 hover:bg-gray-200 dark:hover:bg-slate-600 text-gray-900 dark:text-gray-100 rounded-md transition-colors"
        >
          7日間
        </button>
        <button
          onClick={() => setDateRange(30)}
          className="px-3 py-1 text-sm bg-gray-100 dark:bg-slate-700 hover:bg-gray-200 dark:hover:bg-slate-600 text-gray-900 dark:text-gray-100 rounded-md transition-colors"
        >
          30日間
        </button>
      </div>

      {/* 日付入力 */}
      <div className="flex items-center space-x-2">
        <input
          type="date"
          value={startDate}
          onChange={handleStartChange}
          className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
        />
        <span className="text-gray-500 dark:text-gray-400">〜</span>
        <input
          type="date"
          value={endDate}
          onChange={handleEndChange}
          className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
        />
      </div>
    </div>
  );
}