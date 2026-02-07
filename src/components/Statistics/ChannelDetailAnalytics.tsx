import { useQuery } from '@tanstack/react-query';
import { DualAxisChart, PieChart, BubbleChart } from '../common/charts';
import { DataAvailabilityBanner } from './DataAvailabilityBanner';
import { 
  getBroadcasterAnalytics, 
  getGameAnalytics, 
  getChannelDailyStats, 
  getDataAvailability 
} from '../../api/statistics';

interface ChannelDetailAnalyticsProps {
  channelId: number;
  channelName: string;
  startTime?: string;
  endTime?: string;
  onBackClick?: () => void;
  onGameClick?: (category: string) => void;
}

export default function ChannelDetailAnalytics({
  channelId,
  channelName,
  startTime,
  endTime,
  onBackClick,
  onGameClick,
}: ChannelDetailAnalyticsProps) {
  // データ可用性情報を取得
  const { data: availability } = useQuery({
    queryKey: ['data-availability'],
    queryFn: getDataAvailability,
  });

  // 日次統計を取得
  const { data: dailyStats } = useQuery({
    queryKey: ['channel-daily-stats', channelId, startTime, endTime],
    queryFn: () => getChannelDailyStats({
      channelId,
      startTime: startTime || '',
      endTime: endTime || '',
    }),
    enabled: !!startTime && !!endTime,
  });

  // チャンネル統計を取得
  const { data: channelStats, isLoading, error } = useQuery({
    queryKey: ['channel-detail-analytics', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await getBroadcasterAnalytics({
        channelId,
        startTime,
        endTime,
      });
      return result[0];
    },
  });

  // ゲーム別統計を取得（このチャンネルがプレイした全ゲーム）
  const { data: gameAnalytics } = useQuery({
    queryKey: ['channel-games-analytics', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await getGameAnalytics({
        category: undefined,
        startTime,
        endTime,
      });
      // このチャンネルがトップチャンネルのゲームのみフィルタ
      return result.filter(g => g.top_channel === channelName);
    },
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <div className="text-gray-400">Loading channel detail...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-500/10 border border-red-500 rounded-lg">
        <p className="text-red-500">Error loading channel detail: {String(error)}</p>
      </div>
    );
  }

  if (!channelStats) {
    return (
      <div className="p-8 text-center text-gray-400">
        No data available for this channel.
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
    average_ccu: stat.average_ccu,
  }));

  // ゲーム別円グラフ用データ
  const gamePieData = (gameAnalytics || []).slice(0, 10).map(game => ({
    name: game.category,
    value: game.minutes_watched,
  }));

  // ゲーム別バブルチャート用データ
  const gameBubbleData = (gameAnalytics || []).map(game => ({
    name: game.category,
    hours_broadcasted: game.hours_broadcasted,
    average_ccu: game.average_ccu,
    minutes_watched: game.minutes_watched,
  }));

  const totalGameMW = (gameAnalytics || []).reduce((sum, g) => sum + g.minutes_watched, 0);

  return (
    <div className="space-y-6">
      {/* パンくずリスト */}
      <div className="flex items-center gap-2 text-sm">
        <button
          onClick={onBackClick}
          className="text-blue-600 dark:text-blue-400 hover:underline"
        >
          ← Top Channels
        </button>
        <span className="text-gray-400">/</span>
        <span className="text-gray-900 dark:text-gray-100 font-semibold">{channelName}</span>
      </div>

      {availability && <DataAvailabilityBanner availability={availability} />}

      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total MW</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelStats.minutes_watched)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Streams</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {channelStats.stream_count}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Peak CCU</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelStats.peak_ccu)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Exp Ad Value</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(channelStats.minutes_watched * 0.1333)}
          </p>
        </div>
      </div>

      {/* 時系列チャート */}
      {dailyChartData.length > 0 && (
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <DualAxisChart
            data={dailyChartData}
            primaryDataKey="average_ccu"
            secondaryDataKey="minutes_watched"
            primaryType="bar"
            secondaryType="line"
            xAxisKey="date"
            primaryColor="#3b82f6"
            secondaryColor="#10b981"
            primaryYAxisLabel="Average CCU"
            secondaryYAxisLabel="Minutes Watched"
            title="Daily Trend"
            height={300}
          />
        </div>
      )}

      {/* ゲーム別分析 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <PieChart
            data={gamePieData}
            dataKey="value"
            nameKey="name"
            title="MW Percentile of Games"
            height={300}
            showLegend={true}
          />
        </div>

        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <BubbleChart
            data={gameBubbleData}
            xDataKey="hours_broadcasted"
            yDataKey="average_ccu"
            zDataKey="minutes_watched"
            nameKey="name"
            title="Played Games Bubble Chart"
            xAxisLabel="Hours Broadcasted"
            yAxisLabel="Average CCU"
            height={400}
          />
        </div>
      </div>

      {/* ゲームテーブル */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Games Played by {channelName}
          </h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Rank
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Game Title
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
              {(gameAnalytics || []).map((game, index) => {
                const mwPercent = totalGameMW > 0 ? ((game.minutes_watched / totalGameMW) * 100).toFixed(2) : '0.00';
                return (
                  <tr
                    key={game.category}
                    className="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                    onClick={() => onGameClick?.(game.category)}
                  >
                    <td className="px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                      {index + 1}
                    </td>
                    <td className="px-4 py-3 text-sm text-blue-600 dark:text-blue-400 font-medium">
                      {game.category}
                    </td>
                    <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                      {formatNumber(game.minutes_watched)}
                    </td>
                    <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                      {(game.hours_broadcasted || 0).toFixed(1)}
                    </td>
                    <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                      {formatNumber(game.average_ccu)}
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

      {/* チャンネル詳細統計 */}
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Channel Statistics
        </h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <p className="text-gray-500 dark:text-gray-400">Hours Broadcasted</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {(channelStats.hours_broadcasted || 0).toFixed(1)}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Average CCU</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {formatNumber(channelStats.average_ccu)}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Total Chat Messages</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {formatNumber(channelStats.total_chat_messages)}
            </p>
          </div>
          <div>
            <p className="text-gray-500 dark:text-gray-400">Engagement Rate</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {(channelStats.engagement_rate || 0).toFixed(2)}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
