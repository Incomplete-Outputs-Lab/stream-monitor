import { useQuery } from '@tanstack/react-query';
import { HorizontalBarChart, PieChart, BubbleChart } from '../common/charts';
import { DataAvailabilityBanner } from './DataAvailabilityBanner';
import { LoadingSpinner } from '../common/LoadingSpinner';
import { useSortableData } from '../../hooks/useSortableData';
import { getGameAnalytics, getDataAvailability } from '../../api/statistics';

interface TopGamesAnalyticsProps {
  startTime?: string;
  endTime?: string;
  onGameClick?: (category: string) => void;
}

export default function TopGamesAnalytics({
  startTime,
  endTime,
  onGameClick,
}: TopGamesAnalyticsProps) {
  // データ可用性情報を取得
  const { data: availability } = useQuery({
    queryKey: ['data-availability'],
    queryFn: getDataAvailability,
  });

  // トップゲーム統計を取得（全ゲーム）
  const { data: gameAnalytics, isLoading, error } = useQuery({
    queryKey: ['top-games-analytics', startTime, endTime],
    queryFn: () => getGameAnalytics({
      category: undefined,
      startTime,
      endTime,
    }),
  });

  // ソート機能を追加（hooksは常に同じ順序で呼ぶ必要がある）
  const { sortedItems, sortConfig, requestSort } = useSortableData(gameAnalytics || [], {
    key: 'minutes_watched',
    direction: 'desc',
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <LoadingSpinner size="lg" message="トップゲーム統計を読み込み中..." />
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

  if (!gameAnalytics || gameAnalytics.length === 0) {
    return (
      <div className="p-8 text-center text-gray-400">
        No game data available for the selected period.
      </div>
    );
  }

  // 数値フォーマット
  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat().format(Math.round(num));
  };

  const formatHours = (hours: number): string => {
    return hours.toFixed(1);
  };

  const formatDecimal = (num: number): string => {
    return num.toFixed(2);
  };

  // Top 30 for ranking
  const top30Games = sortedItems.slice(0, 30);

  // Top 10 for pie chart (元のソート順を使用)
  const top10Games = gameAnalytics.slice(0, 10);
  const totalMW = gameAnalytics.reduce((sum, game) => sum + game.minutes_watched, 0);

  // ランキングチャート用データ
  const rankingData = top30Games.map(game => ({
    name: game.category,
    minutes_watched: game.minutes_watched,
  }));

  // 円グラフ用データ
  const pieData = top10Games.map(game => ({
    name: game.category,
    value: game.minutes_watched,
  }));

  // ソート表示用のヘルパー
  const getSortIndicator = (key: string) => {
    if (sortConfig.key !== key) return null;
    return sortConfig.direction === 'asc' ? ' ↑' : ' ↓';
  };

  // バブルチャート用データ
  const bubbleData = top30Games.map(game => ({
    name: game.category,
    hours_broadcasted: game.hours_broadcasted,
    average_ccu: game.average_ccu,
    minutes_watched: game.minutes_watched,
  }));

  // 推定広告価値を計算
  const calculateAdValue = (mw: number): number => {
    return Math.round(mw * 0.1333);
  };

  return (
    <div className="space-y-6">
      {availability && <DataAvailabilityBanner availability={availability} />}

      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total Games</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {gameAnalytics.length}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total MW</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(totalMW)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Total HB</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(gameAnalytics.reduce((sum, g) => sum + g.hours_broadcasted, 0))}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Avg CCU</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatNumber(gameAnalytics.reduce((sum, g) => sum + g.average_ccu, 0) / gameAnalytics.length)}
          </p>
        </div>
        <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <p className="text-sm text-gray-500 dark:text-gray-400">Avg Chat Rate</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {formatDecimal(gameAnalytics.reduce((sum, g) => sum + g.avg_chat_rate, 0) / gameAnalytics.length)}
          </p>
        </div>
      </div>

      {/* チャート */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
          <HorizontalBarChart
            data={rankingData}
            dataKey="minutes_watched"
            nameKey="name"
            title="Game Ranking (Top 30)"
            color="#3b82f6"
            maxItems={30}
            height={800}
            xAxisLabel="Minutes Watched"
          />
        </div>

        <div className="space-y-6">
          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <PieChart
              data={pieData}
              dataKey="value"
              nameKey="name"
              title="MW Percentile of Games (Top 10)"
              height={300}
              showLegend={true}
              showPercentage={true}
            />
          </div>

          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
            <BubbleChart
              data={bubbleData}
              xDataKey="hours_broadcasted"
              yDataKey="average_ccu"
              zDataKey="minutes_watched"
              nameKey="name"
              title="Top Games Bubble Chart"
              xAxisLabel="Hours Broadcasted"
              yAxisLabel="Average CCU"
              height={400}
            />
          </div>
        </div>
      </div>

      {/* テーブル */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Game Ranking</h3>
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
                  onClick={() => requestSort('category')}
                >
                  Game Title{getSortIndicator('category')}
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
                  onClick={() => requestSort('average_chat_rate')}
                >
                  Avg Chat Rate{getSortIndicator('average_chat_rate')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('unique_broadcasters')}
                >
                  Unique Broadcasters{getSortIndicator('unique_broadcasters')}
                </th>
                <th
                  className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:text-gray-700 dark:hover:text-gray-300"
                  onClick={() => requestSort('expected_ad_value')}
                >
                  Exp Ad Value{getSortIndicator('expected_ad_value')}
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Top Channel
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {gameAnalytics.map((game, index) => (
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
                    {formatHours(game.hours_broadcasted)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatNumber(game.average_ccu)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatDecimal(game.avg_chat_rate)}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {game.unique_broadcasters}
                  </td>
                  <td className="px-4 py-3 text-sm text-right text-gray-900 dark:text-gray-100">
                    {formatNumber(calculateAdValue(game.minutes_watched))}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                    {game.top_channel || '-'}
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
