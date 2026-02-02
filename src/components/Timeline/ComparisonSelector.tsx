import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Channel, StreamInfo, StreamTimelineData, SelectedStream } from '../../types';
import { LoadingSpinner } from '../common/LoadingSpinner';
import { getStreamColor, truncateText } from './utils';

interface ComparisonSelectorProps {
  onTimelinesSelect: (timelines: StreamTimelineData[]) => void;
  selectedStreams: SelectedStream[];
  onSelectedStreamsChange: (streams: SelectedStream[]) => void;
}

const MAX_STREAMS = 10;

// 配信中判定ロジック: ポーリング間隔の2倍（2分）以内に収集されたデータがあれば配信中とみなす
const isStreamLive = (stream: StreamInfo): boolean => {
  if (stream.ended_at) return false;
  if (!stream.last_collected_at) return false;
  
  const lastCollected = new Date(stream.last_collected_at).getTime();
  const threshold = 2 * 60 * 1000; // 2分
  return Date.now() - lastCollected < threshold;
};

// 時間重複判定関数: 2つの配信の時刻が一部でも重なっているか
const hasTimeOverlap = (streamA: StreamInfo, streamB: StreamInfo): boolean => {
  const aStart = new Date(streamA.started_at).getTime();
  const aEnd = streamA.ended_at ? new Date(streamA.ended_at).getTime() : Date.now();
  const bStart = new Date(streamB.started_at).getTime();
  const bEnd = streamB.ended_at ? new Date(streamB.ended_at).getTime() : Date.now();
  return aStart < bEnd && bStart < aEnd;
};

