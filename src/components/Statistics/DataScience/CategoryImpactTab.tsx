import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { CategoryImpactResult } from '../../../types';
import { BarChart } from '../../common/charts/BarChart';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface CategoryImpactTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const CategoryImpactTab = ({ channelId, startTime, endTime }: CategoryImpactTabProps) => {
  const { data, isLoading } = useQuery({
    queryKey: ['categoryImpact', channelId, startTime, endTime],
    queryFn: async () => {
      if (!channelId) return null;
      return await invoke<CategoryImpactResult>('get_category_change_impact', {
        channelId,
        startTime,
        endTime,
      });
    },
    enabled: !!channelId,
  });

  if (!channelId) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">
          カテゴリ影響分析には単一のチャンネルを選択してください。
        </p>
      </div>
    );
  }

  if (isLoading) {
    return <LoadingSpinner />;
  }

  if (!data || (data.changes.length === 0 && data.categoryPerformance.length === 0)) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">
          選択した期間にカテゴリ変更データがありません。
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Category Changes Section */}
      {data.changes.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">カテゴリ変更履歴</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            カテゴリ変更前後の視聴者数変動を分析します。
          </p>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    日時
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    変更前
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    変更後
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    視聴者変動
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    変動率
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {data.changes.map((change, idx) => {
                  const viewerDiff = change.afterViewers - change.beforeViewers;
                  const isPositive = viewerDiff > 0;
                  const changePercent = change.viewerChangePercent;
                  
                  return (
                    <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                      <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                        {new Date(change.timestamp).toLocaleString('ja-JP')}
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-900 dark:text-gray-100">
                        {change.fromCategory}
                      </td>
                      <td className="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                        {change.toCategory}
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm ${
                        isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                      }`}>
                        {isPositive ? '+' : ''}{viewerDiff.toLocaleString()}
                        <span className="text-xs ml-1">
                          ({change.beforeViewers.toLocaleString()} → {change.afterViewers.toLocaleString()})
                        </span>
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm font-medium ${
                        isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                      }`}>
                        {isPositive ? '+' : ''}{changePercent.toFixed(1)}%
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Category Performance Section */}
      {data.categoryPerformance.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">カテゴリ別パフォーマンス</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            各カテゴリでの平均パフォーマンスを比較します。
          </p>

          {/* Performance Chart */}
          <div className="mb-6">
            <BarChart
              data={data.categoryPerformance.slice(0, 10).map((p) => ({
                category: p.category.length > 20 ? p.category.substring(0, 20) + '...' : p.category,
                avgViewers: Math.round(p.avgViewers),
              }))}
              xKey="category"
              bars={[{ key: 'avgViewers', color: '#3b82f6' }]}
              title="カテゴリ別平均視聴者数（トップ10）"
              height={350}
            />
          </div>

          {/* Performance Table */}
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    カテゴリ
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    平均視聴者数
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    平均チャットレート
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    総配信時間(分)
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {data.categoryPerformance.map((perf, idx) => (
                  <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                    <td className="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                      {perf.category}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {Math.round(perf.avgViewers).toLocaleString()}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {perf.avgChatRate.toFixed(1)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {Math.round(perf.totalTimeMinutes).toLocaleString()}
                      <span className="text-xs text-gray-500 ml-1">
                        ({(perf.totalTimeMinutes / 60).toFixed(1)}h)
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Insights */}
      <div className="bg-purple-50 dark:bg-purple-900/20 border border-purple-200 dark:border-purple-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-purple-900 dark:text-purple-100 mb-3">
          カテゴリ変更の効果分析
        </h3>
        <div className="space-y-3 text-sm text-purple-800 dark:text-purple-200">
          <div>
            <p className="font-medium mb-1">✅ 効果的なカテゴリ変更の特徴:</p>
            <ul className="ml-4 space-y-1">
              <li>• 視聴者数が10%以上増加する変更</li>
              <li>• 一貫して高パフォーマンスを示すカテゴリへの変更</li>
              <li>• ゴールデンタイムに合わせたカテゴリ変更</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">⚠️ 注意が必要なパターン:</p>
            <ul className="ml-4 space-y-1">
              <li>• 頻繁なカテゴリ変更（視聴者が離脱しやすい）</li>
              <li>• 20%以上の視聴者減少を伴う変更</li>
              <li>• 低パフォーマンスカテゴリへの変更</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">💡 最適化のヒント:</p>
            <ul className="ml-4 space-y-1">
              <li>• 高パフォーマンスカテゴリの配信時間を増やす</li>
              <li>• 視聴者減少を引き起こすカテゴリを避ける</li>
              <li>• カテゴリ変更のタイミングを最適化する</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CategoryImpactTab;
