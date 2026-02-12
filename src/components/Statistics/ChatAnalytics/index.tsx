import { useState } from 'react';
import type { Channel } from '../../../types';
import EngagementTab from './EngagementTab';
import UserSegmentTab from './UserSegmentTab';
import ChatterBehaviorTab from './ChatterBehaviorTab';
import TimePatternTab from './TimePatternTab';

interface ChatAnalyticsProps {
  channels: Channel[];
  parentChannelId: number | null;
  parentDateRange: { start: string; end: string };
}

const ChatAnalytics = ({ channels: _channels, parentChannelId, parentDateRange }: ChatAnalyticsProps) => {
  const [selectedTab, setSelectedTab] = useState<'engagement' | 'segment' | 'behavior' | 'time'>('engagement');

  // 親コンポーネントのフィルタを使用
  const selectedChannelId = parentChannelId;
  const dateRange = parentDateRange;

  const tabs = [
    { id: 'engagement' as const, label: 'エンゲージメント' },
    { id: 'segment' as const, label: 'ユーザーセグメント' },
    { id: 'behavior' as const, label: 'チャッター行動' },
    { id: 'time' as const, label: '時間パターン' },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">チャット分析</h2>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          チャットデータから視聴者エンゲージメントやコミュニティ特性を分析します
        </p>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <nav className="-mb-px flex space-x-8" aria-label="Tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setSelectedTab(tab.id)}
              className={`
                whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm
                ${
                  selectedTab === tab.id
                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                }
              `}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      <div className="mt-6">
        {selectedTab === 'engagement' && (
          <EngagementTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'segment' && (
          <UserSegmentTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'behavior' && (
          <ChatterBehaviorTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'time' && (
          <TimePatternTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
      </div>
    </div>
  );
};

export default ChatAnalytics;
