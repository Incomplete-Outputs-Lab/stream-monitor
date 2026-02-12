import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { openUrl } from '@tauri-apps/plugin-opener';
import { BarChart } from '../common/charts/BarChart';
import { Tooltip } from '../common/Tooltip';
import { useSortableData } from '../../hooks/useSortableData';
import { SortableTableHeader } from '../common/SortableTableHeader';
import { getBroadcasterAnalytics } from '../../api/statistics';
import { useChannelStore } from '../../stores/channelStore';

interface BroadcasterAnalyticsProps {
  startTime?: string;
  endTime?: string;
}

export default function BroadcasterAnalytics({
  startTime,
  endTime,
}: BroadcasterAnalyticsProps) {
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  const { channels, fetchChannels } = useChannelStore();

  // チャンネル一覧取得
  useEffect(() => {
    fetchChannels();
  }, [fetchChannels]);
  const { data: analytics, isLoading, error } = useQuery({
    queryKey: ['broadcaster-analytics', selectedChannelId, startTime, endTime],
    queryFn: () => getBroadcasterAnalytics({
      channelId: selectedChannelId || undefined,
      startTime,
      endTime,
    }),
  });

  // ソート機能
  const { sortedItems, sortConfig, requestSort } = useSortableData(
    analytics || [],
    { key: 'minutes_watched', direction: 'desc' }
  );

  // チャンネルページを開く
  const handleOpenChannel = async (loginName: string) => {
    try {
      await openUrl(`https://www.twitch.tv/${loginName}`);
    } catch (err) {
      console.error('Failed to open channel:', err);
    }
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        {/* チャンネルフィルター */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル
          </label>
          <select
            value={selectedChannelId || ''}
            onChange={(e) => setSelectedChannelId(e.target.value ? parseInt(e.target.value) : null)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
          >
            <option value="">すべてのチャンネル</option>
            {channels?.map((channel) => (
              <option key={channel.id} value={channel.id}>
                {channel.display_name || channel.channel_name} ({channel.platform})
              </option>
            ))}
          </select>
        </div>
        <div className="flex justify-center items-center p-8">
          <div className="text-gray-400">Loading broadcaster analytics...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-4">
        {/* チャンネルフィルター */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル
          </label>
          <select
            value={selectedChannelId || ''}
            onChange={(e) => setSelectedChannelId(e.target.value ? parseInt(e.target.value) : null)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
          >
            <option value="">すべてのチャンネル</option>
            {channels?.map((channel) => (
              <option key={channel.id} value={channel.id}>
                {channel.display_name || channel.channel_name} ({channel.platform})
              </option>
            ))}
          </select>
        </div>
        <div className="p-4 bg-red-500/10 border border-red-500 rounded-lg">
          <p className="text-red-500">Error loading analytics: {String(error)}</p>
        </div>
      </div>
    );
  }

  if (!analytics || analytics.length === 0) {
    return (
      <div className="space-y-4">
        {/* チャンネルフィルター */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル
          </label>
          <select
            value={selectedChannelId || ''}
            onChange={(e) => setSelectedChannelId(e.target.value ? parseInt(e.target.value) : null)}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
          >
            <option value="">すべてのチャンネル</option>
            {channels?.map((channel) => (
              <option key={channel.id} value={channel.id}>
                {channel.display_name || channel.channel_name} ({channel.platform})
              </option>
            ))}
          </select>
        </div>
        <div className="p-8 text-center text-gray-400">
          No analytics data available for the selected period.
        </div>
      </div>
    );
  }

  // 数値フォーマット用のヘルパー関数
  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat().format(Math.round(num));
  };

  const formatHours = (hours: number): string => {
    return (hours || 0).toFixed(1);
  };

  const formatPercent = (percent: number | null): string => {
    if (percent === null) return 'N/A';
    return `${(percent || 0).toFixed(1)}%`;
  };

  const formatDecimal = (num: number): string => {
    return (num || 0).toFixed(2);
  };

  // MinutesWatched チャート用データ
  const mwChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: item.minutes_watched,
  }));

  // Average CCU チャート用データ
  const ccuChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: Math.round(item.average_ccu),
  }));

  // Hours Broadcasted チャート用データ
  const hoursChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: parseFloat((item.hours_broadcasted || 0).toFixed(1)),
  }));

  // Peak CCU チャート用データ
  const peakCcuChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: item.peak_ccu,
  }));

  // Peak to Average Ratio チャート用データ
  const peakToAvgRatioData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: item.average_ccu > 0 ? parseFloat((item.peak_ccu / item.average_ccu).toFixed(2)) : 0,
  }));

  // Engagement Rate チャート用データ
  const engagementChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: parseFloat((item.engagement_rate || 0).toFixed(2)),
  }));

  // Average Chat Rate チャート用データ
  const chatRateChartData = sortedItems.map((item) => ({
    name: item.channel_name,
    value: parseFloat((item.avg_chat_rate || 0).toFixed(2)),
  }));

  return (
    <div className="space-y-6">
      {/* チャンネルフィルター */}
      <div className="card p-4">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          チャンネル
        </label>
        <select
          value={selectedChannelId || ''}
          onChange={(e) => setSelectedChannelId(e.target.value ? parseInt(e.target.value) : null)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-slate-800 text-gray-900 dark:text-gray-100"
        >
          <option value="">すべてのチャンネル</option>
          {channels?.map((channel) => (
            <option key={channel.id} value={channel.id}>
              {channel.display_name || channel.channel_name} ({channel.platform})
            </option>
          ))}
        </select>
      </div>

      {/* サマリーカード */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="分析対象の配信者数">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Broadcasters
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">{analytics.length}</div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="視聴者数 × 時間の合計。配信の総視聴時間を表します。">
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
          <Tooltip content="配信を行った合計時間">
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
          <Tooltip content="期間内の総配信回数">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Streams
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatNumber(
              analytics.reduce((sum, item) => sum + item.stream_count, 0)
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="期間内のチャットメッセージ総数（概算値）">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Total Chat Messages
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatNumber(
              analytics.reduce((sum, item) => sum + item.total_chat_messages, 0)
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="エンゲージメント率の平均値。1000分視聴あたりのチャット数で、視聴者の参加度を表します。">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Avg Engagement
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatDecimal(
              analytics.reduce((sum, item) => sum + item.engagement_rate, 0) / analytics.length
            )}
          </div>
        </div>
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <Tooltip content="1分あたりの平均チャット数">
            <div className="text-gray-400 text-sm mb-1 flex items-center gap-1">
              Avg Chat Rate
              <span className="text-xs opacity-60">ℹ️</span>
            </div>
          </Tooltip>
          <div className="text-2xl font-bold text-white">
            {formatDecimal(
              analytics.reduce((sum, item) => sum + item.avg_chat_rate, 0) / analytics.length
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
          tooltipDescription="視聴者数×時間の合計。配信の総視聴時間を配信者別に表示します。"
          color="#10b981"
          height={300}
          yAxisLabel="Minutes"
        />
        <BarChart
          data={ccuChartData}
          dataKey="value"
          xAxisKey="name"
          title="Average CCU by Broadcaster"
          tooltipDescription="Average Concurrent Users（平均同時視聴者数）。期間内の平均視聴者数を示します。"
          color="#3b82f6"
          height={300}
          yAxisLabel="Viewers"
        />
        <BarChart
          data={peakCcuChartData}
          dataKey="value"
          xAxisKey="name"
          title="Peak CCU by Broadcaster"
          tooltipDescription="Peak Concurrent Users（ピーク同時視聴者数）。期間内の最大視聴者数を示します。"
          color="#8b5cf6"
          height={300}
          yAxisLabel="Viewers"
        />
        <BarChart
          data={hoursChartData}
          dataKey="value"
          xAxisKey="name"
          title="Hours Broadcasted by Channel"
          tooltipDescription="配信を行った合計時間。配信の長さや頻度の指標です。"
          color="#f59e0b"
          height={300}
          yAxisLabel="Hours"
        />
        <BarChart
          data={peakToAvgRatioData}
          dataKey="value"
          xAxisKey="name"
          title="Peak to Average Ratio by Broadcaster"
          tooltipDescription="ピーク集中度（Peak CCU / Average CCU）。1に近い=安定した視聴者数、大きい値=ピーク時に視聴者が集中。配信のバイラル性や話題性を示します。"
          color="#ec4899"
          height={300}
          yAxisLabel="Ratio"
        />
        <BarChart
          data={engagementChartData}
          dataKey="value"
          xAxisKey="name"
          title="Engagement Rate by Broadcaster"
          tooltipDescription="エンゲージメント率（チャット数 / Minutes Watched × 1000）。1000分視聴あたりのチャット数で、視聴者の参加度を表します。高いほど活発です。"
          color="#14b8a6"
          height={300}
          yAxisLabel="Messages/1000MW"
        />
        <BarChart
          data={chatRateChartData}
          dataKey="value"
          xAxisKey="name"
          title="Average Chat Rate by Broadcaster"
          tooltipDescription="1分あたりの平均チャット数。配信の活発度を示します。"
          color="#06b6d4"
          height={300}
          yAxisLabel="Messages/min"
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
                <SortableTableHeader
                  sortKey="channel_name"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="left"
                >
                  Broadcaster
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="minutes_watched"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  MW
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="hours_broadcasted"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Hours
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="average_ccu"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Avg CCU
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="peak_ccu"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Peak CCU
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="stream_count"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Streams
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="peak_ccu / average_ccu"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  P/A Ratio
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="total_chat_messages"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Chat Msgs
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="avg_chat_rate"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Avg Chat Rate
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="engagement_rate"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  Engagement
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="main_played_title"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="left"
                >
                  Main Title
                </SortableTableHeader>
                <SortableTableHeader
                  sortKey="main_title_mw_percent"
                  currentSortKey={sortConfig.key as string}
                  currentDirection={sortConfig.direction}
                  onSort={requestSort}
                  align="right"
                >
                  MW%
                </SortableTableHeader>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {sortedItems.map((item) => (
                <tr key={item.channel_id} className="hover:bg-gray-700/30">
                  <td className="px-4 py-3 text-sm text-white font-medium">
                    <button
                      onClick={() => handleOpenChannel(item.login_name)}
                      className="text-blue-400 hover:text-blue-300 hover:underline transition-colors text-left"
                    >
                      {item.channel_name}
                    </button>
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
                    {formatNumber(item.peak_ccu)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {item.stream_count}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {item.average_ccu > 0 ? formatDecimal(item.peak_ccu / item.average_ccu) : 'N/A'}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatNumber(item.total_chat_messages)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatDecimal(item.avg_chat_rate)}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-300 text-right">
                    {formatDecimal(item.engagement_rate)}
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
