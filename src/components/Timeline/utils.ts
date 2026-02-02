import {
  StreamTimelineData,
  NormalizedTimelinePoint,
  ComparisonEvent,
  SelectedStream,
} from '../../types';

// カラーパレット（10配信対応）
export const STREAM_COLORS = [
  '#3B82F6', // blue
  '#10B981', // emerald
  '#F59E0B', // amber
  '#EF4444', // red
  '#8B5CF6', // violet
  '#EC4899', // pink
  '#06B6D4', // cyan
  '#84CC16', // lime
  '#F97316', // orange
  '#6366F1', // indigo
];

/**
 * タイムラインデータを絶対時刻で正規化
 * @param timeline 配信のタイムラインデータ
 * @param _streamIndex 配信のインデックス（将来の拡張用に予約）
 * @returns 正規化されたデータポイントの配列
 */
export const normalizeTimelineData = (
  timeline: StreamTimelineData,
  _streamIndex: number
): NormalizedTimelinePoint[] => {
  // statsが空の場合は空配列を返す
  if (!timeline.stats || timeline.stats.length === 0) {
    return [];
  }

  const streamLabel = `${timeline.stream_info.channel_name} - ${truncateText(
    timeline.stream_info.title,
    30
  )}`;

  return timeline.stats
    .filter((stat) => {
      // 不正なタイムスタンプを除外
      const timestampMs = new Date(stat.collected_at).getTime();
      return !isNaN(timestampMs);
    })
    .map((stat) => {
      const timestampMs = new Date(stat.collected_at).getTime();
      return {
        timestamp: stat.collected_at,
        timestampMs,
        viewer_count: stat.viewer_count || 0,
        chat_rate_1min: stat.chat_rate_1min || 0,
        streamId: timeline.stream_info.id,
        streamLabel,
      };
    });
};

/**
 * 複数配信のデータを1つの配列にマージ
 * @param timelines 複数配信のタイムラインデータ
 * @returns マージされた正規化データポイントの配列
 */
export const mergeTimelineData = (
  timelines: StreamTimelineData[]
): NormalizedTimelinePoint[] => {
  const merged: NormalizedTimelinePoint[] = [];

  timelines.forEach((timeline, index) => {
    const normalized = normalizeTimelineData(timeline, index);
    merged.push(...normalized);
  });

  // 絶対時刻でソート
  return merged.sort((a, b) => a.timestampMs - b.timestampMs);
};

/**
 * 配信インデックスから色を取得
 * @param index 配信のインデックス
 * @returns カラーコード
 */
export const getStreamColor = (index: number): string => {
  return STREAM_COLORS[index % STREAM_COLORS.length];
};

/**
 * 複数配信のイベントを統合
 * @param timelines 複数配信のタイムラインデータ
 * @returns 統合されたイベントの配列
 */
export const extractComparisonEvents = (
  timelines: StreamTimelineData[]
): ComparisonEvent[] => {
  const events: ComparisonEvent[] = [];

  timelines.forEach((timeline, index) => {
    const streamLabel = `${timeline.stream_info.channel_name}`;
    const color = getStreamColor(index);

    // カテゴリ変更イベント
    if (timeline.category_changes && Array.isArray(timeline.category_changes)) {
      timeline.category_changes.forEach((change) => {
        const timestampMs = new Date(change.timestamp).getTime();
        // タイムスタンプが不正でないかチェック
        if (!isNaN(timestampMs)) {
          events.push({
            timestamp: change.timestamp,
            timestampMs,
            eventType: 'category',
            streamId: timeline.stream_info.id,
            streamLabel,
            description: `${change.from_category || '(なし)'} → ${change.to_category}`,
            color,
          });
        }
      });
    }

    // タイトル変更イベント
    if (timeline.title_changes && Array.isArray(timeline.title_changes)) {
      timeline.title_changes.forEach((change) => {
        const timestampMs = new Date(change.timestamp).getTime();
        // タイムスタンプが不正でないかチェック
        if (!isNaN(timestampMs)) {
          events.push({
            timestamp: change.timestamp,
            timestampMs,
            eventType: 'title',
            streamId: timeline.stream_info.id,
            streamLabel,
            description: `タイトル変更: ${truncateText(change.to_title, 40)}`,
            color,
          });
        }
      });
    }
  });

  // 絶対時刻でソート
  return events.sort((a, b) => a.timestampMs - b.timestampMs);
};

/**
 * テキストを指定文字数で切り詰め
 * @param text テキスト
 * @param maxLength 最大文字数
 * @returns 切り詰められたテキスト
 */
export const truncateText = (text: string, maxLength: number): string => {
  if (!text) return '';
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '...';
};

/**
 * 選択された配信の情報を生成
 * @param timeline タイムラインデータ
 * @param index 配信のインデックス
 * @returns 選択配信情報
 */
export const createSelectedStream = (
  timeline: StreamTimelineData,
  index: number
): SelectedStream => {
  return {
    streamId: timeline.stream_info.id,
    channelName: timeline.stream_info.channel_name,
    streamTitle: timeline.stream_info.title,
    startedAt: timeline.stream_info.started_at,
    color: getStreamColor(index),
  };
};

/**
 * 経過時間を時:分形式にフォーマット
 * @param minutes 経過分数
 * @returns フォーマットされた時間文字列
 */
export const formatElapsedTime = (minutes: number): string => {
  const hours = Math.floor(minutes / 60);
  const mins = Math.floor(minutes % 60);
  
  if (hours === 0) {
    return `${mins}分`;
  }
  return `${hours}:${mins.toString().padStart(2, '0')}`;
};

/**
 * 絶対時刻を時:分形式にフォーマット
 * @param timestamp ISO8601形式のタイムスタンプ
 * @returns フォーマットされた時間文字列
 */
export const formatAbsoluteTime = (timestamp: string): string => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('ja-JP', {
    hour: '2-digit',
    minute: '2-digit',
  });
};
