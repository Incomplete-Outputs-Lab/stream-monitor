import React, { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import type { ChatMessage } from '../../../types';
import * as statisticsApi from '../../../api/statistics';

interface AnomalyChatModalProps {
  isOpen: boolean;
  onClose: () => void;
  streamId: number;
  timestamp: string;
  anomalyType: 'viewer' | 'chat';
}

const AnomalyChatModal: React.FC<AnomalyChatModalProps> = ({
  isOpen,
  onClose,
  streamId,
  timestamp,
  anomalyType,
}) => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [windowMinutes, setWindowMinutes] = useState(2);

  useEffect(() => {
    if (isOpen && streamId && timestamp) {
      fetchMessages();
    }
  }, [isOpen, streamId, timestamp, windowMinutes]);

  const fetchMessages = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await statisticsApi.getChatMessagesAroundTimestamp({
        streamId,
        timestamp,
        windowMinutes,
      });
      setMessages(data);
    } catch (err) {
      console.error('Failed to fetch chat messages:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch chat messages');
    } finally {
      setLoading(false);
    }
  };

  const formatTimestamp = (ts: string) => {
    try {
      const date = new Date(ts);
      return date.toLocaleTimeString('ja-JP', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
    } catch {
      return ts;
    }
  };

  const isAnomalyTimestamp = (msgTimestamp: string) => {
    const anomalyTime = new Date(timestamp).getTime();
    const msgTime = new Date(msgTimestamp).getTime();
    const diff = Math.abs(anomalyTime - msgTime);
    // Highlight messages within 30 seconds of anomaly
    return diff < 30000;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              {anomalyType === 'viewer' ? '視聴者数異常' : 'チャット量異常'}時のチャット
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              異常発生時刻: {formatTimestamp(timestamp)}
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-gray-500 dark:text-gray-400" />
          </button>
        </div>

        {/* Time window selector */}
        <div className="p-4 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            <label className="text-sm text-gray-700 dark:text-gray-300">時間幅:</label>
            <div className="flex gap-2">
              {[1, 2, 5].map((minutes) => (
                <button
                  key={minutes}
                  onClick={() => setWindowMinutes(minutes)}
                  className={`px-3 py-1 text-sm rounded ${
                    windowMinutes === minutes
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'
                  } transition-colors`}
                >
                  ±{minutes}分
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading && (
            <div className="text-center py-8">
              <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <p className="mt-2 text-gray-600 dark:text-gray-400">読み込み中...</p>
            </div>
          )}

          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <p className="text-red-700 dark:text-red-400">{error}</p>
            </div>
          )}

          {!loading && !error && messages.length === 0 && (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              この時間帯のチャットメッセージはありません
            </div>
          )}

          {!loading && !error && messages.length > 0 && (
            <div className="space-y-2">
              {messages.map((msg, index) => (
                <div
                  key={`${msg.id}-${index}`}
                  className={`p-3 rounded-lg ${
                    isAnomalyTimestamp(msg.timestamp)
                      ? 'bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800'
                      : 'bg-gray-50 dark:bg-gray-700/50'
                  }`}
                >
                  <div className="flex items-start gap-3">
                    <span className="text-xs text-gray-500 dark:text-gray-400 font-mono min-w-[65px]">
                      {formatTimestamp(msg.timestamp)}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-semibold text-sm text-gray-900 dark:text-white">
                          {msg.user_name}
                        </span>
                        {msg.badges && msg.badges.length > 0 && (
                          <div className="flex gap-1">
                            {msg.badges.map((badge, idx) => (
                              <span
                                key={idx}
                                className="px-1.5 py-0.5 text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded"
                              >
                                {badge}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                      <p className="text-sm text-gray-700 dark:text-gray-300 break-words">
                        {msg.message}
                      </p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
          <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
            <span>
              {messages.length > 0 ? `${messages.length}件のメッセージ` : ''}
            </span>
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg transition-colors"
            >
              閉じる
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AnomalyChatModal;
