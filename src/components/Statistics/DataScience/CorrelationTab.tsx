import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { CorrelationResult } from '../../../types';
import { BarChart } from '../../common/charts/BarChart';
import { Scatter, ScatterChart, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, ZAxis } from 'recharts';
import { LoadingSpinner } from '../../common/LoadingSpinner';

interface CorrelationTabProps {
  channelId: number | null;
  startTime: string;
  endTime: string;
}

const CorrelationTab = ({ channelId, startTime, endTime }: CorrelationTabProps) => {
  const { data, isLoading } = useQuery({
    queryKey: ['viewerChatCorrelation', channelId, startTime, endTime],
    queryFn: async () => {
      return await invoke<CorrelationResult>('get_viewer_chat_correlation', {
        channelId,
        streamId: null,
        startTime,
        endTime,
      });
    },
  });

  if (isLoading) {
    return <LoadingSpinner />;
  }

  if (!data) {
    return (
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <p className="text-yellow-800 dark:text-yellow-200">データがありません</p>
      </div>
    );
  }

  // Determine correlation strength color
  const getCorrelationColor = (coef: number) => {
    const abs = Math.abs(coef);
    if (abs >= 0.7) return 'text-green-600 dark:text-green-400';
    if (abs >= 0.4) return 'text-blue-600 dark:text-blue-400';
    if (abs >= 0.1) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-gray-600 dark:text-gray-400';
  };

  const getCorrelationBgColor = (coef: number) => {
    const abs = Math.abs(coef);
    if (abs >= 0.7) return 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800';
    if (abs >= 0.4) return 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800';
    if (abs >= 0.1) return 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800';
    return 'bg-gray-50 dark:bg-gray-700 border-gray-200 dark:border-gray-600';
  };

  return (
    <div className="space-y-8">
      {/* Correlation Summary */}
      <div className={`rounded-lg p-6 shadow border ${getCorrelationBgColor(data.pearsonCoefficient)}`}>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">ピアソン相関係数</h3>
        <div className="flex items-center justify-between">
          <div>
            <p className={`text-5xl font-bold ${getCorrelationColor(data.pearsonCoefficient)}`}>
              {data.pearsonCoefficient.toFixed(3)}
            </p>
            <p className="mt-2 text-lg text-gray-700 dark:text-gray-300">{data.interpretation}</p>
          </div>
          <div className="text-right">
            <p className="text-sm text-gray-600 dark:text-gray-400">サンプル数</p>
            <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {data.scatterData.length.toLocaleString()}
            </p>
          </div>
        </div>
        <div className="mt-4 text-sm text-gray-600 dark:text-gray-400">
          <p>
            相関係数は -1 から +1 の範囲で、視聴者数とチャット数の関係性を示します。
          </p>
          <ul className="mt-2 space-y-1">
            <li>• +1.0 に近い: 強い正の相関（視聴者が増えるとチャットも増える）</li>
            <li>• 0.0 に近い: 相関なし</li>
            <li>• -1.0 に近い: 強い負の相関（視聴者が増えるとチャットが減る）</li>
          </ul>
        </div>
      </div>

      {/* Scatter Plot */}
      {data.scatterData.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">散布図: 視聴者数 × チャット数</h3>
          <ResponsiveContainer width="100%" height={400}>
            <ScatterChart margin={{ top: 20, right: 20, bottom: 60, left: 60 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis
                type="number"
                dataKey="viewers"
                name="視聴者数"
                stroke="#9ca3af"
                label={{ value: '視聴者数', position: 'insideBottom', offset: -10, fill: '#9ca3af' }}
              />
              <YAxis
                type="number"
                dataKey="chats"
                name="チャット数"
                stroke="#9ca3af"
                label={{ value: 'チャット数', angle: -90, position: 'insideLeft', fill: '#9ca3af' }}
              />
              <ZAxis range={[50, 50]} />
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
                  if (name === 'viewers') return [value.toLocaleString(), '視聴者数'];
                  if (name === 'chats') return [value.toLocaleString(), 'チャット数'];
                  return [value, name || ''];
                }}
              />
              <Legend />
              <Scatter name="データポイント" data={data.scatterData} fill="#3b82f6" />
            </ScatterChart>
          </ResponsiveContainer>
          <p className="mt-4 text-sm text-gray-600 dark:text-gray-400">
            各点は5分間隔のデータポイントを表します。右上がりの傾向があれば正の相関があります。
          </p>
        </div>
      )}

      {/* Hourly Correlation */}
      {data.hourlyCorrelation.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-6 shadow">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">時間帯別相関係数</h3>
          <BarChart
            data={data.hourlyCorrelation.map((h) => ({
              hour: `${h.hour}時`,
              correlation: h.correlation,
              sampleCount: h.sampleCount,
            }))}
            xKey="hour"
            bars={[{ key: 'correlation', color: '#8b5cf6' }]}
            height={350}
          />
          <div className="mt-4 overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-900">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    時間帯
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    相関係数
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    サンプル数
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">
                    評価
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                {data.hourlyCorrelation.map((h) => {
                  const abs = Math.abs(h.correlation);
                  let evaluation = '';
                  if (abs >= 0.7) evaluation = '強い相関';
                  else if (abs >= 0.4) evaluation = '中程度の相関';
                  else if (abs >= 0.1) evaluation = '弱い相関';
                  else evaluation = '相関なし';

                  return (
                    <tr key={h.hour} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                      <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                        {h.hour}時
                      </td>
                      <td className={`px-4 py-3 whitespace-nowrap text-sm font-medium ${getCorrelationColor(h.correlation)}`}>
                        {h.correlation.toFixed(3)}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                        {h.sampleCount.toLocaleString()}
                      </td>
                      <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">
                        {evaluation}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Insights */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-3">分析のヒント</h3>
        <ul className="space-y-2 text-sm text-blue-800 dark:text-blue-200">
          <li>
            • <strong>強い正の相関 (0.7+)</strong>: 視聴者数が増えるとチャットも活発化。健全なコミュニティの兆候。
          </li>
          <li>
            • <strong>弱い相関 (0.1-0.4)</strong>: 視聴者数に関わらず一定のコアファンがチャットしている可能性。
          </li>
          <li>
            • <strong>相関なし (0.0前後)</strong>: チャット文化が視聴者数に依存していない。特定のイベントでスパイクする可能性。
          </li>
          <li>
            • <strong>時間帯別の違い</strong>: 深夜は少数の熱心なファン、昼間は多数のカジュアル視聴者など、視聴者層の違いを反映。
          </li>
        </ul>
      </div>
    </div>
  );
};

export default CorrelationTab;
