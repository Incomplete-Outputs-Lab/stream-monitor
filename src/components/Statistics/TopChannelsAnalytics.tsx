import { useQuery } from '@tanstack/react-query';
import { HorizontalBarChart } from '../common/charts';
import { DataAvailabilityBanner } from './DataAvailabilityBanner';
import { StatCardSkeleton, ChartSkeleton, TableSkeleton } from '../common/Skeleton';
import { useSortableData } from '../../hooks/useSortableData';
import { getBroadcasterAnalytics, getDataAvailability } from '../../api/statistics';

interface TopChannelsAnalyticsProps {
  startTime?: string;
  endTime?: string;
  onChannelClick?: (channelId: number) => void;
}

export default function TopChannelsAnalytics({
  startTime,
  endTime,
  onChannelClick,
}: TopChannelsAnalyticsProps) {
  // データ可用性情報を取得
  const { data: availability } = useQuery({
    queryKey: ['data-availability'],
    queryFn: getDataAvailability,
  });

  // トップチャンネル統計を取得
  const { data: channelAnalytics, isLoading, error } = useQuery({
    queryKey: ['top-channels-analytics', startTime, endTime],
    queryFn: () => getBroadcasterAnalytics({
      channelId: undefined,
      startTime,
      endTime,
    }),
  });

  // ソート機能を追加（hooksは常に同じ順序で呼ぶ必要がある）
  const { sortedItems, sortConfig, requestSort } = useSortableData(channelAnalytics || [], {
    key: 'minutes_watched',
    direction: 'desc',
  });

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <StatCardSkeleton key={i} />
          ))}
        </div>
        <ChartSkeleton height={600} />
        <TableSkeleton rows={30} columns={9} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-500/10 border border-red-500 rounded-lg">
        <p className="text-red-500">Error loading analytics: {String(error)}</p>
      </div>
    );
  }

  if (!channelAnalytics || channelAnalytics.length === 0) {
    return (
      <div className="p-8 text-center text-gray-400">
        No channel data available for the selected period.
      </div>
    );
  }

  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat().format(Math.round(num));
  };

  const formatHours = (hours: number): string => {
    return (hours || 0).toFixed(1);
  };

  const formatDecimal = (num: number): string => {
    return (num || 0).toFixed(2);
  };

  // ランキングチャート用データ (元のソート順を使用)
  const rankingData = channelAnalytics.slice(0, 30).map(channel => ({
    name: channel.channel_name,
    minutes_watched: channel.minutes_watched,
  }));

  // 推定広告価値を計算
  const calculateAdValue = (mw: number): number => {
    return Math.round(mw * 0.1333);
  };

  // ソート表示用のヘルパー
  const getSortIndicator = (key: string) => {
    if (sortConfig.key !== key) return null;
    return sortConfig.direction === 'asc' ? ' ↑' : ' ↓';
  };

  return (
    <div className="space-y-6">
      {availability && <DataAvailabilityBanner availability={availability} />}

      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Channels</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {channelAnalytics.length}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total MW</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelAnalytics.reduce((sum, ch) => sum + ch.minutes_watched, 0))}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Streams</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelAnalytics.reduce((sum, ch) => sum + ch.stream_count, 0))}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Chat Messages</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelAnalytics.reduce((sum, ch) => sum + ch.total_chat_messages, 0))}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Avg Chat Rate</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatDecimal(channelAnalytics.reduce((sum, ch) => sum + ch.avg_chat_rate, 0) / channelAnalytics.length)}
          </p>
        </div>
      </div>

      {/* ランキングチャート */}
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
        <HorizontalBarChart
          data={rankingData}
          dataKey="minutes_watched"
          nameKey="name"
          title="Streamer Ranking MW (Top 30)"
          color="#10b981"
          maxItems={30}
          height={800}
          xAxisLabel="Minutes Watched"
        />
      </div>

      {/* テーブル */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Channel Ranking</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Rank
                </th>
                <th
                  className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('channel_name')}
                >
                  Channel{getSortIndicator('channel_name')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('minutes_watched')}
                >
                  Minutes Watched{getSortIndicator('minutes_watched')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('hours_broadcasted')}
                >
                  Hours Broadcasted{getSortIndicator('hours_broadcasted')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('average_ccu')}
                >
                  Avg CCU{getSortIndicator('average_ccu')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('avg_chat_rate')}
                >
                  Avg Chat Rate{getSortIndicator('avg_chat_rate')}
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Main Game
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('main_title_mw_percent')}
                >
                  Main Game MW%{getSortIndicator('main_title_mw_percent')}
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Exp Ad Value
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {sortedItems.map((channel, index) => (
                <tr
                  key={channel.channel_id}
                  className="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                  onClick={() => onChannelClick?.(channel.channel_id)}
                >
                  <td className="px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                    {index + 1}
                  </td>
                  <td className="px-4 py-3 text-sm text-blue-600 dark:text-blue-400 font-medium">
                    {channel.channel_name}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatNumber(channel.minutes_watched)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatHours(channel.hours_broadcasted)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatNumber(channel.average_ccu)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatDecimal(channel.avg_chat_rate)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                    {channel.main_played_title || '-'}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {channel.main_title_mw_percent?.toFixed(2) || '0.00'}%
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatNumber(calculateAdValue(channel.minutes_watched))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
