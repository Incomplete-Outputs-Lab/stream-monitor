import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { ChatEngagementStats, ChatSpike } from '../../../types';
import { LineChart } from '../../common/charts';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface EngagementTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const EngagementTab = ({ channelId, startTime, endTime }: EngagementTabProps) => {
  // エンゲージメントタイムライン取得
  const { data: timelineData, isLoading: timelineLoading } = useQuery({
    queryKey: ['chatEngagementTimeline', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<ChatEngagementStats[]>('get_chat_engagement_timeline', {
        channelId,
        streamId: null,
        startTime,
        endTime,
        intervalMinutes: 5,
      });
      return result;
    },
  });

  // チャットスパイク検出
  const { data: spikesData, isLoading: spikesLoading } = useQuery({
    queryKey: ['chatSpikes', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<ChatSpike[]>('detect_chat_spikes', {
        channelId,
        streamId: null,
        startTime,
        endTime,
        minSpikeRatio: 2.0,
      });
      return result;
    },
  });

  if (timelineLoading) {
    return <LoadingSpinner />;
  }

  // Summary Cards
  const totalChats = timelineData?.reduce((sum, d) => sum + d.chatCount, 0) || 0;
  const avgEngagement = timelineData?.length
    ? timelineData.reduce((sum, d) => sum + d.engagementRate, 0) / timelineData.length
    : 0;
  const peakChatters = Math.max(...(timelineData?.map((d) => d.uniqueChatters) || [0]));

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
            {avgEngagement.toFixed(2)}%
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
          <LoadingSpinner />
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
                      {spike.spikeRatio.toFixed(2)}x
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
