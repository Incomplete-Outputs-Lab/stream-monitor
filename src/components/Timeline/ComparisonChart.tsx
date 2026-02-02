import React, { useMemo } from 'react';
import {
  ComposedChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import { StreamTimelineData, SelectedStream } from '../../types';
import {
  normalizeTimelineData,
  extractComparisonEvents,
  formatElapsedTime,
  formatAbsoluteTime,
  truncateText,
} from './utils';

interface ComparisonChartProps {
  timelines: StreamTimelineData[];
  selectedStreams: SelectedStream[];
}

const ComparisonChart: React.FC<ComparisonChartProps> = ({ timelines, selectedStreams }) => {
  // グラフ用のデータを準備
  const chartData = useMemo(() => {
    if (timelines.length === 0) return [];

    // すべての配信データを正規化
    const allNormalized = timelines.map((timeline, index) => ({
      timeline,
      normalized: normalizeTimelineData(timeline, index),
      color: selectedStreams[index]?.color || '#000000',
    }));

    // データポイントが空の配信を除外
    const validNormalized = allNormalized.filter((data) => data.normalized.length > 0);
    
    if (validNormalized.length === 0) return [];

    // すべての配信データを1つの配列にマージ
    const allPoints: Array<{
      timestamp: string;
      timestampMs: number;
      streamId: number;
      viewer_count: number;
    }> = [];

    validNormalized.forEach(({ timeline, normalized }) => {
      normalized.forEach((point) => {
        allPoints.push({
          timestamp: point.timestamp,
          timestampMs: point.timestampMs,
          streamId: timeline.stream_info.id,
          viewer_count: point.viewer_count,
        });
      });
    });

    // 絶対時刻でソート
    allPoints.sort((a, b) => a.timestampMs - b.timestampMs);

    // 1分単位に丸めてデータを集約
    const timeMap = new Map<number, any>();

    allPoints.forEach((point) => {
      // 1分単位に丸める
      const roundedMs = Math.floor(point.timestampMs / 60000) * 60000;
      
      if (!timeMap.has(roundedMs)) {
        timeMap.set(roundedMs, {
          timestamp: new Date(roundedMs).toISOString(),
          timestampMs: roundedMs,
        });
      }
      
      const entry = timeMap.get(roundedMs)!;
      entry[`stream_${point.streamId}`] = point.viewer_count;
    });

    // Mapを配列に変換してソート
    let chartPoints = Array.from(timeMap.values()).sort(
      (a, b) => a.timestampMs - b.timestampMs
    );

    // ポイントが多すぎる場合は間引く（パフォーマンス対策）
    const MAX_POINTS = 1000;
    if (chartPoints.length > MAX_POINTS) {
      const step = Math.ceil(chartPoints.length / MAX_POINTS);
      chartPoints = chartPoints.filter((_, index) => index % step === 0);
    }

    console.log('[ComparisonChart] Chart data prepared:', {
      totalPoints: chartPoints.length,
      streams: validNormalized.length,
      samplePoint: chartPoints[0],
      timeRange: chartPoints.length > 0 
        ? `${formatAbsoluteTime(chartPoints[0].timestamp)} ~ ${formatAbsoluteTime(chartPoints[chartPoints.length - 1].timestamp)}`
        : 'N/A',
    });

    return chartPoints;
  }, [timelines, selectedStreams]);

  // イベントマーカーを準備
  const events = useMemo(() => {
    return extractComparisonEvents(timelines);
  }, [timelines]);

  // カスタムツールチップ
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || payload.length === 0) return null;

    const data = payload[0].payload;
    
    // 実際に値が存在するpayloadのみフィルタリング
    const validPayload = payload.filter(
      (entry: any) => entry.value !== undefined && entry.value !== null
    );

    if (validPayload.length === 0) return null;

    return (
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4 min-w-[250px]">
        <p className="font-medium text-gray-900 dark:text-white mb-3">
          時刻: {formatAbsoluteTime(data.timestamp)}
        </p>
        <div className="space-y-2 text-sm">
          {validPayload.map((entry: any, index: number) => (
            <div key={entry.dataKey || entry.name || `tooltip-${index}`} className="flex items-center justify-between gap-4">
              <div className="flex items-center gap-2 flex-1 min-w-0">
                <div
                  className="w-3 h-3 rounded-full flex-shrink-0"
                  style={{ backgroundColor: entry.color }}
                />
                <span className="text-gray-700 dark:text-gray-300 truncate">
                  {entry.name?.split(' - ')[0] || '不明'}
                </span>
              </div>
              <span className="font-medium text-gray-900 dark:text-white whitespace-nowrap">
                {entry.value.toLocaleString()}人
              </span>
            </div>
          ))}
        </div>
      </div>
    );
  };

  // カスタム凡例
  const CustomLegend = () => {
    return (
      <div className="flex flex-wrap gap-3 justify-center mt-4">
        {selectedStreams.map((stream) => (
          <div key={stream.streamId} className="flex items-center gap-2">
            <div
              className="w-3 h-3 rounded-full flex-shrink-0"
              style={{ backgroundColor: stream.color }}
            />
            <span className="text-sm text-gray-700 dark:text-gray-300">
              {stream.channelName} - {truncateText(stream.streamTitle, 25)}
            </span>
          </div>
        ))}
      </div>
    );
  };

  // データポイントがあるかチェック
  const hasValidData = useMemo(() => {
    return chartData.length > 0;
  }, [chartData]);

  if (timelines.length === 0) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <p className="text-center text-gray-500 dark:text-gray-400">
          配信を選択してください
        </p>
      </div>
    );
  }

  if (!hasValidData) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <p className="text-center text-gray-500 dark:text-gray-400">
          選択された配信に表示可能なデータがありません
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* 視聴者数比較グラフ */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            視聴者数の比較（絶対時刻）
          </h3>
          <div className="text-sm text-gray-500 dark:text-gray-400">
            {chartData.length}データポイント / {timelines.length}配信
          </div>
        </div>
        
        <ResponsiveContainer width="100%" height={500}>
          <ComposedChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.1} />
            <XAxis
              dataKey="timestampMs"
              domain={['dataMin', 'dataMax']}
              type="number"
              scale="time"
              stroke="#6B7280"
              tick={{ fill: '#6B7280' }}
              tickLine={{ stroke: '#6B7280' }}
              label={{
                value: '時刻',
                position: 'insideBottom',
                offset: -5,
                fill: '#6B7280',
              }}
              tickFormatter={(value) => formatAbsoluteTime(new Date(value).toISOString())}
            />
            <YAxis
              stroke="#6B7280"
              tick={{ fill: '#6B7280' }}
              tickLine={{ stroke: '#6B7280' }}
              label={{
                value: '視聴者数',
                angle: -90,
                position: 'insideLeft',
                fill: '#6B7280',
              }}
            />
            <Tooltip content={<CustomTooltip />} />

            {/* イベントマーカー（垂直線） */}
            {events.map((event, index) => (
              <ReferenceLine
                key={`event-line-${event.timestamp}-${event.streamId}-${index}`}
                x={event.timestampMs}
                stroke={event.eventType === 'category' ? '#F59E0B' : '#3B82F6'}
                strokeDasharray="5 5"
                strokeWidth={1.5}
                opacity={0.5}
              />
            ))}

            {/* 各配信のライン */}
            {timelines.map((timeline, index) => {
              const stream = selectedStreams[index];
              const streamLabel = `${stream?.channelName || ''} - ${truncateText(
                stream?.streamTitle || '',
                25
              )}`;
              return (
                <Line
                  key={timeline.stream_info.id}
                  type="monotone"
                  dataKey={`stream_${timeline.stream_info.id}`}
                  stroke={stream?.color || '#000000'}
                  strokeWidth={3}
                  dot={false}
                  activeDot={{ r: 6, strokeWidth: 2 }}
                  name={streamLabel}
                  connectNulls={true}
                  isAnimationActive={false}
                />
              );
            })}
          </ComposedChart>
        </ResponsiveContainer>

        <CustomLegend />
      </div>

      {/* イベント一覧 */}
      {events.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
          <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white flex items-center">
            <span className="mr-2">イベント一覧</span>
            <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
              ({events.length}件)
            </span>
          </h3>
          <div className="space-y-3 max-h-80 overflow-y-auto">
            {events.map((event, index) => (
              <div
                key={`event-list-${event.timestamp}-${event.streamId}-${index}`}
                className="flex items-start gap-3 p-3 border-l-4 rounded-r"
                style={{
                  borderLeftColor: event.eventType === 'category' ? '#F59E0B' : '#3B82F6',
                  backgroundColor:
                    event.eventType === 'category'
                      ? 'rgba(245, 158, 11, 0.05)'
                      : 'rgba(59, 130, 246, 0.05)',
                }}
              >
                <div
                  className="w-3 h-3 rounded-full flex-shrink-0 mt-1"
                  style={{ backgroundColor: event.color }}
                />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      {event.streamLabel}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {formatAbsoluteTime(event.timestamp)}
                    </span>
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        event.eventType === 'category'
                          ? 'bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400'
                          : 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
                      }`}
                    >
                      {event.eventType === 'category' ? 'カテゴリ変更' : 'タイトル変更'}
                    </span>
                  </div>
                  <p className="text-sm text-gray-700 dark:text-gray-300">{event.description}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 配信情報サマリー */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
          配信情報
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {timelines.map((timeline, index) => {
            const stream = selectedStreams[index];
            const dataPoints = timeline.stats.length;
            const normalized = normalizeTimelineData(timeline, index);
            const timeRange = normalized.length > 0
              ? `${formatAbsoluteTime(normalized[0].timestamp)} ~ ${formatAbsoluteTime(normalized[normalized.length - 1].timestamp)}`
              : 'データなし';
            
            return (
              <div
                key={timeline.stream_info.id}
                className="p-4 border border-gray-200 dark:border-gray-600 rounded-lg"
              >
                <div className="flex items-center gap-2 mb-2">
                  <div
                    className="w-3 h-3 rounded-full flex-shrink-0"
                    style={{ backgroundColor: stream?.color }}
                  />
                  <h4 className="font-medium text-gray-900 dark:text-white truncate">
                    {timeline.stream_info.channel_name}
                  </h4>
                </div>
                <p className="text-sm text-gray-600 dark:text-gray-400 mb-3 truncate">
                  {timeline.stream_info.title}
                </p>
                <div className="space-y-1 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-500 dark:text-gray-400">ピーク:</span>
                    <span className="text-gray-900 dark:text-white font-medium">
                      {timeline.stream_info.peak_viewers.toLocaleString()}人
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500 dark:text-gray-400">平均:</span>
                    <span className="text-gray-900 dark:text-white font-medium">
                      {timeline.stream_info.avg_viewers.toLocaleString()}人
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500 dark:text-gray-400">配信時間:</span>
                    <span className="text-gray-900 dark:text-white font-medium">
                      {formatElapsedTime(timeline.stream_info.duration_minutes)}
                    </span>
                  </div>
                  <div className="flex justify-between pt-2 border-t border-gray-200 dark:border-gray-600">
                    <span className="text-gray-500 dark:text-gray-400">データ数:</span>
                    <span className="text-gray-900 dark:text-white font-medium">
                      {dataPoints}ポイント
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-500 dark:text-gray-400">時間範囲:</span>
                    <span className="text-gray-900 dark:text-white font-medium text-xs">
                      {timeRange}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default ComparisonChart;
