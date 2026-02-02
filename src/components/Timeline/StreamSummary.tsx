import React from 'react';
import { StreamInfo } from '../../types';

interface StreamSummaryProps {
  streamInfo: StreamInfo;
}

// 配信中判定ロジック: ポーリング間隔の2倍（2分）以内に収集されたデータがあれば配信中とみなす
const isStreamLive = (stream: StreamInfo): boolean => {
  if (stream.ended_at) return false;
  if (!stream.last_collected_at) return false;
  
  const lastCollected = new Date(stream.last_collected_at).getTime();
  const threshold = 2 * 60 * 1000; // 2分
  return Date.now() - lastCollected < threshold;
};

const StreamSummary: React.FC<StreamSummaryProps> = ({ streamInfo }) => {
  const formatDuration = (minutes: number): string => {
    const hours = Math.floor(minutes / 60);
    const mins = Math.floor(minutes % 60);
    return `${hours}時間${mins}分`;
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white">
          {streamInfo.title || '(タイトルなし)'}
        </h2>
        {isStreamLive(streamInfo) ? (
          <span className="px-3 py-1 text-sm font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded">
            配信中
          </span>
        ) : (
          <span className="px-3 py-1 text-sm font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
            終了
          </span>
        )}
      </div>

      <div className="text-gray-600 dark:text-gray-400 mb-6">
        <p className="mb-1">配信者: <span className="font-medium text-gray-900 dark:text-white">{streamInfo.channel_name}</span></p>
        <p className="mb-1">カテゴリ: <span className="font-medium text-gray-900 dark:text-white">{streamInfo.category || '(カテゴリなし)'}</span></p>
        <p className="mb-1">開始: <span className="font-medium text-gray-900 dark:text-white">{formatDate(streamInfo.started_at)}</span></p>
        {streamInfo.ended_at && (
          <p>終了: <span className="font-medium text-gray-900 dark:text-white">{formatDate(streamInfo.ended_at)}</span></p>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
          <p className="text-sm text-blue-600 dark:text-blue-400 mb-1">配信時間 (HB)</p>
          <p className="text-2xl font-bold text-blue-900 dark:text-blue-300">
            {formatDuration(streamInfo.duration_minutes)}
          </p>
        </div>

        <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
          <p className="text-sm text-green-600 dark:text-green-400 mb-1">Peak CCU</p>
          <p className="text-2xl font-bold text-green-900 dark:text-green-300">
            {streamInfo.peak_viewers.toLocaleString()}
          </p>
        </div>

        <div className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
          <p className="text-sm text-purple-600 dark:text-purple-400 mb-1">Avg CCU</p>
          <p className="text-2xl font-bold text-purple-900 dark:text-purple-300">
            {streamInfo.avg_viewers.toLocaleString()}
          </p>
        </div>

        <div className="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-4">
          <p className="text-sm text-orange-600 dark:text-orange-400 mb-1">ピーク/平均比</p>
          <p className="text-2xl font-bold text-orange-900 dark:text-orange-300">
            {streamInfo.avg_viewers > 0 
              ? (streamInfo.peak_viewers / streamInfo.avg_viewers).toFixed(2) 
              : 'N/A'}
          </p>
        </div>

        <div className="bg-cyan-50 dark:bg-cyan-900/20 rounded-lg p-4">
          <p className="text-sm text-cyan-600 dark:text-cyan-400 mb-1">MW (総視聴時間)</p>
          <p className="text-2xl font-bold text-cyan-900 dark:text-cyan-300">
            {streamInfo.minutes_watched.toLocaleString()} 分
          </p>
        </div>

        <div className="bg-pink-50 dark:bg-pink-900/20 rounded-lg p-4">
          <p className="text-sm text-pink-600 dark:text-pink-400 mb-1">フォロワー増加数</p>
          <p className="text-2xl font-bold text-pink-900 dark:text-pink-300">
            +{streamInfo.follower_gain.toLocaleString()}
          </p>
        </div>

        <div className="bg-indigo-50 dark:bg-indigo-900/20 rounded-lg p-4">
          <p className="text-sm text-indigo-600 dark:text-indigo-400 mb-1">合計チャット数</p>
          <p className="text-2xl font-bold text-indigo-900 dark:text-indigo-300">
            {streamInfo.total_chat_messages.toLocaleString()}
          </p>
        </div>

        <div className="bg-teal-50 dark:bg-teal-900/20 rounded-lg p-4">
          <p className="text-sm text-teal-600 dark:text-teal-400 mb-1">エンゲージメント率</p>
          <p className="text-2xl font-bold text-teal-900 dark:text-teal-300">
            {streamInfo.engagement_rate.toFixed(2)}
          </p>
        </div>
      </div>
    </div>
  );
};

export default StreamSummary;
