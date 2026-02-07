import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import * as gameCategoriesApi from '../../api/gameCategories';

export function GameCategoryManager() {
  const [searchQuery, setSearchQuery] = useState('');
  const queryClient = useQueryClient();

  // 全カテゴリ取得
  const { data: categories, isLoading } = useQuery({
    queryKey: ['game-categories'],
    queryFn: gameCategoriesApi.getGameCategories,
  });

  // 削除ミューテーション
  const deleteMutation = useMutation({
    mutationFn: gameCategoriesApi.deleteGameCategory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['game-categories'] });
    },
  });

  // フィルタリング
  const filteredCategories = categories?.filter((cat) => {
    if (!cat?.gameName || !cat?.gameId) return false;
    return (
      cat.gameName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      cat.gameId.includes(searchQuery)
    );
  }) || [];

  const handleDelete = async (gameId: string) => {
    if (confirm('このカテゴリを削除しますか？')) {
      await deleteMutation.mutateAsync(gameId);
    }
  };

  return (
    <section className="card p-4 space-y-4">
      <div>
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          ゲームカテゴリ管理
        </h2>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Twitch カテゴリID→名前の管理（自動発見された配信から自動登録されます）
        </p>
      </div>

      {/* 検索バー */}
      <div>
        <input
          type="text"
          placeholder="カテゴリ名またはIDで検索..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-purple-500"
        />
      </div>

      {/* カテゴリ数 */}
      <div className="text-xs text-gray-600 dark:text-gray-400">
        登録済み: {categories?.length || 0} カテゴリ
        {searchQuery && ` (${filteredCategories.length} 件表示)`}
      </div>

      {/* カテゴリ一覧 */}
      <div className="max-h-96 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded-lg">
        {isLoading ? (
          <div className="p-4 text-center text-sm text-gray-500 dark:text-gray-400">
            読み込み中...
          </div>
        ) : filteredCategories.length === 0 ? (
          <div className="p-4 text-center text-sm text-gray-500 dark:text-gray-400">
            {searchQuery ? '検索結果がありません' : 'カテゴリがありません'}
          </div>
        ) : (
          <table className="w-full text-xs">
            <thead className="bg-gray-50 dark:bg-gray-800 sticky top-0">
              <tr>
                <th className="px-3 py-2 text-left font-medium text-gray-700 dark:text-gray-300">
                  ゲームID
                </th>
                <th className="px-3 py-2 text-left font-medium text-gray-700 dark:text-gray-300">
                  カテゴリ名
                </th>
                <th className="px-3 py-2 text-left font-medium text-gray-700 dark:text-gray-300">
                  更新日時
                </th>
                <th className="px-3 py-2 text-right font-medium text-gray-700 dark:text-gray-300">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
              {filteredCategories.map((category) => (
                <tr
                  key={category.gameId}
                  className="hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                >
                  <td className="px-3 py-2 font-mono text-gray-600 dark:text-gray-400">
                    {category.gameId}
                  </td>
                  <td className="px-3 py-2 text-gray-900 dark:text-gray-100">
                    {category.gameName}
                  </td>
                  <td className="px-3 py-2 text-gray-600 dark:text-gray-400">
                    {category.lastUpdated
                      ? new Date(category.lastUpdated).toLocaleString('ja-JP')
                      : '-'}
                  </td>
                  <td className="px-3 py-2 text-right">
                    <button
                      onClick={() => handleDelete(category.gameId)}
                      disabled={deleteMutation.isPending}
                      className="text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                      削除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="text-xs text-gray-500 dark:text-gray-400">
        💡 ヒント: カテゴリは配信データの収集時に自動的に登録されます
      </div>
    </section>
  );
}
