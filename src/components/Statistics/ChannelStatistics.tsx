import { Channel } from "../../types";

interface ChannelStat {
  channel: Channel;
  stats: {
    avgViewers: number;
    maxViewers: number;
    totalChatMessages: number;
    dataPoints: number;
  };
}

interface DateRange {
  start: string;
  end: string;
}

interface ChannelStatisticsProps {
  channelStats: ChannelStat[];
  dateRange: DateRange;
}

export function ChannelStatistics({ channelStats, dateRange }: ChannelStatisticsProps) {
  const platformColors = {
    twitch: "bg-purple-100 text-purple-800",
    youtube: "bg-red-100 text-red-800",
  };

  const platformNames = {
    twitch: "Twitch",
    youtube: "YouTube",
  };

  return (
    <div className="space-y-6">
      <div className="bg-white dark:bg-slate-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            チャンネル別統計 ({dateRange.start} 〜 {dateRange.end})
          </h3>
        </div>

        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {channelStats.length === 0 ? (
            <div className="px-6 py-8 text-center text-gray-500 dark:text-gray-400">
              統計データがありません
            </div>
          ) : (
            channelStats.map(({ channel, stats }) => (
              <div key={`${channel.platform}-${channel.channel_id}`} className="px-6 py-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    {/* プラットフォームバッジ */}
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${platformColors[channel.platform as keyof typeof platformColors]}`}>
                      {platformNames[channel.platform as keyof typeof platformNames]}
                    </span>

                    {/* チャンネル情報 */}
                    <div>
                      <h4 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        {channel.display_name || channel.channel_name}
                      </h4>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {channel.channel_id}
                      </p>
                    </div>
                  </div>

                  {/* 統計データ */}
                  <div className="grid grid-cols-4 gap-8 text-center">
                    <div>
                      <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                        {stats.avgViewers.toLocaleString()}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">平均視聴者数</div>
                    </div>

                    <div>
                      <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                        {stats.maxViewers.toLocaleString()}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">最大視聴者数</div>
                    </div>

                    <div>
                      <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                        {stats.totalChatMessages.toLocaleString()}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">総チャット数</div>
                    </div>

                    <div>
                      <div className="text-2xl font-bold text-gray-600 dark:text-gray-400">
                        {stats.dataPoints}
                      </div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">データポイント</div>
                    </div>
                  </div>
                </div>

                {/* 詳細チャートや追加情報 */}
                <div className="mt-4 grid grid-cols-2 gap-4">
                  <div className="bg-gray-50 dark:bg-slate-700 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">監視間隔</div>
                    <div className="font-semibold text-gray-900 dark:text-gray-100">{channel.poll_interval}秒</div>
                  </div>
                  <div className="bg-gray-50 dark:bg-slate-700 rounded p-3">
                    <div className="text-sm text-gray-600 dark:text-gray-400">ステータス</div>
                    <div className={`font-semibold ${channel.enabled ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>
                      {channel.enabled ? '有効' : '無効'}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* サマリー統計 */}
      {channelStats.length > 0 && (
        <div className="bg-white dark:bg-slate-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">全体サマリー</h3>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
                {Math.round(channelStats.reduce((sum, stat) => sum + stat.stats.avgViewers, 0) / channelStats.length) || 0}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">全チャンネル平均視聴者数</div>
            </div>

            <div className="text-center">
              <div className="text-3xl font-bold text-green-600 dark:text-green-400">
                {Math.max(...channelStats.map(stat => stat.stats.maxViewers))}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">最高視聴者数</div>
            </div>

            <div className="text-center">
              <div className="text-3xl font-bold text-purple-600 dark:text-purple-400">
                {channelStats.reduce((sum, stat) => sum + stat.stats.totalChatMessages, 0)}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">総チャットメッセージ数</div>
            </div>

            <div className="text-center">
              <div className="text-3xl font-bold text-gray-600 dark:text-gray-400">
                {channelStats.reduce((sum, stat) => sum + stat.stats.dataPoints, 0)}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">総データポイント数</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}