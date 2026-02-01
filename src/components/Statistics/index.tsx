import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import BroadcasterAnalytics from "./BroadcasterAnalytics";
import GameAnalytics from "./GameAnalytics";
import TopGamesAnalytics from "./TopGamesAnalytics";
import GameDetailAnalytics from "./GameDetailAnalytics";
import TopChannelsAnalytics from "./TopChannelsAnalytics";
import ChannelDetailAnalytics from "./ChannelDetailAnalytics";
import { DateRangePicker } from "./DateRangePicker";
import { Channel } from "../../types";

type TabType = "overview" | "broadcaster" | "game" | "topGames" | "gameDetail" | "topChannels" | "channelDetail";

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
  
  // ドリルダウン用のstate
  const [selectedGame, setSelectedGame] = useState<string>("");
  const [selectedChannelName, setSelectedChannelName] = useState<string>("");

  // チャンネル一覧取得
  const { data: channels } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      return await invoke<Channel[]>("list_channels");
    },
  });

  const handleDateRangeChange = (start: string, end: string) => {
    setDateRange({ start, end });
  };

  const tabs = [
    { id: "overview" as TabType, label: "概要", icon: "📊" },
    { id: "broadcaster" as TabType, label: "配信者分析", icon: "👤" },
    { id: "game" as TabType, label: "ゲーム分析", icon: "🎮" },
    { id: "topGames" as TabType, label: "トップゲーム", icon: "🏆" },
    { id: "topChannels" as TabType, label: "トップチャンネル", icon: "⭐" },
  ];
  
  // ドリルダウンナビゲーションハンドラー
  const handleGameClick = (category: string) => {
    setSelectedGame(category);
    setActiveTab("gameDetail");
  };
  
  const handleChannelClick = (channelId: number) => {
    const channel = channels?.find(ch => ch.id === channelId);
    if (channel) {
      setSelectedChannelId(channelId);
      setSelectedChannelName(channel.display_name || channel.channel_name);
      setActiveTab("channelDetail");
    }
  };
  
  const handleBackToTopGames = () => {
    setSelectedGame("");
    setActiveTab("topGames");
  };
  
  const handleBackToTopChannels = () => {
    setSelectedChannelId(null);
    setSelectedChannelName("");
    setActiveTab("topChannels");
  };

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
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-4xl mx-auto">
                  <button
                    onClick={() => setActiveTab("broadcaster")}
                    className="p-6 bg-indigo-50 dark:bg-indigo-900/20 rounded-lg hover:bg-indigo-100 dark:hover:bg-indigo-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">👤</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">配信者分析</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      MinutesWatched・放送時間等
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("game")}
                    className="p-6 bg-pink-50 dark:bg-pink-900/20 rounded-lg hover:bg-pink-100 dark:hover:bg-pink-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">🎮</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">ゲーム分析</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      ゲームタイトル別統計
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("topGames")}
                    className="p-6 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg hover:bg-yellow-100 dark:hover:bg-yellow-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">🏆</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">トップゲーム</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      ゲームランキング
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("topChannels")}
                    className="p-6 bg-green-50 dark:bg-green-900/20 rounded-lg hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">⭐</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">トップチャンネル</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      チャンネルランキング
                    </div>
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === "broadcaster" && (
            <BroadcasterAnalytics
              channelId={selectedChannelId || undefined}
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
            />
          )}

          {activeTab === "game" && (
            <GameAnalytics
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
            />
          )}

          {activeTab === "topGames" && (
            <TopGamesAnalytics
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
              onGameClick={handleGameClick}
            />
          )}

          {activeTab === "gameDetail" && selectedGame && (
            <GameDetailAnalytics
              category={selectedGame}
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
              onBackClick={handleBackToTopGames}
              onChannelClick={handleChannelClick}
            />
          )}

          {activeTab === "topChannels" && (
            <TopChannelsAnalytics
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
              onChannelClick={handleChannelClick}
            />
          )}

          {activeTab === "channelDetail" && selectedChannelId && selectedChannelName && (
            <ChannelDetailAnalytics
              channelId={selectedChannelId}
              channelName={selectedChannelName}
              startTime={dateRange.start + 'T00:00:00'}
              endTime={dateRange.end + 'T23:59:59'}
              onBackClick={handleBackToTopChannels}
              onGameClick={handleGameClick}
            />
          )}
        </div>
      </div>
    </div>
  );
}