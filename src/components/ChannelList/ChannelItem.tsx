import { Channel } from "../../types";

interface ChannelItemProps {
  channel: Channel;
  onEdit: (channel: Channel) => void;
  onDelete: (channelId: number) => void;
  onToggle: (channelId: number) => void;
}

export function ChannelItem({ channel, onEdit, onDelete, onToggle }: ChannelItemProps) {
  const platformNames = {
    twitch: "Twitch",
    youtube: "YouTube",
  };

  return (
    <div className="card p-6 hover:shadow-md transition-all duration-200 group">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div className="flex items-center space-x-4 flex-1 min-w-0">
          {/* プラットフォームアイコン */}
          <div className={`flex-shrink-0 w-12 h-12 rounded-xl flex items-center justify-center ${
            channel.platform === 'twitch' 
              ? 'bg-gradient-to-br from-purple-500 to-purple-600' 
              : 'bg-gradient-to-br from-red-500 to-red-600'
          } shadow-lg`}>
            <span className="text-white text-xl">
              {channel.platform === 'twitch' ? '🎮' : '▶️'}
            </span>
          </div>

          {/* チャンネル情報 */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center space-x-2 mb-1">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 truncate">
                {channel.display_name || channel.channel_name}
              </h3>
              <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-semibold ${
                channel.platform === 'twitch'
                  ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300'
                  : 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
              }`}>
                {platformNames[channel.platform as keyof typeof platformNames]}
              </span>
            </div>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              ID: {channel.channel_id} • {channel.poll_interval}秒間隔
            </p>
          </div>
        </div>

        {/* ステータスとアクション */}
        <div className="flex items-center space-x-4">
          {/* 有効/無効スイッチ */}
          <label className="flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={channel.enabled}
              onChange={() => channel.id && onToggle(channel.id)}
              className="sr-only"
            />
            <div className={`relative inline-block w-11 h-6 transition duration-200 ease-in-out rounded-full shadow-inner ${
              channel.enabled 
                ? 'bg-gradient-to-r from-green-400 to-emerald-500' 
                : 'bg-gray-300 dark:bg-slate-600'
            }`}>
              <span className={`absolute left-0.5 top-0.5 m-0.5 w-5 h-5 bg-white rounded-full transition-transform duration-200 ease-in-out shadow-md ${
                channel.enabled ? 'translate-x-5' : 'translate-x-0'
              }`}></span>
            </div>
            <span className={`ml-2 text-sm font-medium ${
              channel.enabled 
                ? 'text-green-600 dark:text-green-400' 
                : 'text-gray-500 dark:text-gray-400'
            }`}>
              {channel.enabled ? '有効' : '無効'}
            </span>
          </label>

          {/* アクションボタン */}
          <div className="flex space-x-2">
            <button
              onClick={() => onEdit(channel)}
              className="px-4 py-2 text-sm font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded-lg transition-all duration-200 hover:scale-105 active:scale-95"
            >
              編集
            </button>
            <button
              onClick={() => channel.id && onDelete(channel.id)}
              className="px-4 py-2 text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all duration-200 hover:scale-105 active:scale-95"
            >
              削除
            </button>
          </div>
        </div>
      </div>

      {/* 追加情報 */}
      <div className="mt-4 pt-4 border-t border-gray-200 dark:border-slate-700 flex items-center text-xs text-gray-500 dark:text-gray-400 space-x-4">
        <span className="flex items-center space-x-1">
          <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <span>作成: {channel.created_at ? new Date(channel.created_at).toLocaleDateString('ja-JP') : '不明'}</span>
        </span>
        {channel.updated_at && channel.updated_at !== channel.created_at && (
          <span className="flex items-center space-x-1">
            <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <span>更新: {new Date(channel.updated_at).toLocaleDateString('ja-JP')}</span>
          </span>
        )}
      </div>
    </div>
  );
}