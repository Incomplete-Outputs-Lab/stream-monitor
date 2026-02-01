import { openUrl } from '@tauri-apps/plugin-opener';
import { ChannelWithStats } from "../../types";
import { toast } from "../../utils/toast";

interface ChannelItemProps {
  channel: ChannelWithStats;
  onEdit: (channel: ChannelWithStats) => void;
  onDelete: (channelId: number) => void;
  onToggle: (channelId: number) => void;
}

export function ChannelItem({ channel, onEdit, onDelete, onToggle }: ChannelItemProps) {
  const platformNames = {
    twitch: "Twitch",
    youtube: "YouTube",
  };

  // ブラウザでチャンネルを開く
  const openChannelInBrowser = async () => {
    const url = channel.platform === 'twitch'
      ? `https://twitch.tv/${channel.channel_id}`
      : `https://youtube.com/channel/${channel.channel_id}`;
    
    try {
      await openUrl(url);
    } catch (error) {
      console.error("Failed to open URL:", error);
      toast.error("URLを開くことができませんでした: " + String(error));
    }
  };

  // ライブ状態と視聴者数を取得
  const isLive = channel.is_live;
  const viewerCount = channel.current_viewers;

  return (
    <div className="card p-6 hover:shadow-md transition-all duration-200 group">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div className="flex items-center space-x-4 flex-1 min-w-0">
          {/* プラットフォームアイコン / プロフィール画像 */}
          {channel.profile_image_url ? (
            <div className="flex-shrink-0 w-12 h-12 rounded-xl overflow-hidden shadow-lg border-2 border-gray-200 dark:border-slate-600">
              <img 
                src={channel.profile_image_url} 
                alt={channel.display_name || channel.channel_name}
                className="w-full h-full object-cover"
                onError={(e) => {
                  // 画像の読み込みに失敗した場合、フォールバックとしてデフォルトアイコンを表示
                  const target = e.target as HTMLImageElement;
                  target.style.display = 'none';
                  if (target.parentElement) {
                    target.parentElement.className = `flex-shrink-0 w-12 h-12 rounded-xl flex items-center justify-center ${
                      channel.platform === 'twitch'
                        ? 'bg-gradient-to-br from-purple-500 to-purple-600'
                        : 'bg-gradient-to-br from-red-500 to-red-600'
                    } shadow-lg`;
                    target.parentElement.innerHTML = `<span class="text-white text-xl">${channel.platform === 'twitch' ? '🎮' : '▶️'}</span>`;
                  }
                }}
              />
            </div>
          ) : (
            <div className={`flex-shrink-0 w-12 h-12 rounded-xl flex items-center justify-center ${
              channel.platform === 'twitch'
                ? 'bg-gradient-to-br from-purple-500 to-purple-600'
                : 'bg-gradient-to-br from-red-500 to-red-600'
            } shadow-lg`}>
              <span className="text-white text-xl">
                {channel.platform === 'twitch' ? '🎮' : '▶️'}
              </span>
            </div>
          )}

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
              {/* ライブステータス */}
              <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                isLive
                  ? 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-300'
              }`}>
                <div className={`w-1.5 h-1.5 rounded-full mr-1.5 ${isLive ? 'bg-red-500 animate-pulse' : 'bg-gray-400'}`}></div>
                {isLive ? 'ライブ中' : 'オフライン'}
              </span>
            </div>
            <div className="flex items-center flex-wrap gap-x-4 gap-y-1 text-sm text-gray-500 dark:text-gray-400">
              <span>ID: {channel.channel_id}</span>
              <span>•</span>
              <span>{channel.poll_interval}秒間隔</span>
              {channel.follower_count != null && (
                <>
                  <span>•</span>
                  <span className="flex items-center space-x-1 text-purple-600 dark:text-purple-400 font-semibold">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                    </svg>
                    <span>{channel.follower_count.toLocaleString()}フォロワー</span>
                  </span>
                </>
              )}
              {channel.broadcaster_type && channel.broadcaster_type !== '' && (
                <>
                  <span>•</span>
                  <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold ${
                    channel.broadcaster_type === 'partner'
                      ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300'
                      : 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
                  }`}>
                    {channel.broadcaster_type === 'partner' ? '✓ パートナー' : '✓ アフェリエイト'}
                  </span>
                </>
              )}
              {isLive && viewerCount != null && (
                <>
                  <span>•</span>
                  <span className="flex items-center space-x-1 text-red-600 dark:text-red-400 font-semibold">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                    <span>{viewerCount.toLocaleString()}人視聴中</span>
                  </span>
                </>
              )}
            </div>
            {isLive && channel.current_title && (
              <div className="mt-2 text-sm text-gray-600 dark:text-gray-300 truncate">
                <span className="font-medium">配信タイトル:</span> {channel.current_title}
              </div>
            )}
          </div>
        </div>

        {/* ステータスとアクション */}
        <div className="flex items-center space-x-4">
          {/* ブラウザで開くボタン */}
          <button
            onClick={openChannelInBrowser}
            className="p-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-all duration-200"
            title="ブラウザで開く"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </button>

          {/* 有効/無効スイッチ */}
          <label className="flex items-center cursor-pointer group">
            <input
              type="checkbox"
              checked={channel.enabled}
              onChange={() => channel.id && onToggle(channel.id)}
              className="sr-only"
            />
            <div className={`relative inline-block w-10 h-5 transition duration-300 ease-in-out rounded-full border-2 ${
              channel.enabled
                ? 'bg-green-500 border-green-500'
                : 'bg-gray-200 dark:bg-slate-600 border-gray-300 dark:border-slate-500'
            }`}>
              <span className={`absolute top-0.5 left-0.5 w-3.5 h-3.5 bg-white rounded-full transition-transform duration-300 ease-in-out shadow-sm ${
                channel.enabled ? 'translate-x-4.5' : 'translate-x-0'
              }`}></span>
            </div>
            <span className={`ml-3 text-sm font-medium transition-colors duration-200 ${
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