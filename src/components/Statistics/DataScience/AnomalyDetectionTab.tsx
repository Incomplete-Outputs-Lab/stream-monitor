import { useQuery } from '@tanstack/react-query';
import { Scatter, ScatterChart, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, ReferenceLine, Cell } from 'recharts';
import { StatCardSkeleton, ChartSkeleton } from '../../common/Skeleton';
import { detectAnomalies } from '../../../api/statistics';

interface AnomalyDetectionTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const AnomalyDetectionTab = ({ channelId, startTime, endTime }: AnomalyDetectionTabProps) => {
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
              異常検知には特定のチャンネルを選択する必要があります。上部のチャンネル選択ドロップダウンから分析対象のチャンネルを選んでください。
            </p>
          </div>
        </div>
      </div>
    );
  }

  const { data, isLoading } = useQuery({
    queryKey: ['anomalyDetection', channelId, startTime, endTime],
    queryFn: () => detectAnomalies({
      channelId,
      streamId: null,
      startTime,
      endTime,
      zThreshold: 2.5,
    }),
    enabled: !!channelId,
  });

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <StatCardSkeleton />
          <StatCardSkeleton />
        </div>
        <ChartSkeleton height={400} />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">データがありません</p>
      </div>
    );
  }

  // フィルター: 1971年以前の不正なタイムスタンプを除外
  const validViewerAnomalies = data.viewerAnomalies.filter(a => {
    const date = new Date(a.timestamp);
    const time = date.getTime();
    // NaN、Invalid Date、または1971年以前を除外
    return !isNaN(time) && time > new Date('1971-01-01').getTime();
  });

  const validChatAnomalies = data.chatAnomalies.filter(a => {
    const date = new Date(a.timestamp);
    const time = date.getTime();
    // NaN、Invalid Date、または1971年以前を除外
    return !isNaN(time) && time > new Date('1971-01-01').getTime();
  });

  const getTrendIcon = (trend: string) => {
    switch (trend) {
      case 'increasing':
        return '📈';
      case 'decreasing':
        return '📉';
      case 'stable':
        return '➡️';
      default:
        return '❓';
    }
  };

  const getTrendColor = (trend: string) => {
    switch (trend) {
      case 'increasing':
        return 'text-green-600 dark:text-green-400';
      case 'decreasing':
        return 'text-red-600 dark:text-red-400';
      case 'stable':
        return 'text-blue-600 dark:text-blue-400';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  };

  return (
    <div className="space-y-8">
      {/* Trend Summary */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Viewer Trend */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">視聴者トレンド</h3>
          <div className="flex items-center justify-between mb-4">
            <div className={`text-4xl font-bold ${getTrendColor(data.trendStats.viewerTrend)}`}>
              {getTrendIcon(data.trendStats.viewerTrend)} {data.trendStats.viewerTrend}
            </div>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">平均値:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.viewerAvg).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">標準偏差:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.viewerStdDev).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">検出異常数:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {validViewerAnomalies.length}
              </span>
            </div>
          </div>
        </div>

        {/* Chat Trend */}
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">チャットトレンド</h3>
          <div className="flex items-center justify-between mb-4">
            <div className={`text-4xl font-bold ${getTrendColor(data.trendStats.chatTrend)}`}>
              {getTrendIcon(data.trendStats.chatTrend)} {data.trendStats.chatTrend}
            </div>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">平均値:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.chatAvg).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">標準偏差:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {Math.round(data.trendStats.chatStdDev).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-600 dark:text-gray-400">検出異常数:</span>
              <span className="font-medium text-gray-900 dark:text-gray-100">
                {data.chatAnomalies.length}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Viewer Anomalies */}
      {validViewerAnomalies.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">視聴者数の異常値</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            Z-scoreが±2.5以上の異常なデータポイントを検出します。スパイク（急増）またはドロップ（急減）を識別できます。
          </p>

          {/* Anomalies Scatter Plot */}
          <div className="mb-6">
            <ResponsiveContainer width="100%" height={350}>
              <ScatterChart margin={{ top: 20, right: 20, bottom: 60, left: 60 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  type="number"
                  dataKey="timestampMs"
                  name="時刻"
                  domain={['dataMin', 'dataMax']}
                  stroke="#9ca3af"
                  tickFormatter={(ts) => new Date(ts).toLocaleDateString('ja-JP')}
                  label={{ value: '日時', position: 'insideBottom', offset: -10, fill: '#9ca3af' }}
                />
                <YAxis
                  type="number"
                  dataKey="value"
                  name="視聴者数"
                  stroke="#9ca3af"
                  label={{ value: '視聴者数', angle: -90, position: 'insideLeft', fill: '#9ca3af' }}
                />
                <Tooltip
                  cursor={{ strokeDasharray: '3 3' }}
                  contentStyle={{
                    backgroundColor: 'rgba(31, 41, 55, 0.9)',
                    border: '1px solid #4b5563',
                    borderRadius: '0.375rem',
                  }}
                  labelStyle={{ color: '#f3f4f6' }}
                  itemStyle={{ color: '#f3f4f6' }}
                  formatter={(value: any, name?: string) => {
                    if (name === 'value') return [value.toLocaleString(), '視聴者数'];
                    if (name === 'zScore') return [value.toFixed(2), 'Z-score'];
                    return [value, name || ''];
                  }}
                  labelFormatter={(label: any) => new Date(label).toLocaleString('ja-JP')}
                />
                <Legend />
                <ReferenceLine y={data.trendStats.viewerAvg} stroke="#3b82f6" strokeDasharray="3 3" />
                <Scatter
                  name="異常値"
                  data={validViewerAnomalies.map((a) => ({
                    timestampMs: new Date(a.timestamp).getTime(),
                    value: a.value,
                    zScore: a.zScore,
                  }))}
                  fill="#ef4444"
                >
                  {validViewerAnomalies.map((a, index) => (
                    <Cell key={index} fill={a.isPositive ? '#10b981' : '#ef4444'} />
                  ))}
                </Scatter>
              </ScatterChart>
            </ResponsiveContainer>
          </div>

          {/* Anomalies Table */}
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    日時
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    視聴者数
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    Z-Score
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    タイプ
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {validViewerAnomalies.map((anomaly, idx) => (
                  <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                    <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                      {new Date(anomaly.timestamp).toLocaleString('ja-JP')}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                      {Math.round(anomaly.value).toLocaleString()}
                    </td>
                    <td className={`px-4 py-3 whitespace-nowrap text-sm font-bold ${
                      anomaly.isPositive ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                    }`}>
                      {anomaly.zScore.toFixed(2)}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap text-sm">
                      <span className={`px-2 py-1 rounded text-xs ${
                        anomaly.isPositive
                          ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300'
                          : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300'
                      }`}>
                        {anomaly.isPositive ? '🚀 スパイク' : '📉 ドロップ'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* No Anomalies Message */}
      {validViewerAnomalies.length === 0 && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
          <p className="text-green-800 dark:text-green-200">
            ✅ 選択した期間に大きな異常値は検出されませんでした。データは安定しています。
          </p>
        </div>
      )}

      {/* Explanation */}
      <div className="bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-indigo-900 dark:text-indigo-100 mb-3">
          異常検知について
        </h3>
        <div className="space-y-3 text-sm text-indigo-800 dark:text-indigo-200">
          <div>
            <p className="font-medium mb-1">📊 Z-Scoreとは:</p>
            <p className="ml-4">
              Z-score = (値 - 平均) / 標準偏差 で計算される統計指標です。
              通常、±2.5以上のZ-scoreは統計的に異常とみなされます（発生確率 &lt; 1%）。
            </p>
          </div>
          <div>
            <p className="font-medium mb-1">🔍 検出される異常の種類:</p>
            <ul className="ml-4 space-y-1">
              <li>• <strong>スパイク（正の異常）:</strong> 視聴者数が急激に増加したポイント</li>
              <li>• <strong>ドロップ（負の異常）:</strong> 視聴者数が急激に減少したポイント</li>
            </ul>
          </div>
          <div>
            <p className="font-medium mb-1">💡 活用方法:</p>
            <ul className="ml-4 space-y-1">
              <li>• スパイク発生時の配信内容を分析してバイラル要因を特定</li>
              <li>• ドロップ発生時の状況を確認して改善策を検討</li>
              <li>• トレンド分析と組み合わせて長期的な成長パターンを把握</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AnomalyDetectionTab;
