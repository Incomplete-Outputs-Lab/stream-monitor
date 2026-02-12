import { useQuery } from '@tanstack/react-query';
import { PieChart, BarChart } from '../../common/charts';
import { StatsDashboardSkeleton } from '../../common/Skeleton';
import { getUserSegmentStats } from '../../../api/statistics';

interface UserSegmentTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const UserSegmentTab = ({ channelId, startTime, endTime }: UserSegmentTabProps) => {
  const { data: segmentData, isLoading } = useQuery({
    queryKey: ['userSegmentStats', channelId, startTime, endTime],
    queryFn: () => getUserSegmentStats({
      channelId: channelId ?? undefined,
      startTime,
      endTime,
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
              ユーザーセグメント分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <StatsDashboardSkeleton cardCount={3} chartCount={2} />;
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
            {(segmentData?.find((s) => s.segment === 'subscriber')?.percentage || 0).toFixed(1)}%
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
              平均発言数: Number((s.avgMessagesPerUser || 0).toFixed(2)),
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
                      {(segment.avgMessagesPerUser || 0).toFixed(2)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {(segment.percentage || 0).toFixed(2)}%
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
