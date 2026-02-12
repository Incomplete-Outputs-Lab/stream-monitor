import React, { useState, useEffect } from 'react';
import * as channelsApi from '../../api/channels';
import * as streamsApi from '../../api/streams';
import type { Channel, StreamInfo, StreamTimelineData, SelectedStream } from '../../types';
import { Skeleton } from '../common/Skeleton';
import { getStreamColor, truncateText } from './utils';

/** 1本目の選択方法 */
type FirstStreamSelectMode = 'date' | 'channel';

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

// 類似度スコア計算
interface SimilarityScore {
  stream: StreamInfo;
  score: number;
  matchedCriteria: string[];
}

const calculateSimilarity = (baseStream: StreamInfo, targetStream: StreamInfo): SimilarityScore => {
  let score = 0;
  const matchedCriteria: string[] = [];

  // カテゴリ一致 (+40点)
  if (baseStream.category && targetStream.category && baseStream.category === targetStream.category) {
    score += 40;
    matchedCriteria.push('同じカテゴリ');
  }

  // 日付が同じ (±1日以内) (+30点)
  const baseDate = new Date(baseStream.started_at);
  const targetDate = new Date(targetStream.started_at);
  const daysDiff = Math.abs((baseDate.getTime() - targetDate.getTime()) / (1000 * 60 * 60 * 24));
  if (daysDiff <= 1) {
    score += 30;
    if (daysDiff < 0.1) {
      matchedCriteria.push('同じ日付');
    } else {
      matchedCriteria.push('近い日付');
    }
  }

  // 時間帯の重複 (+20点)
  if (hasTimeOverlap(baseStream, targetStream)) {
    score += 20;
    matchedCriteria.push('時間帯重複');
  }

  // 同じプラットフォーム (+10点)
  if (baseStream.platform === targetStream.platform) {
    score += 10;
    matchedCriteria.push('同じプラットフォーム');
  }

  return {
    stream: targetStream,
    score,
    matchedCriteria,
  };
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
  const [suggestedStreams, setSuggestedStreams] = useState<SimilarityScore[]>([]);

  // 1本目の選択方法（日付 or チャンネル）
  const [firstStreamMode, setFirstStreamMode] = useState<FirstStreamSelectMode>('channel');
  // 日付から選ぶ用
  const [dateFrom, setDateFrom] = useState<string>(() => {
    const d = new Date();
    return d.toISOString().slice(0, 10);
  });
  const [dateTo, setDateTo] = useState<string>(() => {
    const d = new Date();
    return d.toISOString().slice(0, 10);
  });
  const [streamsByDate, setStreamsByDate] = useState<StreamInfo[]>([]);
  const [loadingStreamsByDate, setLoadingStreamsByDate] = useState(false);

  // チャンネル一覧を取得
  useEffect(() => {
    const fetchChannels = async () => {
      try {
        setLoadingChannels(true);
        const result = await channelsApi.listChannels();
        setChannels(result);
      } catch (err) {
        setError(`チャンネル一覧の取得に失敗しました: ${err}`);
      } finally {
        setLoadingChannels(false);
      }
    };

    fetchChannels();
  }, []);

  // 選択したチャンネルの配信一覧を取得（チャンネルから選ぶ用）
  useEffect(() => {
    if (selectedChannelId === null) {
      setStreams([]);
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
  }, [selectedChannelId]);

  // 配信を選択/解除
  const handleStreamToggle = async (stream: StreamInfo) => {
    const isSelected = selectedStreams.some((s) => s.streamId === stream.id);

    if (isSelected) {
      // 選択解除
      const newSelected = selectedStreams.filter((s) => s.streamId !== stream.id);
      onSelectedStreamsChange(newSelected);
      await loadTimelines(newSelected);

      // 最初の配信を解除した場合、サジェストをクリア
      if (selectedStreams.length === 1) {
        setSuggestedStreams([]);
      }
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

      // 最初の配信を選択した場合、APIで類似配信を取得してサジェスト表示
      if (selectedStreams.length === 0) {
        try {
          const apiSuggestions = await streamsApi.getSuggestedStreamsForComparison({
            base_stream_id: stream.id,
            limit: 50,
          });
          const selectedStreamIds = newSelected.map((s) => s.streamId);
          const withScores = apiSuggestions
            .filter((s) => !selectedStreamIds.includes(s.id))
            .map((s) => calculateSimilarity(stream, s))
            .filter((r) => r.score >= 40)
            .sort((a, b) => b.score - a.score)
            .slice(0, 10);
          setSuggestedStreams(withScores);
        } catch {
          setSuggestedStreams([]);
        }
      }
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
        selected.map((s) => streamsApi.getStreamTimeline(s.streamId))
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
    setSuggestedStreams([]);
  };

  // 日付範囲で配信一覧を取得
  const handleLoadStreamsByDate = async () => {
    try {
      setLoadingStreamsByDate(true);
      setError(null);
      const result = await streamsApi.getStreamsByDateRange({
        date_from: dateFrom,
        date_to: dateTo,
        limit: 100,
        offset: 0,
      });
      setStreamsByDate(result);
    } catch (err) {
      setError(`配信一覧の取得に失敗しました: ${err}`);
      setStreamsByDate([]);
    } finally {
      setLoadingStreamsByDate(false);
    }
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
    return (
      <div className="space-y-4">
        <Skeleton variant="rectangular" height={40} className="rounded-lg" />
        <Skeleton variant="rectangular" height={200} className="rounded-lg" />
      </div>
    );
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

      {/* 1本目を選ぶ（未選択時のみ強調表示） */}
      {selectedStreams.length === 0 && (
        <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700">
          <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
            1本目を選ぶ
          </h3>
          <div className="flex gap-2 mb-4">
            <button
              type="button"
              onClick={() => setFirstStreamMode('date')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                firstStreamMode === 'date'
                  ? 'bg-blue-500 text-white'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
            >
              日付から選ぶ
            </button>
            <button
              type="button"
              onClick={() => setFirstStreamMode('channel')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                firstStreamMode === 'channel'
                  ? 'bg-blue-500 text-white'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
            >
              チャンネルから選ぶ
            </button>
          </div>

          {firstStreamMode === 'date' && (
            <div className="space-y-3">
              <div className="flex flex-wrap items-center gap-2">
                <label className="text-sm text-gray-600 dark:text-gray-400">
                  開始日
                </label>
                <input
                  type="date"
                  value={dateFrom}
                  onChange={(e) => setDateFrom(e.target.value)}
                  className="px-2 py-1.5 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                />
                <label className="text-sm text-gray-600 dark:text-gray-400">
                  終了日
                </label>
                <input
                  type="date"
                  value={dateTo}
                  onChange={(e) => setDateTo(e.target.value)}
                  className="px-2 py-1.5 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
                />
                <button
                  type="button"
                  onClick={handleLoadStreamsByDate}
                  disabled={loadingStreamsByDate}
                  className="px-3 py-1.5 bg-blue-500 text-white rounded text-sm font-medium hover:bg-blue-600 disabled:opacity-50"
                >
                  {loadingStreamsByDate ? '取得中…' : 'この期間の配信を表示'}
                </button>
              </div>
              {loadingStreamsByDate ? (
                <div className="space-y-2">
                  {Array.from({ length: 3 }).map((_, i) => (
                    <Skeleton key={i} variant="rectangular" height={56} className="rounded-lg" />
                  ))}
                </div>
              ) : streamsByDate.length > 0 ? (
                <div className="space-y-2 max-h-80 overflow-y-auto">
                  {streamsByDate.map((stream) => (
                    <button
                      key={stream.id}
                      type="button"
                      onClick={() => handleStreamToggle(stream)}
                      disabled={loadingTimelines}
                      className="w-full text-left p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
                    >
                      <div className="flex justify-between items-start">
                        <div className="min-w-0 flex-1">
                          <p className="font-medium text-gray-900 dark:text-white truncate">
                            {stream.channel_name} — {truncateText(stream.title || '(タイトルなし)', 40)}
                          </p>
                          <p className="text-sm text-gray-500 dark:text-gray-400">
                            {stream.category || '(カテゴリなし)'} · {formatDate(stream.started_at)}
                          </p>
                        </div>
                        <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">
                          👁 ピーク: {stream.peak_viewers.toLocaleString()}
                        </span>
                      </div>
                    </button>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  日付を選んで「この期間の配信を表示」を押すと、その期間の配信一覧が表示されます。
                </p>
              )}
            </div>
          )}

          {firstStreamMode === 'channel' && (
            <div className="space-y-3">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                配信者
              </label>
              <select
                value={selectedChannelId ?? ''}
                onChange={(e) => setSelectedChannelId(e.target.value ? Number(e.target.value) : null)}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
              >
                <option value="">配信者を選択してください</option>
                {channels.map((ch) => (
                  <option key={ch.id} value={ch.id}>
                    {ch.channel_name} ({ch.platform ?? 'twitch'})
                  </option>
                ))}
              </select>
              {selectedChannelId && (
                <>
                  {loadingStreams ? (
                    <div className="space-y-2">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <Skeleton key={i} variant="rectangular" height={56} className="rounded-lg" />
                      ))}
                    </div>
                  ) : streams.length === 0 ? (
                    <p className="text-sm text-gray-500 dark:text-gray-400">配信データがありません</p>
                  ) : (
                    <div className="space-y-2 max-h-80 overflow-y-auto">
                      {streams.map((stream) => (
                        <button
                          key={stream.id}
                          type="button"
                          onClick={() => handleStreamToggle(stream)}
                          disabled={loadingTimelines}
                          className="w-full text-left p-3 border border-gray-200 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
                        >
                          <div className="flex justify-between items-start">
                            <div className="min-w-0 flex-1">
                              <p className="font-medium text-gray-900 dark:text-white truncate">
                                {stream.title || '(タイトルなし)'}
                              </p>
                              <p className="text-sm text-gray-500 dark:text-gray-400">
                                {stream.category || '(カテゴリなし)'} · {formatDate(stream.started_at)}
                              </p>
                            </div>
                            <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">
                              👁 ピーク: {stream.peak_viewers.toLocaleString()}
                            </span>
                          </div>
                        </button>
                      ))}
                    </div>
                  )}
                </>
              )}
            </div>
          )}
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
                      ended_at: '',
                      peak_viewers: 0,
                      avg_viewers: 0,
                      duration_minutes: 0,
                      minutes_watched: 0,
                      follower_gain: 0,
                      total_chat_messages: 0,
                      engagement_rate: 0,
                      last_collected_at: '',
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

      {/* おすすめの配信 */}
      {suggestedStreams.length > 0 && (
        <div className="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
          <h3 className="text-sm font-semibold text-blue-800 dark:text-blue-300 mb-3 flex items-center">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            おすすめの配信（類似度が高い順）
          </h3>
          <p className="text-xs text-blue-600 dark:text-blue-400 mb-3">
            選択した配信と似た特徴を持つ配信です。クリックで追加できます。
          </p>
          <div className="space-y-2 max-h-80 overflow-y-auto">
            {suggestedStreams.map((suggestion) => (
              <button
                key={suggestion.stream.id}
                onClick={() => handleStreamToggle(suggestion.stream)}
                disabled={selectedStreams.length >= MAX_STREAMS}
                className="w-full text-left p-3 bg-white dark:bg-gray-800 hover:bg-blue-100 dark:hover:bg-blue-900/30 border border-blue-200 dark:border-blue-700 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-medium text-gray-900 dark:text-white">
                      {suggestion.stream.channel_name}
                    </span>
                    <span className="px-2 py-0.5 bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 rounded text-xs font-semibold">
                      {suggestion.score}点
                    </span>
                  </div>
                  <span className="text-xs text-gray-500 dark:text-gray-400">
                    {formatDate(suggestion.stream.started_at)}
                  </span>
                </div>
                <div className="text-xs text-gray-600 dark:text-gray-300 mb-2">
                  {truncateText(suggestion.stream.title || '(タイトルなし)', 60)}
                </div>
                <div className="flex flex-wrap gap-1">
                  {suggestion.matchedCriteria.map((criteria, idx) => (
                    <span
                      key={idx}
                      className="inline-flex items-center px-2 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded text-xs"
                    >
                      ✓ {criteria}
                    </span>
                  ))}
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* 2本目以降を追加するときのチャンネル選択（1本目選択後のみ表示） */}
      {selectedStreams.length >= 1 && (
        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            配信者を選んで追加
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
      )}

      {/* 配信一覧（チャンネル選択時、2本目以降の追加用） */}
      {selectedStreams.length >= 1 && selectedChannelId && (
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            配信一覧（クリックで選択/解除）
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
        <div className="mt-4 space-y-2">
          <Skeleton variant="rectangular" height={300} className="rounded-lg" />
        </div>
      )}
    </div>
  );
};

export default ComparisonSelector;
