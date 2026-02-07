import { useQuery } from '@tanstack/react-query';
import { BarChart } from '../../common/charts/BarChart';
import { LineChart } from '../../common/charts/LineChart';
import { StatsDashboardSkeleton } from '../../common/Skeleton';
import { getWordFrequencyAnalysis, getEmoteAnalysis, getMessageLengthStats } from '../../../api/statistics';

interface WordAnalysisTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const WordAnalysisTab = ({ channelId, startTime, endTime }: WordAnalysisTabProps) => {
  // Word Frequency Query
  const { data: wordData, isLoading: wordLoading } = useQuery({
    queryKey: ['wordFrequency', channelId, startTime, endTime],
    queryFn: () => getWordFrequencyAnalysis({
      channelId: channelId!,
      streamId: null,
      startTime,
      endTime,
      limit: 50,
    }),
    enabled: !!channelId,
  });

  // Emote Analysis Query
  const { data: emoteData, isLoading: emoteLoading } = useQuery({
    queryKey: ['emoteAnalysis', channelId, startTime, endTime],
    queryFn: () => getEmoteAnalysis({
      channelId: channelId!,
      streamId: null,
      startTime,
      endTime,
    }),
    enabled: !!channelId,
  });

  // Message Length Stats Query
  const { data: lengthData, isLoading: lengthLoading } = useQuery({
    queryKey: ['messageLengthStats', channelId, startTime, endTime],
    queryFn: () => getMessageLengthStats({
      channelId: channelId!,
      streamId: null,
      startTime,
      endTime,
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
              ワード分析には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (wordLoading || emoteLoading || lengthLoading) {
    return <StatsDashboardSkeleton cardCount={4} chartCount={3} />;
  }

  return (
    <div className="space-y-8">
      {/* Word Frequency Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">頻出ワード分析</h3>
        
        {/* Summary Cards */}
        {wordData && (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
            <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
              <p className="text-sm text-blue-600 dark:text-blue-400">総単語数</p>
              <p className="text-2xl font-bold text-blue-900 dark:text-blue-100">
                {wordData.totalWords.toLocaleString()}
              </p>
            </div>
            <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
              <p className="text-sm text-green-600 dark:text-green-400">ユニーク単語</p>
              <p className="text-2xl font-bold text-green-900 dark:text-green-100">
                {wordData.uniqueWords.toLocaleString()}
              </p>
            </div>
            <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
              <p className="text-sm text-purple-600 dark:text-purple-400">平均単語/メッセージ</p>
              <p className="text-2xl font-bold text-purple-900 dark:text-purple-100">
                {wordData.avgWordsPerMessage.toFixed(2)}
              </p>
            </div>
            <div className="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-4">
              <p className="text-sm text-orange-600 dark:text-orange-400">総メッセージ</p>
              <p className="text-2xl font-bold text-orange-900 dark:text-orange-100">
                {wordData.totalMessages.toLocaleString()}
              </p>
            </div>
          </div>
        )}

        {/* Top Words Chart */}
        {wordData && wordData.words.length > 0 && (
          <div className="mt-6">
            <BarChart
              data={wordData.words.slice(0, 20).map((w) => ({
                name: w.word,
                count: w.count,
              }))}
              xKey="name"
              bars={[{ key: 'count', color: '#3b82f6' }]}
              title="トップ20頻出ワード"
              height={400}
            />
          </div>
        )}

        {/* Top Words Table */}
        {wordData && wordData.words.length > 0 && (
          <div className="mt-6 overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    順位
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    単語
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    出現回数
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    割合
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {wordData.words.slice(0, 30).map((word, idx) => (
                  <tr key={word.word} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {idx + 1}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                      {word.word}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {word.count.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {word.percentage.toFixed(2)}%
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Emote Analysis Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">エモート使用分析</h3>
        
        {/* Emote Summary */}
        {emoteData && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
            <div className="bg-indigo-50 dark:bg-indigo-900/20 rounded-lg p-4">
              <p className="text-sm text-indigo-600 dark:text-indigo-400">総エモート使用</p>
              <p className="text-2xl font-bold text-indigo-900 dark:text-indigo-100">
                {emoteData.totalEmoteUses.toLocaleString()}
              </p>
            </div>
            <div className="bg-pink-50 dark:bg-pink-900/20 rounded-lg p-4">
              <p className="text-sm text-pink-600 dark:text-pink-400">エモート/メッセージ</p>
              <p className="text-2xl font-bold text-pink-900 dark:text-pink-100">
                {emoteData.emotePerMessageRate.toFixed(2)}
              </p>
            </div>
          </div>
        )}

        {/* Top Emotes Chart */}
        {emoteData && emoteData.emotes.length > 0 && (
          <div className="mt-6">
            <BarChart
              data={emoteData.emotes.slice(0, 15).map((e) => ({
                name: e.name,
                count: e.count,
                users: e.users,
              }))}
              xKey="name"
              bars={[
                { key: 'count', color: '#8b5cf6' },
                { key: 'users', color: '#ec4899' },
              ]}
              title="トップエモート"
              height={350}
            />
          </div>
        )}

        {/* Hourly Emote Pattern */}
        {emoteData && emoteData.hourlyPattern.length > 0 && (
          <div className="mt-6">
            <LineChart
              data={emoteData.hourlyPattern.map((h) => ({
                hour: `${h.hour}時`,
                count: h.count,
              }))}
              xKey="hour"
              lines={[{ key: 'count', color: '#8b5cf6' }]}
              title="時間帯別エモート使用パターン"
              height={300}
            />
          </div>
        )}
      </div>

      {/* Message Length Stats Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">メッセージ長統計</h3>
        
        {/* Length Stats Summary */}
        {lengthData && (
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mb-6">
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">平均</p>
              <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {lengthData.avgLength.toFixed(1)}
              </p>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">中央値</p>
              <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {lengthData.medianLength.toFixed(1)}
              </p>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">標準偏差</p>
              <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {lengthData.stdDev.toFixed(1)}
              </p>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">最小</p>
              <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {lengthData.minLength}
              </p>
            </div>
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
              <p className="text-sm text-gray-600 dark:text-gray-400">最大</p>
              <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {lengthData.maxLength}
              </p>
            </div>
          </div>
        )}

        {/* Length Distribution Chart */}
        {lengthData && lengthData.distribution.length > 0 && (
          <div className="mt-6">
            <BarChart
              data={lengthData.distribution.map((d) => ({
                bucket: d.bucket,
                count: d.count,
              }))}
              xKey="bucket"
              bars={[{ key: 'count', color: '#10b981' }]}
              title="メッセージ長分布"
              height={300}
            />
          </div>
        )}

        {/* By Segment Stats */}
        {lengthData && lengthData.bySegment.length > 0 && (
          <div className="mt-6">
            <BarChart
              data={lengthData.bySegment.map((s) => ({
                segment: s.segment,
                avgLength: s.avgLength,
              }))}
              xKey="segment"
              bars={[{ key: 'avgLength', color: '#f59e0b' }]}
              title="セグメント別平均メッセージ長"
              height={300}
            />
          </div>
        )}
      </div>

      {/* No Data Message */}
      {wordData && wordData.words.length === 0 && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <p className="text-yellow-800 dark:text-yellow-200">
            選択した期間にデータがありません。チャットメッセージが記録されていることを確認してください。
          </p>
        </div>
      )}
    </div>
  );
};

export default WordAnalysisTab;
