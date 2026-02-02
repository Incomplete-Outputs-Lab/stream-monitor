import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { TopChatter, ChatterBehaviorStats } from '../../../types';
import { BarChart } from '../../common/charts';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface ChatterBehaviorTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const ChatterBehaviorTab = ({ channelId, startTime, endTime }: ChatterBehaviorTabProps) => {
  // Top Chatters
  const { data: topChatters, isLoading: chattersLoading } = useQuery({
    queryKey: ['topChatters', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<TopChatter[]>('get_top_chatters', {
        channelId,
        streamId: null,
        startTime,
        endTime,
        limit: 50,
      });
      return result;
    },
  });

  // Behavior Stats
  const { data: behaviorStats, isLoading: behaviorLoading } = useQuery({
    queryKey: ['chatterBehaviorStats', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<ChatterBehaviorStats>('get_chatter_behavior_stats', {
        channelId,
        startTime,
        endTime,
      });
      return result;
    },
  });

  if (chattersLoading || behaviorLoading) {
    return <LoadingSpinner />;
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
              user: c.userName,
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
                  <tr key={chatter.userName}>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {index + 1}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                      {chatter.userName}
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
