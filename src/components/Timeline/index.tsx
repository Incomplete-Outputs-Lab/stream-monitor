import React, { useState } from 'react';
import { ErrorBoundary } from '../common/ErrorBoundary';
import StreamSelector from './StreamSelector';
import TimelineWithChat from './TimelineWithChat';
import StreamSummary from './StreamSummary';
import ComparisonSelector from './ComparisonSelector';
import ComparisonChart from './ComparisonChart';
import { StreamTimelineData, SelectedStream } from '../../types';

type ViewMode = 'single' | 'comparison';

const Timeline: React.FC = () => {
  const [viewMode, setViewMode] = useState<ViewMode>('single');
  
  // 単一表示用の状態
  const [selectedTimeline, setSelectedTimeline] = useState<StreamTimelineData | null>(null);
  
  // 比較表示用の状態
  const [selectedStreams, setSelectedStreams] = useState<SelectedStream[]>([]);
  const [comparisonTimelines, setComparisonTimelines] = useState<StreamTimelineData[]>([]);

  return (
    <ErrorBoundary>
      <div className="p-6 space-y-6">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Timeline</h1>
        </div>

        {/* モード切り替えタブ */}
        <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-1 inline-flex">
          <button
            onClick={() => setViewMode('single')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              viewMode === 'single'
                ? 'bg-blue-500 text-white'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            }`}
          >
            単一表示
          </button>
          <button
            onClick={() => setViewMode('comparison')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              viewMode === 'comparison'
                ? 'bg-blue-500 text-white'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            }`}
          >
            比較表示
          </button>
        </div>

        {/* 単一表示モード */}
        {viewMode === 'single' && (
          <>
            <StreamSelector onTimelineSelect={setSelectedTimeline} />

            {selectedTimeline && (
              <div className="space-y-6">
                <StreamSummary streamInfo={selectedTimeline.stream_info} />
                <TimelineWithChat
                  timelineData={selectedTimeline}
                  streamId={selectedTimeline.stream_info.id}
                  channelId={selectedTimeline.stream_info.channel_id}
                />
              </div>
            )}

            {!selectedTimeline && (
              <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                配信者とストリームを選択して、タイムライン表示を開始してください
              </div>
            )}
          </>
        )}

        {/* 比較表示モード */}
        {viewMode === 'comparison' && (
          <>
            <ComparisonSelector
              onTimelinesSelect={setComparisonTimelines}
              selectedStreams={selectedStreams}
              onSelectedStreamsChange={setSelectedStreams}
            />

            {comparisonTimelines.length > 0 && (
              <ComparisonChart
                timelines={comparisonTimelines}
                selectedStreams={selectedStreams}
              />
            )}

            {comparisonTimelines.length === 0 && (
              <div className="text-center py-12 text-gray-500 dark:text-gray-400">
                配信を選択して比較表示を開始してください（最大10件）
              </div>
            )}
          </>
        )}
      </div>
    </ErrorBoundary>
  );
};

export default Timeline;
