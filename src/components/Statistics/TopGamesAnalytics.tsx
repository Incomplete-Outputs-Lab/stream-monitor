import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { GameAnalytics, DataAvailability } from '../../types';
import { HorizontalBarChart, PieChart, BubbleChart } from '../common/charts';
import { DataAvailabilityBanner } from './DataAvailabilityBanner';

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
    queryFn: async () => {
      return await invoke<DataAvailability>('get_data_availability');
    },
  });

  // トップゲーム統計を取得（全ゲーム）
  const { data: gameAnalytics, isLoading, error } = useQuery({
    queryKey: ['top-games-analytics', startTime, endTime],
    queryFn: async () => {
      const result = await invoke<GameAnalytics[]>('get_game_analytics', {
        category: undefined,
        startTime,
        endTime,
      });
      return result;
    },
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <div className="text-gray-400">Loading top games analytics...</div>
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

  // Top 30 for ranking
  const top30Games = gameAnalytics.slice(0, 30);

  // Top 10 for pie chart
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
                  Unique Broadcasters
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  Exp Ad Value
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
