import { useQuery } from '@tanstack/react-query';
import { BarChart } from '../../common/charts';
import { StatsDashboardSkeleton } from '../../common/Skeleton';
import { getTopChatters, getChatterBehaviorStats } from '../../../api/statistics';

interface ChatterBehaviorTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const ChatterBehaviorTab = ({ channelId, startTime, endTime }: ChatterBehaviorTabProps) => {
  // Top Chatters
  const { data: topChatters, isLoading: chattersLoading } = useQuery({
    queryKey: ['topChatters', channelId, startTime, endTime],
    queryFn: () => getTopChatters({
      channelId: channelId ?? undefined,
      startTime,
      endTime,
      limit: 50,
    }),
    enabled: !!channelId,
  });

  // Behavior Stats
  const { data: behaviorStats, isLoading: behaviorLoading } = useQuery({
    queryKey: ['chatterBehaviorStats', channelId, startTime, endTime],
    queryFn: () => getChatterBehaviorStats({
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
              チャット者行動分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (chattersLoading || behaviorLoading) {
    return <StatsDashboardSkeleton cardCount={4} chartCount={2} />;
  }

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            総ユニークチャッター
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {behaviorStats?.totalUniqueChatters.toLocaleString() || 0}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            リピーター数
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {behaviorStats?.repeaterCount.toLocaleString() || 0}
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            リピーター率
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {behaviorStats?.repeaterPercentage.toFixed(1) || 0}%
          </p>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400">
            新規チャッター数
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {behaviorStats?.newChatterCount.toLocaleString() || 0}
          </p>
        </div>
      </div>

      {/* Top Chatters Bar Chart */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          上位チャッター（Top 20）
        </h3>
        {topChatters && topChatters.length > 0 ? (
          <BarChart
            data={topChatters.slice(0, 20).map((c) => ({
              user: c.displayName || c.userName,
              発言数: c.messageCount,
            }))}
            xKey="user"
            bars={[{ key: '発言数', color: '#3b82f6' }]}
            height={400}
          />
        ) : (
          <p className="text-gray-500 dark:text-gray-400 text-center py-8">
            データがありません
          </p>
        )}
      </div>

      {/* Top Chatters Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          上位チャッター詳細
        </h3>
        {topChatters && topChatters.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    順位
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    ユーザー名
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    発言数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    バッジ
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    配信参加数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    初回参加
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {topChatters.map((chatter, index) => (
                  <tr key={chatter.userId || chatter.userName}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {index + 1}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                      {chatter.displayName || chatter.userName}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {chatter.messageCount.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      <div className="flex flex-wrap gap-1">
                        {chatter.badges.map((badge) => (
                          <span
                            key={badge}
                            className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
                          >
                            {badge}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {chatter.streamCount}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {new Date(chatter.firstSeen).toLocaleDateString('ja-JP')}
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

export default ChatterBehaviorTab;
