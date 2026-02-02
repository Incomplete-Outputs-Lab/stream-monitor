import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { UserSegmentStats } from '../../../types';
import { PieChart, BarChart } from '../../common/charts';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface UserSegmentTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const UserSegmentTab = ({ channelId, startTime, endTime }: UserSegmentTabProps) => {
  const { data: segmentData, isLoading } = useQuery({
    queryKey: ['userSegmentStats', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<UserSegmentStats[]>('get_user_segment_stats', {
        channelId,
        streamId: null,
        startTime,
        endTime,
      });
      return result;
    },
  });

  if (isLoading) {
    return <LoadingSpinner />;
  }

  const segmentLabels: Record<string, string> = {
    broadcaster: '配信者',
    moderator: 'モデレーター',
    vip: 'VIP',
    subscriber: 'サブスクライバー',
    regular: '一般',
  };

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            総メッセージ数
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {segmentData
              ?.reduce((sum, s) => sum + s.messageCount, 0)
              .toLocaleString() || 0}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            総ユーザー数
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {segmentData
              ?.reduce((sum, s) => sum + s.userCount, 0)
              .toLocaleString() || 0}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            サブスク率
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {segmentData?.find((s) => s.segment === 'subscriber')?.percentage.toFixed(1) ||
              0}
            %
          </p>
        </div>
      </div>

      {/* Segment Distribution Pie Chart */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          ユーザーセグメント別メッセージ割合
        </h3>
        {segmentData && segmentData.length > 0 ? (
          <PieChart
            data={segmentData.map((s) => ({
              name: segmentLabels[s.segment] || s.segment,
              value: s.messageCount,
              percentage: s.percentage,
            }))}
            dataKey="value"
            nameKey="name"
            height={400}
          />
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>

      {/* Average Messages per User */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          セグメント別平均発言数
        </h3>
        {segmentData && segmentData.length > 0 ? (
          <BarChart
            data={segmentData.map((s) => ({
              segment: segmentLabels[s.segment] || s.segment,
              平均発言数: Number(s.avgMessagesPerUser.toFixed(2)),
            }))}
            xKey="segment"
            bars={[{ key: '平均発言数', color: '#3b82f6' }]}
            height={350}
          />
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>

      {/* Detailed Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          詳細データ
        </h3>
        {segmentData && segmentData.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    セグメント
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    メッセージ数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    ユーザー数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    平均発言数/人
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    割合
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {segmentData.map((segment) => (
                  <tr key={segment.segment}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                      {segmentLabels[segment.segment] || segment.segment}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {segment.messageCount.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {segment.userCount.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {segment.avgMessagesPerUser.toFixed(2)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {segment.percentage.toFixed(2)}%
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

export default UserSegmentTab;
