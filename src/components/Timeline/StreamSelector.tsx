import React, { useState, useEffect } from 'react';
import * as channelsApi from '../../api/channels';
import * as streamsApi from '../../api/streams';
import type { Channel, StreamInfo, StreamTimelineData } from '../../types';
import { Skeleton } from '../common/Skeleton';

interface StreamSelectorProps {
  onTimelineSelect: (timeline: StreamTimelineData | null) => void;
}

// 配信中判定ロジック: ポーリング間隔の2倍（2分）以内に収集されたデータがあれば配信中とみなす
const isStreamLive = (stream: StreamInfo): boolean => {
  if (stream.ended_at) return false;
  if (!stream.last_collected_at) return false;
  
  const lastCollected = new Date(stream.last_collected_at).getTime();
  const threshold = 2 * 60 * 1000; // 2分
  return Date.now() - lastCollected < threshold;
};

const StreamSelector: React.FC<StreamSelectorProps> = ({ onTimelineSelect }) => {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  const [streams, setStreams] = useState<StreamInfo[]>([]);
  const [loadingChannels, setLoadingChannels] = useState(true);
  const [loadingStreams, setLoadingStreams] = useState(false);
  const [loadingTimeline, setLoadingTimeline] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // チャンネル一覧を取得
  useEffect(() => {
    const fetchChannels = async () => {
      try {
        setLoadingChannels(true);
        // タイムラインでは軽量版のチャンネル一覧を使用（Twitch API にはアクセスしない）
        const result = await channelsApi.listChannelsBasic();
        setChannels(result);
      } catch (err) {
        setError(`チャンネル一覧の取得に失敗しました: ${err}`);
      } finally {
        setLoadingChannels(false);
      }
    };

    fetchChannels();
  }, []);

  // 選択したチャンネルの配信一覧を取得
  useEffect(() => {
    if (selectedChannelId === null) {
      setStreams([]);
      onTimelineSelect(null);
      return;
    }

    const fetchStreams = async () => {
      try {
        setLoadingStreams(true);
        setError(null);
        const result = await streamsApi.getChannelStreams({
          channel_id: selectedChannelId,
          limit: 50,
          offset: 0,
        });
        setStreams(result);
      } catch (err) {
        setError(`配信一覧の取得に失敗しました: ${err}`);
        setStreams([]);
      } finally {
        setLoadingStreams(false);
      }
    };

    fetchStreams();
  }, [selectedChannelId, onTimelineSelect]);

  // 配信を選択してタイムラインデータを取得
  const handleStreamSelect = async (streamId: number) => {
    try {
      setLoadingTimeline(true);
      setError(null);
      const timeline = await streamsApi.getStreamTimeline(streamId);
      onTimelineSelect(timeline);
    } catch (err) {
      setError(`タイムラインデータの取得に失敗しました: ${err}`);
      onTimelineSelect(null);
    } finally {
      setLoadingTimeline(false);
    }
  };

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

  if (loadingChannels) {
    return (
      <div className="space-y-4">
        <Skeleton variant="rectangular" height={40} className="rounded-lg" />
        <Skeleton variant="rectangular" height={200} className="rounded-lg" />
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <h2 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
        配信選択
      </h2>

      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-400">
          {error}
        </div>
      )}

      {/* チャンネル選択 */}
      <div className="mb-6">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          配信者
        </label>
        <select
          value={selectedChannelId ?? ''}
          onChange={(e) => setSelectedChannelId(e.target.value ? Number(e.target.value) : null)}
          className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
        >
          <option value="">配信者を選択してください</option>
          {channels.map((channel) => (
            <option key={channel.id} value={channel.id}>
              {channel.channel_name} ({channel.platform})
            </option>
          ))}
        </select>
      </div>

      {/* 配信一覧 */}
      {selectedChannelId && (
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            配信一覧
          </label>

          {loadingStreams ? (
            <div className="space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <Skeleton key={i} variant="rectangular" height={60} className="rounded-lg" />
              ))}
            </div>
          ) : streams.length === 0 ? (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              配信データがありません
            </div>
          ) : (
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {streams.map((stream) => (
                <button
                  key={stream.id}
                  onClick={() => handleStreamSelect(stream.id)}
                  disabled={loadingTimeline}
                  className="w-full text-left p-4 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <div className="flex justify-between items-start mb-2">
                    <div className="flex-1 min-w-0">
                      <h3 className="font-medium text-gray-900 dark:text-white truncate">
                        {stream.title || '(タイトルなし)'}
                      </h3>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        {stream.category || '(カテゴリなし)'}
                      </p>
                    </div>
                    {isStreamLive(stream) ? (
                      <span className="ml-2 px-2 py-1 text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded">
                        配信中
                      </span>
                    ) : (
                      <span className="ml-2 px-2 py-1 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
                        終了
                      </span>
                    )}
                  </div>
                  <div className="flex items-center text-sm text-gray-600 dark:text-gray-400 space-x-4">
                    <span>{formatDate(stream.started_at)}</span>
                    <span>⏱ {formatDuration(stream.duration_minutes)}</span>
                    <span>👁 ピーク: {stream.peak_viewers.toLocaleString()}人</span>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {loadingTimeline && (
        <div className="mt-4 space-y-2">
          <Skeleton variant="rectangular" height={300} className="rounded-lg" />
        </div>
      )}
    </div>
  );
};

export default StreamSelector;
