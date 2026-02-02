import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Channel, StreamInfo, StreamTimelineData } from '../../types';
import { LoadingSpinner } from '../common/LoadingSpinner';

interface StreamSelectorProps {
  onTimelineSelect: (timeline: StreamTimelineData | null) => void;
}

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
        const result = await invoke<Channel[]>('list_channels');
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
        const result = await invoke<StreamInfo[]>('get_channel_streams', {
          channelId: selectedChannelId,
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
      const timeline = await invoke<StreamTimelineData>('get_stream_timeline', {
        streamId,
      });
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
    return <LoadingSpinner />;
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
            <div className="py-8">
              <LoadingSpinner />
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
                    {stream.ended_at ? (
                      <span className="ml-2 px-2 py-1 text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
                        終了
                      </span>
                    ) : (
                      <span className="ml-2 px-2 py-1 text-xs font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded">
                        配信中
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
        <div className="mt-4 py-8">
          <LoadingSpinner />
          <p className="text-center text-gray-500 dark:text-gray-400 mt-2">
            タイムラインデータを読み込んでいます...
          </p>
        </div>
      )}
    </div>
  );
};

export default StreamSelector;
