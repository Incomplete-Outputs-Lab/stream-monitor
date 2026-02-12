import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import * as channelsApi from "../../api/channels";
import BroadcasterAnalytics from "./BroadcasterAnalytics";
import GameAnalytics from "./GameAnalytics";
import TopGamesAnalytics from "./TopGamesAnalytics";
import GameDetailAnalytics from "./GameDetailAnalytics";
import TopChannelsAnalytics from "./TopChannelsAnalytics";
import ChannelDetailAnalytics from "./ChannelDetailAnalytics";
import ChatAnalytics from "./ChatAnalytics";
import DataScience from "./DataScience";
import { DateRangePicker } from "./DateRangePicker";
import { Skeleton } from "../common/Skeleton";
import { OAuthWarningBanner } from "../common/OAuthWarningBanner";

type TabType = "overview" | "broadcaster" | "game" | "topGames" | "gameDetail" | "topChannels" | "channelDetail" | "chatAnalytics" | "dataScience";

export function Statistics() {
  const [activeTab, setActiveTab] = useState<TabType>("overview");
  
  // 日付範囲の初期値（過去7日間）
  const getInitialDateRange = () => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - 7);
    
    // ローカル日付を正しく取得（UTCではなく）
    const formatLocalDate = (date: Date) => {
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      return `${year}-${month}-${day}`;
    };
    
    return {
      start: formatLocalDate(start),
      end: formatLocalDate(end)
    };
  };

  const [dateRange, setDateRange] = useState(getInitialDateRange());
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  
  // ドリルダウン用のstate
  const [selectedGame, setSelectedGame] = useState<string>("");
  const [selectedChannelName, setSelectedChannelName] = useState<string>("");

  // チャンネル一覧取得（ドリルダウンナビゲーション用）
  const { data: channels, isLoading: isLoadingChannels } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      return await channelsApi.listChannels();
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
    { id: "chatAnalytics" as TabType, label: "チャット分析", icon: "💬" },
    { id: "dataScience" as TabType, label: "データサイエンス", icon: "🔬" },
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

      <OAuthWarningBanner />

      {/* フィルタエリア */}
      <div className="card p-4 space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル
          </label>
          {isLoadingChannels ? (
            <Skeleton variant="rectangular" height={42} className="w-full" />
          ) : (
            <select
              value={selectedChannelId || ''}
              onChange={(e) => setSelectedChannelId(e.target.value ? Number(e.target.value) : null)}
              className="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-gray-900 dark:text-white"
            >
              <option value="">全てのチャンネル</option>
              {channels?.map((ch) => (
                <option key={ch.id} value={ch.id}>
                  {ch.display_name || ch.channel_name}
                </option>
              ))}
            </select>
          )}
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            期間
          </label>
          <DateRangePicker
            startDate={dateRange.start}
            endDate={dateRange.end}
            onChange={handleDateRangeChange}
          />
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
                  <button
                    onClick={() => setActiveTab("chatAnalytics")}
                    className="p-6 bg-purple-50 dark:bg-purple-900/20 rounded-lg hover:bg-purple-100 dark:hover:bg-purple-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">💬</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">チャット分析</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      エンゲージメント・セグメント分析
                    </div>
                  </button>
                  <button
                    onClick={() => setActiveTab("dataScience")}
                    className="p-6 bg-cyan-50 dark:bg-cyan-900/20 rounded-lg hover:bg-cyan-100 dark:hover:bg-cyan-900/30 transition-colors"
                  >
                    <div className="text-3xl mb-2">🔬</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">データサイエンス</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      高度な統計分析・機械学習
                    </div>
                  </button>
                </div>
              </div>
            </div>
          )}

          {activeTab === "broadcaster" && (
            <BroadcasterAnalytics
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

          {activeTab === "chatAnalytics" && (
            <ChatAnalytics
              channels={channels || []}
              parentChannelId={selectedChannelId}
              parentDateRange={dateRange}
            />
          )}

          {activeTab === "dataScience" && (
            <DataScience
              channels={channels || []}
              parentChannelId={selectedChannelId}
              parentDateRange={dateRange}
            />
          )}
        </div>
      </div>
    </div>
  );
}