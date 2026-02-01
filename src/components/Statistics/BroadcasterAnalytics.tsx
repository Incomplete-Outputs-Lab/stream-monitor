import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { BroadcasterAnalytics as BroadcasterAnalyticsType } from '../../types';
import { BarChart } from '../common/charts/BarChart';

interface BroadcasterAnalyticsProps {
  channelId?: number;
  startTime?: string;
  endTime?: string;
}

export default function BroadcasterAnalytics({
  channelId,
  startTime,
  endTime,
}: BroadcasterAnalyticsProps) {
  const { data: analytics, isLoading, error } = useQuery({
    queryKey: ['broadcaster-analytics', channelId, startTime, endTime],
    queryFn: async () => {
      const result = await invoke<BroadcasterAnalyticsType[]>(
        'get_broadcaster_analytics',
        {
          channelId,
          startTime,
          endTime,
        }
      );
      return result;
    },
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <div className="text-gray-400">Loading broadcaster analytics...</div>
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

  const formatPercent = (percent: number | null): string => {
    if (percent === null) return 'N/A';
    return `${percent.toFixed(1)}%`;
  };

  // MinutesWatched チャート用データ
  const mwChartData = analytics.map((item) => ({
    name: item.channel_name,
    value: item.minutes_watched,
  }));

  // Average CCU チャート用データ
  const ccuChartData = analytics.map((item) => ({
    name: item.channel_name,
    value: Math.round(item.average_ccu),
  }));

  // Hours Broadcasted チャート用データ
  const hoursChartData = analytics.map((item) => ({
    name: item.channel_name,
    value: parseFloat(item.hours_broadcasted.toFixed(1)),
  }));

  return (
    <div className="space-y-6">
      {/* サマリーカード */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <div className="text-gray-400 text-sm mb-1">Total Broadcasters</div>
          <div className="text-2xl font-bold text-white">{analytics.length}</div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <div className="text-gray-400 text-sm mb-1">Total Minutes Watched</div>
          <div className="text-2xl font-bold text-white">
            {formatNumber(
              analytics.reduce((sum, item) => sum + item.minutes_watched, 0)
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <div className="text-gray-400 text-sm mb-1">Total Hours Broadcasted</div>
          <div className="text-2xl font-bold text-white">
            {formatHours(
              analytics.reduce((sum, item) => sum + item.hours_broadcasted, 0)
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
          title="Minutes Watched by Broadcaster"
          color="#10b981"
          height={300}
          yAxisLabel="Minutes"
        />
        <BarChart
          data={ccuChartData}
          dataKey="value"
          xAxisKey="name"
          title="Average CCU by Broadcaster"
          color="#3b82f6"
          height={300}
          yAxisLabel="Viewers"
        />
      </div>

      <BarChart
        data={hoursChartData}
        dataKey="value"
        xAxisKey="name"
        title="Hours Broadcasted by Channel"
        color="#f59e0b"
        height={300}
        yAxisLabel="Hours"
      />

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
                  Broadcaster
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Minutes Watched
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Hours Broadcasted
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Avg CCU
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Main Title
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">
                  MW% of Main
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {analytics.map((item) => (
                <tr key={item.channel_id} className="hover:bg-gray-700/30">
                  <td className="px-4 py-3 text-sm text-white font-medium">
                    {item.channel_name}
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
                  <td className="px-4 py-3 text-sm text-gray-300">
                    {item.main_played_title || 'N/A'}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatPercent(item.main_title_mw_percent)}
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
