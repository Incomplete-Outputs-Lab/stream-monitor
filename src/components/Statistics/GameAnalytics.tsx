import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { GameAnalytics as GameAnalyticsType } from '../../types';
import { BarChart } from '../common/charts/BarChart';
import { Tooltip } from '../common/Tooltip';

interface GameAnalyticsProps {
  startTime?: string;
  endTime?: string;
}

export default function GameAnalytics({
  startTime,
  endTime,
}: GameAnalyticsProps) {
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [searchTerm, setSearchTerm] = useState<string>('');

  // カテゴリ一覧を取得
  const { data: categories } = useQuery({
    queryKey: ['game-categories', startTime, endTime],
    queryFn: async () => {
      const result = await invoke<string[]>('list_game_categories', {
        startTime,
        endTime,
      });
      return result;
    },
  });
  // ゲーム分析データを取得
  const { data: analytics, isLoading, error } = useQuery({
    queryKey: ['game-analytics', selectedCategory, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<GameAnalyticsType[]>('get_game_analytics', {
        category: selectedCategory || undefined,
        startTime,
        endTime,
      });
      return result;
    },
  });

  // 検索フィルター
  const filteredCategories = categories?.filter((cat) =>
    cat.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <div className="text-gray-400">Loading game analytics...</div>
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

  if (!analytics || analytics.length === 0) {
    return (
      <div className="p-8 text-center text-gray-400">
        No analytics data available for the selected period.
      </div>
    );
  }

  // 数値フォーマット用のヘルパー関数
  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat().format(Math.round(num));
  };

  const formatHours = (hours: number): string => {
    return hours.toFixed(1);
  };

  // MinutesWatched チャート用データ（上位10件）
  const mwChartData = analytics
    .slice(0, 10)
    .map((item) => ({
      name: item.category,
      value: item.minutes_watched,
    }));

  // Average CCU チャート用データ（上位10件）
  const ccuChartData = analytics
    .slice(0, 10)
    .map((item) => ({
      name: item.category,
      value: Math.round(item.average_ccu),
    }));

  // Hours Broadcasted チャート用データ（上位10件）
  const hoursChartData = analytics
    .slice(0, 10)
    .map((item) => ({
      name: item.category,
      value: parseFloat(item.hours_broadcasted.toFixed(1)),
    }));

  // Unique Broadcasters チャート用データ（上位10件）
  const broadcastersChartData = analytics
    .slice(0, 10)
    .map((item) => ({
      name: item.category,
      value: item.unique_broadcasters,
    }));

  return (
    <div className="space-y-6">
      {/* ゲームフィルター */}
      <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          フィルター: ゲーム / カテゴリ
        </label>
        <div className="relative">
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder="ゲーム名で検索..."
            className="w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded-md text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          {searchTerm && (
            <button
              onClick={() => {
                setSearchTerm('');
                setSelectedCategory('');
              }}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
            >
              ✕
            </button>
          )}
        </div>
        <div className="mt-2">
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">すべてのゲーム</option>
            {filteredCategories.map((cat) => (
              <option key={cat} value={cat}>
                {cat}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="分析対象のゲーム/カテゴリ数">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Games
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">{analytics.length}</div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="このゲーム/カテゴリでの総視聴時間（視聴者数×時間）">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Minutes Watched
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatNumber(
              analytics.reduce((sum, item) => sum + item.minutes_watched, 0)
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="このゲーム/カテゴリで配信された合計時間">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Hours Broadcasted
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatHours(
              analytics.reduce((sum, item) => sum + item.hours_broadcasted, 0)
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="このゲーム/カテゴリを配信したユニークな配信者の合計数">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Unique Broadcasters
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatNumber(
              analytics.reduce((sum, item) => sum + item.unique_broadcasters, 0)
            )}
          </div>
        </div>
      </div>

      {/* チャート */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <BarChart
          data={mwChartData}
          dataKey="value"
          xAxisKey="name"
          title="Top 10 Games by Minutes Watched"
          tooltipDescription="総視聴時間が最も多いゲームTop 10。人気度や注目度の指標です。"
          color="#10b981"
          height={300}
          yAxisLabel="Minutes"
        />
        <BarChart
          data={ccuChartData}
          dataKey="value"
          xAxisKey="name"
          title="Top 10 Games by Average CCU"
          tooltipDescription="平均同時視聴者数が最も多いゲームTop 10。視聴の集中度を示します。"
          color="#3b82f6"
          height={300}
          yAxisLabel="Viewers"
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <BarChart
          data={hoursChartData}
          dataKey="value"
          xAxisKey="name"
          title="Top 10 Games by Hours Broadcasted"
          tooltipDescription="配信時間が最も長いゲームTop 10。配信者にとっての人気度を示します。"
          color="#f59e0b"
          height={300}
          yAxisLabel="Hours"
        />
        <BarChart
          data={broadcastersChartData}
          dataKey="value"
          xAxisKey="name"
          title="Top 10 Games by Unique Broadcasters"
          tooltipDescription="配信者数が最も多いゲームTop 10。ゲームの普及度や配信のしやすさを示します。"
          color="#8b5cf6"
          height={300}
          yAxisLabel="Broadcasters"
        />
      </div>

      {/* 詳細テーブル */}
      <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
        <div className="px-4 py-3 border-b border-gray-700">
          <h3 className="text-lg font-semibold text-white">Detailed Statistics</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-900/50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Game / Category
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  <Tooltip content="総視聴時間（視聴者数×時間）">
                    <span className="cursor-help">Minutes Watched</span>
                  </Tooltip>
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  <Tooltip content="配信時間（時間）">
                    <span className="cursor-help">Hours Broadcasted</span>
                  </Tooltip>
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  <Tooltip content="平均同時視聴者数">
                    <span className="cursor-help">Avg CCU</span>
                  </Tooltip>
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  <Tooltip content="このゲームを配信したユニークな配信者数">
                    <span className="cursor-help">Unique Broadcasters</span>
                  </Tooltip>
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  <Tooltip content="このゲームで最も視聴されたチャンネル">
                    <span className="cursor-help">Top Channel</span>
                  </Tooltip>
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {analytics.map((item, index) => (
                <tr key={`${item.category}-${index}`} className="hover:bg-gray-700/30">
                  <td className="px-4 py-3 text-sm text-white font-medium">
                    {item.category}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatNumber(item.minutes_watched)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatHours(item.hours_broadcasted)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatNumber(item.average_ccu)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {item.unique_broadcasters}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300">
                    {item.top_channel || 'N/A'}
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
