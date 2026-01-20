import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState, useMemo } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from "recharts";
import { Channel, StreamStats } from "../../types";
import { DateRangePicker } from "./DateRangePicker";
import { ChannelStatistics } from "./ChannelStatistics";
import { StreamSessionView } from "./StreamSessionView";
import { ChatAnalysis } from "./ChatAnalysis";

type ViewMode = 'overview' | 'channel' | 'session' | 'chat';

interface DateRange {
  start: string;
  end: string;
}

export function Statistics() {
  const [viewMode, setViewMode] = useState<ViewMode>('overview');
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  const [dateRange, setDateRange] = useState<DateRange>({
    start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], // 7日前
    end: new Date().toISOString().split('T')[0], // 今日
  });

  // チャンネル一覧取得
  const { data: channels } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      return await invoke<Channel[]>("list_channels");
    },
  });

  // 統計データ取得
  const { data: stats, isLoading } = useQuery({
    queryKey: ["stats", dateRange, selectedChannelId],
    queryFn: async () => {
      return await invoke<StreamStats[]>("get_stream_stats", {
        query: {
          start_time: new Date(dateRange.start).toISOString(),
          end_time: new Date(dateRange.end + 'T23:59:59').toISOString(),
          channel_id: selectedChannelId || undefined,
        },
      });
    },
  });

  // チャート用データ変換（メモ化）
  const chartData = useMemo(() => {
    return stats?.map(stat => {
      const time = new Date(stat.collected_at).toLocaleString('ja-JP', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
      return {
        time,
        viewers: stat.viewer_count || 0,
        chatRate: stat.chat_rate_1min,
      };
    }) || [];
  }, [stats]);

  // チャンネル別統計データ（メモ化）
  const channelStats = useMemo(() => {
    return channels?.map(channel => {
      const channelStatsData = stats?.filter(_stat => {
        // 実際のデータ構造に基づいてフィルタリング（stream_idからchannel_idを取得する必要がある）
        // 現時点では簡易的な実装
        return true;
      }) || [];

      const avgViewers = channelStatsData.length > 0
        ? Math.round(channelStatsData.reduce((sum, stat) => sum + (stat.viewer_count || 0), 0) / channelStatsData.length)
        : 0;

      const maxViewers = Math.max(...channelStatsData.map(stat => stat.viewer_count || 0), 0);
      const totalChatMessages = channelStatsData.reduce((sum, stat) => sum + stat.chat_rate_1min, 0);

      return {
        channel,
        stats: {
          avgViewers,
          maxViewers,
          totalChatMessages,
          dataPoints: channelStatsData.length,
        }
      };
    }) || [];
  }, [channels, stats]);

  const handleDateRangeChange = (start: string, end: string) => {
    setDateRange({ start, end });
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">統計閲覧</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">過去の統計データを閲覧・分析</p>
        </div>
        <DateRangePicker
          startDate={dateRange.start}
          endDate={dateRange.end}
          onChange={handleDateRangeChange}
        />
      </div>

      {/* ビューモード切り替え */}
      <div className="flex space-x-2">
        <button
          onClick={() => setViewMode('overview')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            viewMode === 'overview'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          概要
        </button>
        <button
          onClick={() => setViewMode('channel')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            viewMode === 'channel'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          チャンネル別
        </button>
        <button
          onClick={() => setViewMode('session')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            viewMode === 'session'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          セッション別
        </button>
        <button
          onClick={() => setViewMode('chat')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            viewMode === 'chat'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          チャット分析
        </button>
      </div>

      {/* チャンネル選択（チャンネル別ビュー時） */}
      {(viewMode === 'channel' || viewMode === 'session') && (
        <div className="card p-4">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル選択
          </label>
          <select
            value={selectedChannelId || ''}
            onChange={(e) => setSelectedChannelId(e.target.value ? parseInt(e.target.value) : null)}
            className="input-field w-full max-w-xs"
          >
            <option value="">すべてのチャンネル</option>
            {channels?.map((channel) => (
              <option key={channel.id} value={channel.id}>
                {channel.display_name || channel.channel_name} ({channel.platform})
              </option>
            ))}
          </select>
        </div>
      )}

      {/* コンテンツ表示 */}
      {viewMode === 'overview' && (
        <div className="space-y-6">
          {/* 視聴者数推移グラフ */}
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">視聴者数推移</h3>
            {chartData.length > 0 ? (
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="time" />
                    <YAxis />
                    <Tooltip />
                    <Line
                      type="monotone"
                      dataKey="viewers"
                      stroke="#3b82f6"
                      strokeWidth={2}
                      dot={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            ) : (
              <div className="h-80 flex items-center justify-center text-gray-500 dark:text-gray-400">
                データがありません
              </div>
            )}
          </div>

          {/* チャット速度グラフ */}
          <div className="card p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">チャット速度推移</h3>
            {chartData.length > 0 ? (
              <div className="h-80">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
                    <XAxis dataKey="time" stroke="#64748b" style={{ fontSize: '12px' }} />
                    <YAxis stroke="#64748b" style={{ fontSize: '12px' }} />
                    <Tooltip 
                      contentStyle={{
                        backgroundColor: 'rgba(255, 255, 255, 0.95)',
                        border: '1px solid #e2e8f0',
                        borderRadius: '8px',
                        boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
                      }}
                    />
                    <Bar dataKey="chatRate" fill="#10b981" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            ) : (
              <div className="h-80 flex items-center justify-center text-gray-500 dark:text-gray-400">
                データがありません
              </div>
            )}
          </div>
        </div>
      )}

      {viewMode === 'channel' && (
        <ChannelStatistics
          channelStats={channelStats}
          dateRange={dateRange}
        />
      )}

      {viewMode === 'session' && (
        <StreamSessionView
          channelId={selectedChannelId}
          dateRange={dateRange}
        />
      )}

      {viewMode === 'chat' && (
        <ChatAnalysis
          dateRange={dateRange}
          selectedChannelId={selectedChannelId}
        />
      )}

      {/* データ件数表示 */}
      <div className="text-sm text-gray-500 dark:text-gray-400 text-center">
        期間: {dateRange.start} 〜 {dateRange.end} |
        データ件数: {stats?.length || 0}件
      </div>
    </div>
  );
}