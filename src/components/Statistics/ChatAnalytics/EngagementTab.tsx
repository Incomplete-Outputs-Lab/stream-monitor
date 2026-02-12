import { useQuery } from '@tanstack/react-query';
import { LineChart } from '../../common/charts';
import { StatsDashboardSkeleton, TableSkeleton } from '../../common/Skeleton';
import { getChatEngagementTimeline, detectChatSpikes } from '../../../api/statistics';

interface EngagementTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const EngagementTab = ({ channelId, startTime, endTime }: EngagementTabProps) => {
  // エンゲージメントタイムライン取得
  const { data: timelineData, isLoading: timelineLoading } = useQuery({
    queryKey: ['chatEngagementTimeline', channelId, startTime, endTime],
    queryFn: () => getChatEngagementTimeline({
      channelId: channelId ?? undefined,
      startTime,
      endTime,
      intervalMinutes: 5,
    }),
    enabled: !!channelId,
  });

  // チャットスパイク検出
  const { data: spikesData, isLoading: spikesLoading } = useQuery({
    queryKey: ['chatSpikes', channelId, startTime, endTime],
    queryFn: () => detectChatSpikes({
      channelId: channelId ?? undefined,
      startTime,
      endTime,
      minSpikeRatio: 2.0,
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
              エンゲージメント分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (timelineLoading) {
    return <StatsDashboardSkeleton cardCount={3} chartCount={2} />;
  }

  // Summary Cards
  const totalChats = timelineData?.reduce((sum, d) => sum + (d.chatCount || 0), 0) || 0;
  const avgEngagement = timelineData?.length
    ? timelineData.reduce((sum, d) => sum + (d.engagementRate || 0), 0) / timelineData.length
    : 0;
  const peakChatters = Math.max(...(timelineData?.map((d) => d.uniqueChatters || 0) || [0]));

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">総チャット数</h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {totalChats.toLocaleString()}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            平均エンゲージメント率
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {(avgEngagement || 0).toFixed(2)}%
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            ピークチャッター数
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {peakChatters.toLocaleString()}
          </p>
        </div>
      </div>

      {/* Engagement Timeline Chart */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          エンゲージメント推移（5分間隔）
        </h3>
        {timelineData && timelineData.length > 0 ? (
          <LineChart
            data={timelineData.map((d) => ({
              timestamp: new Date(d.timestamp).toLocaleString('ja-JP', {
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
              }),
              チャット数: d.chatCount,
              エンゲージメント率: d.engagementRate,
            }))}
            xKey="timestamp"
            lines={[
              { key: 'チャット数', color: '#3b82f6' },
              { key: 'エンゲージメント率', color: '#10b981', yAxisId: 'right' },
            ]}
            height={400}
          />
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>

      {/* Chat Spikes */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          チャットスパイク検出（2倍以上の急増）
        </h3>
        {spikesLoading ? (
          <TableSkeleton rows={5} columns={4} />
        ) : spikesData && spikesData.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    時刻
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    チャット数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    前回比
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    スパイク倍率
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {spikesData.map((spike, index) => (
                  <tr key={index}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {new Date(spike.timestamp).toLocaleString('ja-JP')}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {spike.chatCount.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {spike.prevCount.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-bold text-red-600 dark:text-red-400">
                      {(spike.spikeRatio || 0).toFixed(2)}x
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            スパイクが検出されませんでした
          </p>
        )}
      </div>
    </div>
  );
};

export default EngagementTab;
