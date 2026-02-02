import React from 'react';
import { CategoryChange, TitleChange } from '../../types';

interface EventMarkersProps {
  categoryChanges: CategoryChange[];
  titleChanges: TitleChange[];
}

const EventMarkers: React.FC<EventMarkersProps> = ({
  categoryChanges,
  titleChanges,
}) => {
  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      {/* カテゴリ変更 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white flex items-center">
          <span className="w-3 h-3 bg-orange-500 rounded-full mr-2"></span>
          カテゴリ変更 ({categoryChanges.length}回)
        </h3>
        {categoryChanges.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">カテゴリ変更はありません</p>
        ) : (
          <div className="space-y-3 max-h-60 overflow-y-auto">
            {categoryChanges.map((change, index) => (
              <div
                key={`category-${change.timestamp}-${index}`}
                className="border-l-4 border-orange-500 pl-4 py-2"
              >
                <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-1">
                  <span className="font-mono">{formatTime(change.timestamp)}</span>
                </div>
                <div className="text-sm">
                  {change.from_category && (
                    <span className="text-gray-500 dark:text-gray-400 line-through mr-2">
                      {change.from_category}
                    </span>
                  )}
                  <span className="text-gray-900 dark:text-white font-medium">
                    → {change.to_category}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* タイトル変更 */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white flex items-center">
          <span className="w-3 h-3 bg-blue-500 rounded-full mr-2"></span>
          タイトル変更 ({titleChanges.length}回)
        </h3>
        {titleChanges.length === 0 ? (
          <p className="text-gray-500 dark:text-gray-400 text-sm">タイトル変更はありません</p>
        ) : (
          <div className="space-y-3 max-h-60 overflow-y-auto">
            {titleChanges.map((change, index) => (
              <div
                key={`title-${change.timestamp}-${index}`}
                className="border-l-4 border-blue-500 pl-4 py-2"
              >
                <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 mb-1">
                  <span className="font-mono">{formatTime(change.timestamp)}</span>
                </div>
                <div className="text-sm">
                  {change.from_title && (
                    <div className="text-gray-500 dark:text-gray-400 line-through mb-1">
                      {change.from_title}
                    </div>
                  )}
                  <div className="text-gray-900 dark:text-white font-medium">
                    → {change.to_title}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default EventMarkers;
