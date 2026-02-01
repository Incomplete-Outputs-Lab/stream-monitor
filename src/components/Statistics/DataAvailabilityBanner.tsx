import { DataAvailability } from "../../types";

interface DataAvailabilityBannerProps {
  availability: DataAvailability;
}

export function DataAvailabilityBanner({ availability }: DataAvailabilityBannerProps) {
  const formatDate = (dateStr: string) => {
    if (!dateStr) return '-';
    const date = new Date(dateStr);
    return date.toLocaleDateString('ja-JP', { year: 'numeric', month: '2-digit', day: '2-digit' });
  };

  return (
    <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-6 text-sm">
          <div>
            <span className="text-gray-600 dark:text-gray-400">データ収集期間: </span>
            <span className="font-semibold text-gray-900 dark:text-gray-100">
              {formatDate(availability.first_record)} 〜 {formatDate(availability.last_record)}
            </span>
          </div>
          <div>
            <span className="text-gray-600 dark:text-gray-400">データがある日数: </span>
            <span className="font-semibold text-gray-900 dark:text-gray-100">
              {availability.total_days_with_data}日分
            </span>
          </div>
          <div>
            <span className="text-gray-600 dark:text-gray-400">総レコード数: </span>
            <span className="font-semibold text-gray-900 dark:text-gray-100">
              {availability.total_records.toLocaleString()}
            </span>
          </div>
        </div>
      </div>
      <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
        ※ デスクトップアプリのため、アプリ起動中のみデータが収集されます。期間中の連続データではない場合があります。
      </p>
    </div>
  );
}
