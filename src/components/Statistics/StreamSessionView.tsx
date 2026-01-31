import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

interface DateRange {
  start: string;
  end: string;
}

interface StreamSessionViewProps {
  channelId: number | null;
  dateRange: DateRange;
}

interface StreamStats {
  id?: number;
  stream_id: number;
  collected_at: string;
  viewer_count?: number;
  chat_rate_1min: number;
}

export function StreamSessionView({ channelId, dateRange }: StreamSessionViewProps) {
  // ストリームセッションデータ取得（簡易実装）
  // 実際にはバックエンドでストリームセッション情報を取得する必要がある
  const { data: sessions, isLoading } = useQuery({
    queryKey: ["stream-sessions", channelId, dateRange],
    queryFn: async () => {
      // 現時点では統計データからセッションを推定
      // 本来はstreamsテーブルから直接取得すべき
      const stats = await invoke<StreamStats[]>("get_stream_stats", {
        query: {
          start_time: new Date(dateRange.start).toISOString(),
          end_time: new Date(dateRange.end + 'T23:59:59').toISOString(),
          channel_id: channelId || undefined,
        },
      });

      // 統計データからセッションをグループ化（簡易実装）
      const sessionsMap = new Map<string, StreamStats[]>();

      stats.forEach(stat => {
        const date = new Date(stat.collected_at).toDateString();
        if (!sessionsMap.has(date)) {
          sessionsMap.set(date, []);
        }
        sessionsMap.get(date)!.push(stat);
      });

      return Array.from(sessionsMap.entries()).map(([date, sessionStats]) => ({
        date,
        stats: sessionStats,
        avgViewers: Math.round(sessionStats.reduce((sum, stat) => sum + (stat.viewer_count || 0), 0) / sessionStats.length),
        maxViewers: Math.max(...sessionStats.map(stat => stat.viewer_count || 0)),
        totalChat: sessionStats.reduce((sum, stat) => sum + stat.chat_rate_1min, 0),
        duration: sessionStats.length, // 簡易的な持続時間
      }));
    },
  });

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="bg-white dark:bg-slate-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          配信セッション一覧 ({dateRange.start} 〜 {dateRange.end})
        </h3>

        {sessions && sessions.length > 0 ? (
          <div className="space-y-4">
            {sessions.map((session, index) => (
              <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
                <div className="flex justify-between items-start mb-4">
                  <div>
                    <h4 className="font-semibold text-gray-900 dark:text-gray-100">
                      {new Date(session.date).toLocaleDateString('ja-JP', {
                        year: 'numeric',
                        month: 'long',
                        day: 'numeric',
                        weekday: 'long'
                      })}
                    </h4>
                    <p className="text-sm text-gray-500 dark:text-gray-400">
                      データポイント: {session.duration}個
                    </p>
                  </div>

                  <div className="grid grid-cols-3 gap-6 text-center">
                    <div>
                      <div className="text-lg font-bold text-blue-600 dark:text-blue-400">
                        {session.avgViewers.toLocaleString()}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">平均視聴者数</div>
                    </div>
                    <div>
                      <div className="text-lg font-bold text-green-600 dark:text-green-400">
                        {session.maxViewers.toLocaleString()}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">最高視聴者数</div>
                    </div>
                    <div>
                      <div className="text-lg font-bold text-purple-600 dark:text-purple-400">
                        {session.totalChat.toLocaleString()}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400">総チャット数</div>
                    </div>
                  </div>
                </div>

                {/* セッションの詳細チャート */}
                <div className="h-32">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={session.stats.map((stat, idx) => ({
                      time: idx,
                      viewers: stat.viewer_count || 0,
                      chatRate: stat.chat_rate_1min,
                    }))}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="time" />
                      <YAxis />
                      <Tooltip />
                      <Bar dataKey="viewers" fill="#3b82f6" />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-gray-500 dark:text-gray-400">
            指定期間内の配信セッションデータがありません
          </div>
        )}
      </div>

      {/* セッション統計サマリー */}
      {sessions && sessions.length > 0 && (
        <div className="bg-white dark:bg-slate-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">セッション統計サマリー</h3>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                {sessions.length}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">総セッション数</div>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                {Math.max(...sessions.map(s => s.maxViewers))}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">全期間最高視聴者数</div>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                {Math.round(sessions.reduce((sum, s) => sum + s.avgViewers, 0) / sessions.length)}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">平均視聴者数（全セッション）</div>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-gray-600 dark:text-gray-400">
                {sessions.reduce((sum, s) => sum + s.totalChat, 0)}
              </div>
              <div className="text-sm text-gray-500 dark:text-gray-400">総チャットメッセージ数</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}