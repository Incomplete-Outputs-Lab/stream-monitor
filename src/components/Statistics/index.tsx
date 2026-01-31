import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { ChannelStatistics } from "./ChannelStatistics";
import { ChatAnalysis } from "./ChatAnalysis";
import { StreamSessionView } from "./StreamSessionView";
import { DateRangePicker } from "./DateRangePicker";
import { Channel, StreamStats } from "../../types";

type TabType = "overview" | "channels" | "chat" | "sessions";

interface ChannelStat {
  channel: Channel;
  stats: {
    avgViewers: number;
    maxViewers: number;
    totalChatMessages: number;
    dataPoints: number;
  };
}

export function Statistics() {
  const [activeTab, setActiveTab] = useState<TabType>("overview");
  
  // 日付範囲の初期値（過去7日間）
  const getInitialDateRange = () => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - 7);
    return {
      start: start.toISOString().split('T')[0],
      end: end.toISOString().split('T')[0]
    };
  };

  const [dateRange, setDateRange] = useState(getInitialDateRange());
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);

  // チャンネル一覧取得
  const { data: channels } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      return await invoke<Channel[]>("list_channels");
    },
  });

  // チャンネル統計データ取得
  const { data: channelStats, isLoading: channelStatsLoading } = useQuery({
    queryKey: ["channel-statistics", dateRange, selectedChannelId],
    queryFn: async () => {
      const channelList = selectedChannelId 
        ? channels?.filter(ch => ch.id === selectedChannelId) || []
        : channels || [];

      const statsPromises = channelList.map(async (channel) => {
        const stats = await invoke<StreamStats[]>("get_stream_stats", {
          query: {
            channel_id: channel.id,
            start_time: new Date(dateRange.start).toISOString(),
            end_time: new Date(dateRange.end + 'T23:59:59').toISOString(),
          },
        });

        const viewerCounts = stats.map(s => s.viewer_count || 0);
        const totalChatMessages = stats.reduce((sum, s) => sum + s.chat_rate_1min, 0);

        return {
          channel,
          stats: {
            avgViewers: viewerCounts.length > 0 
              ? Math.round(viewerCounts.reduce((sum, v) => sum + v, 0) / viewerCounts.length)
              : 0,
            maxViewers: viewerCounts.length > 0 ? Math.max(...viewerCounts) : 0,
            totalChatMessages,
            dataPoints: stats.length,
          },
        } as ChannelStat;
      });

      return await Promise.all(statsPromises);
    },
    enabled: !!channels && activeTab === "channels",
  });

  const handleDateRangeChange = (start: string, end: string) => {
    setDateRange({ start, end });
  };

  const tabs = [
    { id: "overview" as TabType, label: "概要", icon: "📊" },
    { id: "channels" as TabType, label: "チャンネル統計", icon: "📺" },
    { id: "chat" as TabType, label: "チャット分析", icon: "💬" },
    { id: "sessions" as TabType, label: "セッション履歴", icon: "📅" },
  ];

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">統計閲覧</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">過去の統計データを閲覧・分析</p>
      </div>

      {/* フィルタエリア */}
      <div className="card p-4 space-y-4">
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex-1 min-w-[300px]">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              期間
            </label>
            <DateRangePicker
              startDate={dateRange.start}
              endDate={dateRange.end}
              onChange={handleDateRangeChange}
            />
          </div>

          <div className="flex-1 min-w-[200px]">
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
        </div>
      </div>

      {/* タブナビゲーション */}
      <div className="card">
        <div className="border-b border-gray-200 dark:border-gray-700">
          <nav className="flex -mb-px">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  flex-1 py-4 px-4 text-center border-b-2 font-medium text-sm transition-colors
                  ${activeTab === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'
                  }
                `}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        {/* タブコンテンツ */}
        <div className="p-6">
          {activeTab === "overview" && (
            <div className="space-y-6">
              <div className="text-center py-8">
                <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
                  <span className="text-4xl">📊</span>
                </div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
                  統計概要
                </h3>
                <p className="text-gray-600 dark:text-gray-400 mb-6">
                  タブを選択して詳細な統計データを確認してください
                </p>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 max-w-3xl mx-auto">
                  <button
                    onClick={() => setActiveTab("channels")}
                    className="p-6 bg-purple-50 dark:bg-purple-900/20 rounded-lg hover:bg-purple-100 dark:hover:bg-purple-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">📺</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">チャンネル統計</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      チャンネル別の詳細統計
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("chat")}
                    className="p-6 bg-green-50 dark:bg-green-900/20 rounded-lg hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">💬</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">チャット分析</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      チャットメッセージの分析
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("sessions")}
                    className="p-6 bg-blue-50 dark:bg-blue-900/20 rounded-lg hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">📅</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">セッション履歴</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      配信セッションの履歴
                    </div>
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === "channels" && (
            channelStatsLoading ? (
              <div className="flex justify-center items-center h-64">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
              </div>
            ) : (
              <ChannelStatistics
                channelStats={channelStats || []}
                dateRange={dateRange}
              />
            )
          )}

          {activeTab === "chat" && (
            <ChatAnalysis
              dateRange={dateRange}
              selectedChannelId={selectedChannelId}
            />
          )}

          {activeTab === "sessions" && (
            <StreamSessionView
              channelId={selectedChannelId}
              dateRange={dateRange}
            />
          )}
        </div>
      </div>
    </div>
  );
}