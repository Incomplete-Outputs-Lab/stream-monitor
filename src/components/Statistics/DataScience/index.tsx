import { useState } from 'react';
import type { Channel } from '../../../types';
import WordAnalysisTab from './WordAnalysisTab';
import CorrelationTab from './CorrelationTab';
import CategoryImpactTab from './CategoryImpactTab';
import ChatterScoreTab from './ChatterScoreTab';
import AnomalyDetectionTab from './AnomalyDetectionTab';

interface DataScienceProps {
  channels: Channel[];
  parentChannelId: number | null;
  parentDateRange: { start: string; end: string };
}

const DataScience = ({ channels: _channels, parentChannelId, parentDateRange }: DataScienceProps) => {
  const [selectedTab, setSelectedTab] = useState<'word' | 'correlation' | 'category' | 'score' | 'anomaly'>('word');

  // 親コンポーネントのフィルタを使用
  const selectedChannelId = parentChannelId;
  const dateRange = parentDateRange;

  const tabs = [
    { id: 'word' as const, label: 'ワード分析' },
    { id: 'correlation' as const, label: '相関分析' },
    { id: 'category' as const, label: 'カテゴリ影響' },
    { id: 'score' as const, label: 'チャッタースコア' },
    { id: 'anomaly' as const, label: '異常検知' },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white">データサイエンス分析</h2>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          高度な統計分析と機械学習的アプローチでデータから洞察を得ます
        </p>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <nav className="-mb-px flex space-x-8 overflow-x-auto" aria-label="Tabs">
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
        {selectedTab === 'word' && (
          <WordAnalysisTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'correlation' && (
          <CorrelationTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'category' && (
          <CategoryImpactTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'score' && (
          <ChatterScoreTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
        {selectedTab === 'anomaly' && (
          <AnomalyDetectionTab
            channelId={selectedChannelId}
            startTime={dateRange.start + 'T00:00:00'}
            endTime={dateRange.end + 'T23:59:59'}
          />
        )}
      </div>
    </div>
  );
};

export default DataScience;
