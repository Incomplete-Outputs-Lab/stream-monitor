import { useState } from 'react';
import type { Channel } from '../../../types';
import { DateRangePicker } from '../DateRangePicker';
import EngagementTab from './EngagementTab';
import UserSegmentTab from './UserSegmentTab';
import ChatterBehaviorTab from './ChatterBehaviorTab';
import TimePatternTab from './TimePatternTab';

interface ChatAnalyticsProps {
  channels: Channel[];
}

const ChatAnalytics = ({ channels }: ChatAnalyticsProps) => {
  const [selectedTab, setSelectedTab] = useState<'engagement' | 'segment' | 'behavior' | 'time'>('engagement');
  const [selectedChannelId, setSelectedChannelId] = useState<number | null>(null);
  const [dateRange, setDateRange] = useState<{ start: string; end: string }>(() => {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - 7);
    return {
      start: start.toISOString().split('T')[0],
      end: end.toISOString().split('T')[0],
    };
  });

  const handleDateRangeChange = (start: string, end: string) => {
    setDateRange({ start, end });
  };

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

      {/* Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow space-y-4">
        {/* Channel Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            チャンネル
          </label>
          <select
            value={selectedChannelId || ''}
            onChange={(e) => setSelectedChannelId(e.target.value ? Number(e.target.value) : null)}
            className="w-full px-3 py-2 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-gray-900 dark:text-white"
          >
            <option value="">全てのチャンネル</option>
            {channels.map((ch) => (
              <option key={ch.id} value={ch.id}>
                {ch.channel_name}
              </option>
            ))}
          </select>
        </div>

        {/* Date Range */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            期間
          </label>
          <DateRangePicker
            startDate={dateRange.start}
            endDate={dateRange.end}
            onChange={handleDateRangeChange}
          />
        </div>
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