const ComparisonSelector: React.FC<ComparisonSelectorProps> = ({
  onTimelinesSelect,
  selectedStreams,
  onSelectedStreamsChange,
}) => {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  const [streams, setStreams] = useState<StreamInfo[]>([]);
  const [loadingChannels, setLoadingChannels] = useState(true);
  const [loadingStreams, setLoadingStreams] = useState(false);
  const [loadingTimelines, setLoadingTimelines] = useState(false);
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
  }, [selectedChannelId]);

  // 配信を選択/解除
  const handleStreamToggle = async (stream: StreamInfo) => {
    const isSelected = selectedStreams.some((s) => s.streamId === stream.id);

    if (isSelected) {
      // 選択解除
      const newSelected = selectedStreams.filter((s) => s.streamId !== stream.id);
      onSelectedStreamsChange(newSelected);
      await loadTimelines(newSelected);
    } else {
      // 選択追加
      if (selectedStreams.length >= MAX_STREAMS) {
        setError(`最大${MAX_STREAMS}件まで選択できます`);
        return;
      }

      const newStream: SelectedStream = {
        streamId: stream.id,
        channelName: stream.channel_name,
        streamTitle: stream.title,
        startedAt: stream.started_at,
        color: getStreamColor(selectedStreams.length),
      };

      const newSelected = [...selectedStreams, newStream];
      onSelectedStreamsChange(newSelected);
      await loadTimelines(newSelected);
    }
  };

  // 選択された配信のタイムラインデータを読み込み
  const loadTimelines = async (selected: SelectedStream[]) => {
    if (selected.length === 0) {
      onTimelinesSelect([]);
      return;
    }

    try {
      setLoadingTimelines(true);
      setError(null);

      // 各配信のタイムラインを個別に取得（エラーが発生しても他の配信は取得できるようにする）
      const timelineResults = await Promise.allSettled(
        selected.map((s) =>
          invoke<StreamTimelineData>('get_stream_timeline', {
            streamId: s.streamId,
          })
        )
      );

      // 成功した結果のみを抽出
      const timelines: StreamTimelineData[] = [];
      const failedCount = timelineResults.filter((result) => result.status === 'rejected').length;

      timelineResults.forEach((result) => {
        if (result.status === 'fulfilled') {
          timelines.push(result.value);
        }
      });

      if (failedCount > 0) {
        setError(`${failedCount}件のタイムラインデータ取得に失敗しました`);
      }

      onTimelinesSelect(timelines);
    } catch (err) {
      setError(`タイムラインデータの取得に失敗しました: ${err}`);
      onTimelinesSelect([]);
    } finally {
      setLoadingTimelines(false);
    }
  };

  // 選択をクリア
  const handleClearSelection = () => {
    onSelectedStreamsChange([]);
    onTimelinesSelect([]);
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleString('ja-JP', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const formatDuration = (minutes: number): string => {
    const hours = Math.floor(minutes / 60);
    const mins = Math.floor(minutes % 60);
    return `${hours}時間${mins}分`;
  };

  if (loadingChannels) {
    return <LoadingSpinner />;
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">配信選択</h2>
        <div className="text-sm text-gray-500 dark:text-gray-400">
          {selectedStreams.length} / {MAX_STREAMS} 選択中
        </div>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-400">
          {error}
        </div>
      )}

      {/* 選択済み配信のチップ表示 */}
      {selectedStreams.length > 0 && (
        <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">
              選択済み配信
            </h3>
            <button
              onClick={handleClearSelection}
              className="text-xs text-red-600 dark:text-red-400 hover:underline"
            >
              すべて解除
            </button>
          </div>
          <div className="flex flex-wrap gap-2">
            {selectedStreams.map((stream) => (
              <div
                key={stream.streamId}
                className="flex items-center gap-2 px-3 py-1.5 rounded-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800"
              >
                <div
                  className="w-3 h-3 rounded-full flex-shrink-0"
                  style={{ backgroundColor: stream.color }}
                />
                <span className="text-sm text-gray-900 dark:text-white">
                  {stream.channelName}
                </span>
                <span className="text-xs text-gray-500 dark:text-gray-400">
                  {truncateText(stream.streamTitle, 20)}
                </span>
                <button
                  onClick={() =>
                    handleStreamToggle({
                      id: stream.streamId,
                      stream_id: '',
                      channel_id: 0,
                      channel_name: stream.channelName,
                      title: stream.streamTitle,
                      category: '',
                      started_at: stream.startedAt,
                      peak_viewers: 0,
                      avg_viewers: 0,
                      duration_minutes: 0,
                      minutes_watched: 0,
                      follower_gain: 0,
                      total_chat_messages: 0,
                      engagement_rate: 0,
                    })
                  }
                  className="ml-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
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
            配信一覧（クリックで選択/解除）
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
              {streams
                .filter((stream) => {
                  // 最初の配信が選択されている場合、時刻が重複する配信のみ表示
                  if (selectedStreams.length === 0) return true;
                  const firstSelected = streams.find(s => s.id === selectedStreams[0].streamId);
                  if (!firstSelected) return true;
                  return hasTimeOverlap(firstSelected, stream);
                })
                .map((stream) => {
                const isSelected = selectedStreams.some((s) => s.streamId === stream.id);
                const selectedStream = selectedStreams.find((s) => s.streamId === stream.id);

                return (
                  <button
                    key={stream.id}
                    onClick={() => handleStreamToggle(stream)}
                    disabled={loadingTimelines}
                    className={`w-full text-left p-4 border rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                      isSelected
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      {/* チェックボックス風のインジケーター */}
                      <div className="flex-shrink-0 mt-1">
                        {isSelected ? (
                          <div
                            className="w-5 h-5 rounded flex items-center justify-center"
                            style={{ backgroundColor: selectedStream?.color }}
                          >
                            <svg
                              className="w-3 h-3 text-white"
                              fill="none"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="2"
                              viewBox="0 0 24 24"
                              stroke="currentColor"
                            >
                              <path d="M5 13l4 4L19 7" />
                            </svg>
                          </div>
                        ) : (
                          <div className="w-5 h-5 border-2 border-gray-300 dark:border-gray-600 rounded" />
                        )}
                      </div>

                      {/* 配信情報 */}
                      <div className="flex-1 min-w-0">
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
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>
      )}

      {loadingTimelines && (
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

export default ComparisonSelector;
