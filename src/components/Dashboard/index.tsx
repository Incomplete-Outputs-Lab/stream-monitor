import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { ChannelWithStats, CollectorStatus } from "../../types";

interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
}

interface LiveChannelCardProps {
  channel: ChannelWithStats;
}

function LiveChannelCard({ channel }: LiveChannelCardProps) {
  return (
    <div className="card p-6 hover:shadow-md transition-all duration-200 animate-fade-in">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center space-x-2 mb-1">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 truncate">
              {channel.channel_name}
            </h3>
          </div>
          <p className="text-sm text-gray-500 dark:text-gray-400 capitalize">
            {channel.platform === 'twitch' ? '🎮 Twitch' : '▶️ YouTube'}
          </p>
        </div>
        <div className="text-right ml-4">
          <div className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
            {channel.current_viewers?.toLocaleString() || 0}
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400 font-medium">視聴者</div>
        </div>
      </div>

      {channel.current_title && (
        <div className="mt-4 pt-4 border-t border-gray-200 dark:border-slate-700">
          <p className="text-sm text-gray-700 dark:text-gray-300 truncate" title={channel.current_title}>
            {channel.current_title}
          </p>
        </div>
      )}

      <div className="mt-4 flex items-center">
        <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-green-400 to-emerald-500 text-white shadow-sm">
          <span className="w-2 h-2 bg-white rounded-full mr-2 animate-pulse"></span>
          ライブ中
        </span>
      </div>
    </div>
  );
}

interface ViewerChartProps {
  data: StreamStats[];
}

interface CollectorStatusPanelProps {
  statuses: CollectorStatus[];
}

