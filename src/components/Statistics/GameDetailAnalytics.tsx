import { useQuery } from '@tanstack/react-query';
import { DualAxisChart, PieChart, BubbleChart } from '../common/charts';
import { DataAvailabilityBanner } from './DataAvailabilityBanner';
import { 
  getBroadcasterAnalytics, 
  getGameDailyStats, 
  getDataAvailability 
} from '../../api/statistics';

interface GameDetailAnalyticsProps {
  category: string;
  startTime?: string;
  endTime?: string;
  onBackClick?: () => void;
  onChannelClick?: (channelId: number) => void;
}

export default function GameDetailAnalytics({
  category,
  startTime,
  endTime,
  onBackClick,
  onChannelClick,
}: GameDetailAnalyticsProps) {
  // データ可用性情報を取得
  const { data: availability } = useQuery({
    queryKey: ['data-availability'],
    queryFn: getDataAvailability,
  });

  // 日次統計を取得
  const { data: dailyStats } = useQuery({
    queryKey: ['game-daily-stats', category, startTime, endTime],
    queryFn: () => getGameDailyStats({
      category,
      startTime: startTime || '',
      endTime: endTime || '',
    }),
    enabled: !!startTime && !!endTime,
  });

  // チャンネル別統計を取得
  const { data: channelAnalytics, isLoading, error } = useQuery({
    queryKey: ['game-channel-analytics', category, startTime, endTime],
    queryFn: async () => {
      // カテゴリでフィルタした配信者統計を取得
      const result = await getBroadcasterAnalytics({
        channelId: undefined,
        startTime,
        endTime,
      });
      // 指定カテゴリをメインにプレイしているチャンネルのみフィルタ
      return result.filter(ch => ch.main_played_title === category);
    },
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <div className="text-gray-400">Loading game detail...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-500/10 border border-red-500 rounded-lg">
        <p className="text-red-500">Error loading game detail: {String(error)}</p>
      </div>
    );
  }

  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat().format(Math.round(num));
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('ja-JP', { month: '2-digit', day: '2-digit' });
  };

  // 日次チャート用データ
  const dailyChartData = (dailyStats || []).map(stat => ({
    date: formatDate(stat.date),
    minutes_watched: stat.minutes_watched,
    hours_broadcasted: stat.hours_broadcasted,
  }));

  // チャンネル別円グラフ用データ
  const channelPieData = (channelAnalytics || []).slice(0, 10).map(ch => ({
    name: ch.channel_name,
    value: ch.minutes_watched,
  }));

  // チャンネル別バブルチャート用データ
  const channelBubbleData = (channelAnalytics || []).slice(0, 30).map(ch => ({
    name: ch.channel_name,
    hours_broadcasted: ch.hours_broadcasted,
    average_ccu: ch.average_ccu,
    minutes_watched: ch.minutes_watched,
  }));

  const totalMW = (channelAnalytics || []).reduce((sum, ch) => sum + ch.minutes_watched, 0);

  return (
    <div className="space-y-6">
      {/* パンくずリスト */}
      <div className="flex items-center gap-2 text-sm">
        <button
          onClick={onBackClick}
          className="text-blue-600 dark:text-blue-400 hover:underline"
        >
          ← Top Games
        </button>
        <span className="text-gray-400">/</span>
        <span className="text-gray-900 dark:text-gray-100 font-semibold">{category}</span>
      </div>

      {availability && <DataAvailabilityBanner availability={availability} />}

      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total MW</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(totalMW)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Unique Broadcasters</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {channelAnalytics?.length || 0}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Top Channel</p>
          <p className="text-lg font-bold text-gray-900 dark:text-gray-100 truncate">
            {channelAnalytics?.[0]?.channel_name || '-'}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Exp Ad Value</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(totalMW * 0.1333)}
          </p>
        </div>
      </div>

      {/* 時系列チャート */}
      {dailyChartData.length > 0 && (
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <DualAxisChart
            data={dailyChartData}
            primaryDataKey="minutes_watched"
            secondaryDataKey="hours_broadcasted"
            primaryType="bar"
            secondaryType="line"
            xAxisKey="date"
            primaryColor="#3b82f6"
            secondaryColor="#10b981"
            primaryYAxisLabel="Minutes Watched"
            secondaryYAxisLabel="Hours Broadcasted"
            title="Daily Trend"
            height={300}
          />
        </div>
      )}

      {/* チャンネル別分析 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <PieChart
            data={channelPieData}
            dataKey="value"
            nameKey="name"
            title="MW Percentile of Channels (Top 10)"
            height={300}
            showLegend={true}
          />
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <BubbleChart
            data={channelBubbleData}
            xDataKey="hours_broadcasted"
            yDataKey="average_ccu"
            zDataKey="minutes_watched"
            nameKey="name"
            title="Top 30 Channel Bubble Chart"
            xAxisLabel="Hours Broadcasted"
            yAxisLabel="Average CCU"
            height={400}
          />
        </div>
      </div>

      {/* チャンネルテーブル */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Channels Playing {category}</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Rank
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Channel
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Minutes Watched
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Hours Broadcasted
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Avg CCU
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  MW%
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {(channelAnalytics || []).map((channel, index) => {
                const mwPercent = totalMW > 0 ? ((channel.minutes_watched / totalMW) * 100).toFixed(2) : '0.00';
                return (
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
                      {(channel.hours_broadcasted || 0).toFixed(1)}
                    </td>
                    <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                      {formatNumber(channel.average_ccu)}
                    </td>
                    <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                      {mwPercent}%
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
