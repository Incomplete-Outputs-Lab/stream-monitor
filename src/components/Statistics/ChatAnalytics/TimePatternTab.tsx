import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { BarChart } from '../../common/charts';
import { StatsDashboardSkeleton } from '../../common/Skeleton';
import { getTimePatternStats } from '../../../api/statistics';

interface TimePatternTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const TimePatternTab = ({ channelId, startTime, endTime }: TimePatternTabProps) => {
  const [groupByDay, setGroupByDay] = useState(false);

  const { data: patternData, isLoading } = useQuery({
    queryKey: ['timePatternStats', channelId, startTime, endTime, groupByDay],
    queryFn: () => getTimePatternStats({
      channelId: channelId ?? undefined,
      startTime,
      endTime,
      groupByDay,
    }),
    enabled: !!channelId,
  });

  // チャンネル選択チェック
  if (channelId === null) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <svg className="h-6 w-6 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
              チャンネルを選択してください
            </h3>
            <p className="mt-2 text-sm text-yellow-700 dark:text-yellow-300">
              時間帯パターン分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <StatsDashboardSkeleton cardCount={3} chartCount={1} />;
  }

  const dayLabels = ['日', '月', '火', '水', '木', '金', '土'];

  // Group data for heatmap if groupByDay is true
  const heatmapData = groupByDay && patternData
    ? Array.from({ length: 7 }, (_, dayIndex) => {
        const dayData = patternData.filter((p) => p.dayOfWeek === dayIndex);
        return {
          day: dayLabels[dayIndex],
          data: Array.from({ length: 24 }, (_, hourIndex) => {
            const hourData = dayData.find((p) => p.hour === hourIndex);
            return hourData ? hourData.totalMessages : 0;
          }),
        };
      })
    : null;

  return (
    <div className="space-y-6">
      {/* Toggle */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow">
        <label className="flex items-center space-x-3">
          <input
            type="checkbox"
            checked={groupByDay}
            onChange={(e) => setGroupByDay(e.target.checked)}
            className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 dark:border-gray-600 dark:bg-gray-700 dark:focus:ring-gray-600"
          />
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            曜日別に表示
          </span>
        </label>
      </div>

      {/* Hourly Pattern */}
      {!groupByDay && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            時間帯別チャット数
          </h3>
          {patternData && patternData.length > 0 ? (
            <BarChart
              data={patternData.map((p) => ({
                時間: `${p.hour}:00`,
                チャット数: p.totalMessages,
                平均エンゲージメント: Number((p.avgEngagement || 0).toFixed(2)),
              }))}
              xKey="時間"
              bars={[
                { key: 'チャット数', color: '#3b82f6' },
                { key: '平均エンゲージメント', color: '#10b981' },
              ]}
              height={400}
            />
          ) : (
            <p className="text-gray-500 dark:text-gray-400 text-center py-8">
              データがありません
            </p>
          )}
        </div>
      )}

      {/* Day/Hour Heatmap */}
      {groupByDay && heatmapData && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
            曜日×時間帯ヒートマップ
          </h3>
          <div className="overflow-x-auto">
            <table className="min-w-full">
              <thead>
                <tr>
                  <th className="px-2 py-2 text-xs font-medium text-gray-500 dark:text-gray-400">
                    曜日
                  </th>
                  {Array.from({ length: 24 }, (_, i) => (
                    <th
                      key={i}
                      className="px-2 py-2 text-xs font-medium text-gray-500 dark:text-gray-400"
                    >
                      {i}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {heatmapData.map((dayRow) => {
                  const maxValue = Math.max(...dayRow.data);
                  return (
                    <tr key={dayRow.day}>
                      <td className="px-2 py-2 text-xs font-medium text-gray-700 dark:text-gray-300">
                        {dayRow.day}
                      </td>
                      {dayRow.data.map((value, hourIndex) => {
                        const intensity = maxValue > 0 ? value / maxValue : 0;
                        return (
                          <td
                            key={hourIndex}
                            className="px-2 py-2 text-center text-xs"
                            style={{
                              backgroundColor: `rgba(59, 130, 246, ${intensity})`,
                              color: intensity > 0.5 ? 'white' : 'inherit',
                            }}
                            title={`${dayRow.day} ${hourIndex}:00 - ${value.toLocaleString()}件`}
                          >
                            {value > 0 ? value : ''}
                          </td>
                        );
                      })}
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Detailed Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          詳細データ
        </h3>
        {patternData && patternData.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  {groupByDay && (
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      曜日
                    </th>
                  )}
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    時間
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    総チャット数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    平均チャット率
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    平均エンゲージメント
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {patternData.map((pattern, index) => (
                  <tr key={index}>
                    {groupByDay && pattern.dayOfWeek !== undefined && (
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                        {dayLabels[pattern.dayOfWeek]}
                      </td>
                    )}
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {pattern.hour}:00
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {pattern.totalMessages.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {(pattern.avgChatRate || 0).toFixed(2)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {(pattern.avgEngagement || 0).toFixed(2)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>
    </div>
  );
};

export default TimePatternTab;