function CollectorStatusPanel({ statuses }: CollectorStatusPanelProps) {
  const formatTime = (timestamp?: string) => {
    if (!timestamp) return '-';
    return new Date(timestamp).toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  const getSuccessRate = (status: CollectorStatus) => {
    if (status.poll_count === 0) return '-';
    const successCount = status.poll_count - status.error_count;
    return `${Math.round((successCount / status.poll_count) * 100)}%`;
  };

  return (
    <div className="card p-6 animate-fade-in">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">コレクター状態</h3>
        <span className="text-xs font-medium text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-slate-700 px-2 py-1 rounded-full">
          {statuses.length}件
        </span>
      </div>
      <div className="space-y-3 max-h-96 overflow-y-auto">
        {statuses.length > 0 ? (
          statuses.map((status) => (
            <div
              key={status.channel_id}
              className="border border-gray-200 dark:border-slate-700 rounded-lg p-4 hover:bg-gray-50 dark:hover:bg-slate-800 transition-colors"
            >
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center space-x-2">
                  {status.is_running ? (
                    <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></span>
                  ) : (
                    <span className="w-2 h-2 bg-gray-400 rounded-full"></span>
                  )}
                  <span className="font-medium text-gray-900 dark:text-gray-100">
                    {status.channel_name}
                  </span>
                  <span className="text-xs text-gray-500 dark:text-gray-400 capitalize">
                    {status.platform === 'twitch' ? '🎮' : '▶️'} {status.platform}
                  </span>
                </div>
                <span className={`text-xs font-medium px-2 py-1 rounded ${
                  status.is_running 
                    ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' 
                    : 'bg-gray-100 text-gray-700 dark:bg-slate-700 dark:text-gray-300'
                }`}>
                  {status.is_running ? '動作中' : '停止'}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2 text-xs text-gray-600 dark:text-gray-400 mt-2">
                <div>
                  <span className="font-medium">最終ポーリング:</span> {formatTime(status.last_poll_at)}
                </div>
                <div>
                  <span className="font-medium">最終成功:</span> {formatTime(status.last_success_at)}
                </div>
                <div>
                  <span className="font-medium">ポーリング回数:</span> {status.poll_count}
                </div>
                <div>
                  <span className="font-medium">成功率:</span> {getSuccessRate(status)}
                </div>
              </div>
              {status.last_error && (
                <div className="mt-2 text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
                  ⚠ {status.last_error}
                </div>
              )}
            </div>
          ))
        ) : (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            コレクターが実行されていません
          </div>
        )}
      </div>
    </div>
  );
}

function ViewerChart({ data }: ViewerChartProps) {
  // データをグラフ用に変換
  const chartData = data.slice(-20).map(stat => ({
    time: new Date(stat.collected_at).toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit'
    }),
    viewers: stat.viewer_count || 0,
    chatRate: stat.chat_rate_1min,
  }));

  return (
    <div className="card p-6 animate-fade-in">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">視聴者数推移</h3>
        <div className="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
          <div className="w-3 h-3 rounded-full bg-gradient-to-r from-blue-500 to-indigo-600"></div>
          <span>視聴者数</span>
        </div>
      </div>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
            <XAxis 
              dataKey="time" 
              stroke="#64748b"
              style={{ fontSize: '12px' }}
            />
            <YAxis 
              stroke="#64748b"
              style={{ fontSize: '12px' }}
            />
            <Tooltip 
              contentStyle={{
                backgroundColor: 'rgba(255, 255, 255, 0.95)',
                border: '1px solid #e2e8f0',
                borderRadius: '8px',
                boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
              }}
            />
            <Line
              type="monotone"
              dataKey="viewers"
              stroke="url(#colorGradient)"
              strokeWidth={3}
              dot={false}
              activeDot={{ r: 6 }}
            />
            <defs>
              <linearGradient id="colorGradient" x1="0" y1="0" x2="1" y2="0">
                <stop offset="0%" stopColor="#3b82f6" />
                <stop offset="100%" stopColor="#6366f1" />
              </linearGradient>
            </defs>
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

export function Dashboard() {
  const [statsData, setStatsData] = useState<StreamStats[]>([]);

  // コレクターステータスを取得
  const { data: collectorStatuses } = useQuery({
    queryKey: ["collector-status"],
    queryFn: async () => {
      return await invoke<CollectorStatus[]>("get_collector_status");
    },
    refetchInterval: 10000, // 10秒ごとに更新
  });

  // ライブチャンネルを取得
  const { data: liveChannels, isLoading: channelsLoading } = useQuery({
    queryKey: ["live-channels"],
    queryFn: async () => {
      return await invoke<ChannelWithStats[]>("get_live_channels");
    },
    refetchInterval: 30000, // 30秒ごとに更新
    staleTime: 10000, // 10秒間はキャッシュを使用
    gcTime: 60000, // 1分間キャッシュを保持
  });

  // 最新の統計データを取得
  const { data: recentStats } = useQuery({
    queryKey: ["recent-stats"],
    queryFn: async () => {
      return await invoke<StreamStats[]>("get_stream_stats", {
        query: {
          start_time: new Date(Date.now() - 3600000).toISOString(), // 1時間前から
        },
      });
    },
    refetchInterval: 10000, // 10秒ごとに更新
  });

  useEffect(() => {
    if (recentStats) {
      setStatsData(recentStats);
    }
  }, [recentStats]);

  const totalViewers = liveChannels?.reduce((sum, channel) => sum + (channel.current_viewers || 0), 0) || 0;

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">ダッシュボード</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">リアルタイム統計情報</p>
        </div>
        <div className="text-right">
          <div className="text-sm font-medium text-gray-600 dark:text-gray-400">最終更新</div>
          <div className="text-sm text-gray-500 dark:text-gray-500">
            {new Date().toLocaleTimeString('ja-JP')}
          </div>
        </div>
      </div>

      {/* 概要統計 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">{liveChannels?.length || 0}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">ライブ中チャンネル</p>
            </div>
          </div>
        </div>

        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in" style={{ animationDelay: '0.1s' }}>
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-green-500 to-emerald-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">{totalViewers.toLocaleString()}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">総視聴者数</p>
            </div>
          </div>
        </div>

        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in" style={{ animationDelay: '0.2s' }}>
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-purple-500 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                {statsData.length > 0 ? statsData[statsData.length - 1]?.chat_rate_1min || 0 : 0}
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">1分間チャット数</p>
            </div>
          </div>
        </div>
      </div>

      {/* コレクターステータス */}
      {collectorStatuses && collectorStatuses.length > 0 && (
        <CollectorStatusPanel statuses={collectorStatuses} />
      )}

      {/* チャートとライブチャンネル */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ViewerChart data={statsData} />

        <div className="card p-6 animate-fade-in">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">ライブ中チャンネル</h3>
            {liveChannels && liveChannels.length > 0 && (
              <span className="text-xs font-medium text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-slate-700 px-2 py-1 rounded-full">
                {liveChannels.length}件
              </span>
            )}
          </div>
          <div className="space-y-4 max-h-96 overflow-y-auto">
            {channelsLoading ? (
              <div className="text-center py-12">
                <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600 mx-auto"></div>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-3 font-medium">読み込み中...</p>
              </div>
            ) : liveChannels && liveChannels.length > 0 ? (
              liveChannels.map((channel) => (
                <LiveChannelCard key={`${channel.platform}-${channel.channel_id}`} channel={channel} />
              ))
            ) : (
              <div className="text-center py-12">
                <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
                  <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </div>
                <p className="text-gray-500 dark:text-gray-400 font-medium">現在ライブ中のチャンネルはありません</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}