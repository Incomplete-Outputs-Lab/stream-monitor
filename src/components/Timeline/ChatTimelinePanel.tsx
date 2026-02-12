import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import * as statisticsApi from '../../api/statistics';
import { ChatMessagesSkeleton } from '../common/Skeleton';

interface ChatTimelinePanelProps {
  streamId: number;
  channelId?: number;
}

const ChatTimelinePanel: React.FC<ChatTimelinePanelProps> = ({
  streamId,
  channelId,
}) => {
  const [currentPage, setCurrentPage] = useState(1);
  const messagesPerPage = 100;

  // Fetch chat messages for the entire stream
  const { data: messages, isLoading } = useQuery({
    queryKey: ['timelineChatMessages', streamId, channelId, currentPage],
    queryFn: async () => {
      console.log('[ChatTimelinePanel] Fetching messages with params:', {
        streamId,
        channelId,
        limit: messagesPerPage,
        offset: (currentPage - 1) * messagesPerPage,
      });
      const result = await statisticsApi.getChatMessages({
        streamId,
        channelId,
        limit: messagesPerPage,
        offset: (currentPage - 1) * messagesPerPage,
      });
      console.log('[ChatTimelinePanel] Received messages:', result.length);
      return result;
    },
  });

  const getBadgeColor = (badge: string): string => {
    if (badge.includes('broadcaster'))
      return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
    if (badge.includes('moderator'))
      return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    if (badge.includes('vip'))
      return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
    if (badge.includes('subscriber'))
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
    return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300';
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  return (
    <div className="h-full flex flex-col bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
          チャットメッセージ
        </h3>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          配信全体のチャットログ（時系列順）
        </p>
      </div>

      {/* Chat messages */}
      <div className="flex-1 overflow-y-auto p-4">
        {isLoading ? (
          <ChatMessagesSkeleton count={10} />
        ) : messages && messages.length > 0 ? (
          <div className="space-y-3">
            {messages.map((message, index) => (
              <div
                key={message.id || index}
                className="flex flex-col space-y-1 p-2 rounded hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
              >
                {/* Time and User */}
                <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                  <span className="font-mono">{formatTime(message.timestamp)}</span>
                  <span className="font-medium text-gray-900 dark:text-white">
                    {message.display_name || message.user_name}
                  </span>
                  {/* Badges */}
                  {message.badges && message.badges.length > 0 && (
                    <div className="flex gap-1">
                      {message.badges.map((badge, idx) => (
                        <span
                          key={idx}
                          className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium ${getBadgeColor(
                            badge
                          )}`}
                        >
                          {badge}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
                {/* Message */}
                <div className="text-sm text-gray-900 dark:text-white break-words">
                  {message.message}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center h-full">
            <p className="text-gray-500 dark:text-gray-400 text-center">
              この時間帯にチャットメッセージがありません
            </p>
          </div>
        )}
      </div>

      {/* Pagination */}
      {messages && messages.length > 0 && (
        <div className="p-3 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="px-3 py-1.5 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            前へ
          </button>
          <span className="text-xs text-gray-600 dark:text-gray-400">
            ページ {currentPage} ({messages.length} 件)
          </span>
          <button
            onClick={() => setCurrentPage((p) => p + 1)}
            disabled={!messages || messages.length < messagesPerPage}
            className="px-3 py-1.5 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            次へ
          </button>
        </div>
      )}
    </div>
  );
};

export default ChatTimelinePanel;
