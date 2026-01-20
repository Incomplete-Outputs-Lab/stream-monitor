import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { LineChart } from "../common/charts/LineChart";
import { BarChart } from "../common/charts/BarChart";

interface ChatStatsQuery {
  stream_id?: number;
  channel_id?: number;
  start_time?: string;
  end_time?: string;
}

interface ChatStats {
  total_messages: number;
  unique_users: number;
  messages_per_minute: number;
  top_users: Array<{
    user_name: string;
    message_count: number;
  }>;
  message_types: Array<{
    message_type: string;
    count: number;
  }>;
  hourly_distribution: Array<{
    hour: number;
    message_count: number;
  }>;
}

interface ChatRateData {
  timestamp: string;
  message_count: number;
  interval_minutes: number;
}

interface ChatAnalysisProps {
  dateRange: { start: string; end: string };
  selectedChannelId: number | null;
}

export function ChatAnalysis({ dateRange, selectedChannelId }: ChatAnalysisProps) {
  const [selectedStreamId, setSelectedStreamId] = useState<number | null>(null);

  // チャット統計取得
  const { data: chatStats, isLoading: statsLoading } = useQuery({
    queryKey: ["chat-stats", selectedChannelId, selectedStreamId, dateRange],
    queryFn: async () => {
      const query: ChatStatsQuery = {
        start_time: new Date(dateRange.start).toISOString(),
        end_time: new Date(dateRange.end + 'T23:59:59').toISOString(),
        channel_id: selectedChannelId || undefined,
        stream_id: selectedStreamId || undefined,
      };

      return await invoke<ChatStats>("get_chat_stats", { query });
    },
  });

  // チャット速度取得
  const { data: chatRates, isLoading: ratesLoading } = useQuery({
    queryKey: ["chat-rates", selectedChannelId, selectedStreamId, dateRange],
    queryFn: async () => {
      const query = {
        start_time: new Date(dateRange.start).toISOString(),
        end_time: new Date(dateRange.end + 'T23:59:59').toISOString(),
        interval_minutes: 5,
        channel_id: selectedChannelId || undefined,
        stream_id: selectedStreamId || undefined,
      };

      return await invoke<ChatRateData[]>("get_chat_rate", { query });
    },
  });

  // チャンネル一覧取得（ストリーム選択用）- 将来的に使用予定
  // const { data: channels } = useQuery({
  //   queryKey: ["channels"],
  //   queryFn: async () => {
  //     return await invoke<Array<{ id: number; channel_name: string; platform: string }>>("list_channels");
  //   },
  // });

  if (statsLoading || ratesLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  // チャート用データ変換
  const rateChartData = chatRates?.map(rate => ({
    time: new Date(rate.timestamp).toLocaleString('ja-JP', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }),
    messages: rate.message_count,
  })) || [];

  const hourlyChartData = chatStats?.hourly_distribution.map(hour => ({
    hour: `${hour.hour}:00`,
    messages: hour.message_count,
  })) || [];

  const topUsersData = chatStats?.top_users.slice(0, 10).map((user, index) => ({
    rank: index + 1,
    user: user.user_name,
    messages: user.message_count,
  })) || [];

  return (
    <div className="space-y-6">
      {/* ストリーム選択 */}
      <div className="bg-white rounded-lg shadow p-4 border border-gray-200">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          ストリーム選択（オプション）
        </label>
        <select
          value={selectedStreamId || ''}
          onChange={(e) => setSelectedStreamId(e.target.value ? parseInt(e.target.value) : null)}
          className="w-full max-w-xs px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="">すべてのストリーム</option>
          {/* TODO: ストリーム一覧を取得して表示 */}
        </select>
      </div>

      {/* チャット統計サマリー */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">💬</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900">
                {chatStats?.total_messages.toLocaleString() || 0}
              </h3>
              <p className="text-sm text-gray-500">総メッセージ数</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-green-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">👥</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900">
                {chatStats?.unique_users.toLocaleString() || 0}
              </h3>
              <p className="text-sm text-gray-500">ユニークユーザー数</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">⚡</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900">
                {chatStats?.messages_per_minute.toFixed(1) || '0.0'}
              </h3>
              <p className="text-sm text-gray-500">1分間平均メッセージ数</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-yellow-500 rounded-full flex items-center justify-center">
                <span className="text-white text-sm font-bold">📊</span>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900">
                {chatStats?.top_users[0]?.message_count || 0}
              </h3>
              <p className="text-sm text-gray-500">最多投稿者メッセージ数</p>
            </div>
          </div>
        </div>
      </div>

      {/* チャート表示 */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* チャット速度推移 */}
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">チャット速度推移</h3>
          {rateChartData.length > 0 ? (
            <LineChart
              data={rateChartData}
              dataKey="messages"
              xAxisKey="time"
              color="#8b5cf6"
              height={300}
              yAxisLabel="メッセージ数"
            />
          ) : (
            <div className="h-64 flex items-center justify-center text-gray-500">
              データがありません
            </div>
          )}
        </div>

        {/* 時間帯別分布 */}
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">時間帯別メッセージ分布</h3>
          {hourlyChartData.length > 0 ? (
            <BarChart
              data={hourlyChartData}
              dataKey="messages"
              xAxisKey="hour"
              color="#f59e0b"
              height={300}
              yAxisLabel="メッセージ数"
            />
          ) : (
            <div className="h-64 flex items-center justify-center text-gray-500">
              データがありません
            </div>
          )}
        </div>
      </div>

      {/* トップユーザー */}
      <div className="bg-white rounded-lg shadow border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">トップ投稿者</h3>
        </div>
        <div className="divide-y divide-gray-200">
          {topUsersData.length > 0 ? (
            topUsersData.map((user) => (
              <div key={user.rank} className="px-6 py-4 flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
                    <span className="text-sm font-medium text-blue-800">#{user.rank}</span>
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900">{user.user}</p>
                  </div>
                </div>
                <div className="text-sm text-gray-500">
                  {user.messages.toLocaleString()} メッセージ
                </div>
              </div>
            ))
          ) : (
            <div className="px-6 py-8 text-center text-gray-500">
              投稿者データがありません
            </div>
          )}
        </div>
      </div>

      {/* メッセージタイプ分布 */}
      {chatStats?.message_types && chatStats.message_types.length > 0 && (
        <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">メッセージタイプ分布</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {chatStats.message_types.map((type) => (
              <div key={type.message_type} className="text-center">
                <div className="text-2xl font-bold text-gray-900">
                  {type.count.toLocaleString()}
                </div>
                <div className="text-sm text-gray-500 capitalize">
                  {type.message_type}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}