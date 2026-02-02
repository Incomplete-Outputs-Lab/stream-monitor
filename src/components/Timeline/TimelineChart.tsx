import React, { useMemo } from 'react';
import { StreamTimelineData } from '../../types';
import {
  ComposedChart,
  Line,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import EventMarkers from './EventMarkers';

interface TimelineChartProps {
  timelineData: StreamTimelineData;
}

const TimelineChart: React.FC<TimelineChartProps> = ({ timelineData }) => {
  const chartData = useMemo(() => {
    return timelineData.stats.map((stat) => {
      const date = new Date(stat.collected_at);
      const timeStr = date.toLocaleTimeString('ja-JP', {
        hour: '2-digit',
        minute: '2-digit',
      });

      return {
        time: timeStr,
        timestamp: stat.collected_at,
        viewers: stat.viewer_count || 0,
        chatRate: stat.chat_rate_1min,
        followers: stat.follower_count || 0,
        category: stat.category,
        title: stat.title,
      };
    });
  }, [timelineData.stats]);

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4">
          <p className="font-medium text-gray-900 dark:text-white mb-2">{data.time}</p>
          <div className="space-y-1 text-sm">
            <p className="text-blue-600 dark:text-blue-400">
              視聴者数: <span className="font-medium">{data.viewers.toLocaleString()}</span>
            </p>
            <p className="text-green-600 dark:text-green-400">
              チャットレート: <span className="font-medium">{data.chatRate}</span>
            </p>
            <p className="text-purple-600 dark:text-purple-400">
              フォロワー: <span className="font-medium">{data.followers.toLocaleString()}</span>
            </p>
            {data.category && (
              <p className="text-orange-600 dark:text-orange-400">
                カテゴリ: <span className="font-medium">{data.category}</span>
              </p>
            )}
          </div>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="space-y-6">
      {/* イベントマーカー */}
      <EventMarkers
        categoryChanges={timelineData.category_changes}
        titleChanges={timelineData.title_changes}
      />

      {/* 視聴者数とチャットレートのグラフ */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
          視聴者数とチャットレート
        </h3>
        <ResponsiveContainer width="100%" height={400}>
          <ComposedChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.1} />
            <XAxis
              dataKey="time"
              stroke="#6B7280"
              tick={{ fill: '#6B7280' }}
              tickLine={{ stroke: '#6B7280' }}
            />
            <YAxis
              yAxisId="left"
              stroke="#3B82F6"
              tick={{ fill: '#6B7280' }}
              tickLine={{ stroke: '#6B7280' }}
              label={{ value: '視聴者数', angle: -90, position: 'insideLeft', fill: '#6B7280' }}
            />
            <YAxis
              yAxisId="right"
              orientation="right"
              stroke="#10B981"
              tick={{ fill: '#6B7280' }}
              tickLine={{ stroke: '#6B7280' }}
              label={{ value: 'チャットレート', angle: 90, position: 'insideRight', fill: '#6B7280' }}
            />
            <Tooltip content={<CustomTooltip />} />
            <Legend />
            <Area
              yAxisId="left"
              type="monotone"
              dataKey="viewers"
              fill="#3B82F6"
              stroke="#3B82F6"
              fillOpacity={0.2}
              name="視聴者数"
            />
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="chatRate"
              stroke="#10B981"
              strokeWidth={2}
              dot={false}
              name="チャットレート"
            />
          </ComposedChart>
        </ResponsiveContainer>
      </div>

      {/* フォロワー数の推移 */}
      {timelineData.stats.some((s) => s.follower_count !== null && s.follower_count !== undefined) && (
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
          <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
            フォロワー数の推移
          </h3>
          <ResponsiveContainer width="100%" height={300}>
            <ComposedChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.1} />
              <XAxis
                dataKey="time"
                stroke="#6B7280"
                tick={{ fill: '#6B7280' }}
                tickLine={{ stroke: '#6B7280' }}
              />
              <YAxis
                stroke="#8B5CF6"
                tick={{ fill: '#6B7280' }}
                tickLine={{ stroke: '#6B7280' }}
                label={{ value: 'フォロワー数', angle: -90, position: 'insideLeft', fill: '#6B7280' }}
              />
              <Tooltip content={<CustomTooltip />} />
              <Legend />
              <Line
                type="monotone"
                dataKey="followers"
                stroke="#8B5CF6"
                strokeWidth={2}
                dot={false}
                name="フォロワー数"
              />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
};

export default TimelineChart;
