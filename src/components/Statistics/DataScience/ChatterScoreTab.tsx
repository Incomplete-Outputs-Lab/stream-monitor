import { useQuery } from '@tanstack/react-query';
import { BarChart } from '../../common/charts/BarChart';
import { StatsDashboardSkeleton } from '../../common/Skeleton';
import { getChatterActivityScores } from '../../../api/statistics';

interface ChatterScoreTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const ChatterScoreTab = ({ channelId, startTime, endTime }: ChatterScoreTabProps) => {
  const { data, isLoading } = useQuery({
    queryKey: ['chatterActivityScores', channelId, startTime, endTime],
    queryFn: () => getChatterActivityScores({
      channelId: channelId!,
      streamId: null,
      startTime,
      endTime,
      limit: 50,
    }),
    enabled: !!channelId,
  });

  // チャンネル選択チェック
  if (channelId === null) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <svg className="h-6 w-6 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
              チャンネルを選択してください
            </h3>
            <p className="mt-2 text-sm text-yellow-700 dark:text-yellow-300">
              チャッタースコア分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <StatsDashboardSkeleton cardCount={2} chartCount={1} />;
  }

  if (!data || data.scores.length === 0) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">
          選択した期間にチャッターデータがありません。
        </p>
      </div>
    );
  }

  const getBadgeColor = (badge: string) => {
    switch (badge) {
      case 'broadcaster':
        return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300';
      case 'moderator':
        return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300';
      case 'vip':
        return 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300';
      case 'subscriber':
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600 dark:text-green-400';
    if (score >= 60) return 'text-blue-600 dark:text-blue-400';
    if (score >= 40) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-gray-600 dark:text-gray-400';
  };

  return (
    <div className="space-y-8">
      {/* Score Distribution */}
      {data.scoreDistribution.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">活性度スコア分布</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            チャッター全体のスコア分布を表示します。スコアは0-100+の範囲で、メッセージ数・参加頻度・継続性を総合評価します。
          </p>
          <BarChart
            data={data.scoreDistribution.map((d) => ({
              range: d.scoreRange,
              count: d.userCount,
            }))}
            xKey="range"
            bars={[{ key: 'count', color: '#8b5cf6' }]}
            height={300}
          />
        </div>
      )}

      {/* Top Chatters Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">トップチャッター</h3>
        <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
          活性度スコア上位のユーザー一覧です。スコア計算式：
          <code className="ml-2 px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs">
            メッセージ数(30%) + 配信参加(30%) + 継続性(20%) + バッジ(20%)
          </code>
        </p>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead className="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  順位
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  ユーザー名
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  バッジ
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  スコア
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  メッセージ数
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                  参加配信数
                </th>
              </tr>
            </thead>
            <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
              {data.scores.map((chatter) => (
                <tr key={chatter.userName} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                  <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                    #{chatter.rank}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                    {chatter.userName}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm">
                    <div className="flex flex-wrap gap-1">
                      {chatter.badges.length > 0 ? (
                        chatter.badges.map((badge) => (
                          <span
                            key={badge}
                            className={`px-2 py-1 rounded text-xs ${getBadgeColor(badge)}`}
                          >
                            {badge}
                          </span>
                        ))
                      ) : (
                        <span className="text-gray-400 dark:text-gray-500 text-xs">なし</span>
                      )}
                    </div>
                  </td>
                  <td className={`px-4 py-3 whitespace-nowrap text-sm font-bold ${getScoreColor(chatter.score)}`}>
                    {chatter.score.toFixed(1)}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                    {chatter.messageCount.toLocaleString()}
                  </td>
                  <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                    {chatter.streamCount.toLocaleString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Score Breakdown Explanation */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-3">
          活性度スコアの計算方法
        </h3>
        <div className="space-y-3 text-sm text-blue-800 dark:text-blue-200">
          <div>
            <p className="font-medium mb-1">📊 スコア構成（合計100点以上）:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong>メッセージ数（30%）:</strong> 総メッセージ数の対数スケール × 10</li>
              <li>• <strong>配信参加率（30%）:</strong> 参加配信数 × 5</li>
              <li>• <strong>継続性（20%）:</strong> 活動日数 × 3</li>
              <li>• <strong>バッジ加重（20%）:</strong> broadcaster=3.0, mod=2.5, VIP=2.0, sub=1.5, regular=1.0</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">🎯 スコアの解釈:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong>80+:</strong> 超アクティブユーザー（コアコミュニティ）</li>
              <li>• <strong>60-79:</strong> アクティブユーザー（熱心なファン）</li>
              <li>• <strong>40-59:</strong> 中程度のアクティブユーザー（定期視聴者）</li>
              <li>• <strong>0-39:</strong> ライトユーザー（カジュアル視聴者）</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">💡 活用方法:</p>
            <ul className="ml-4 space-y-1">
              <li>• トップチャッターを特定してコミュニティマネジメントに活用</li>
              <li>• モデレーター候補の発掘（高スコア + regular バッジ）</li>
              <li>• 長期的なコミュニティ育成の指標として追跡</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChatterScoreTab;
