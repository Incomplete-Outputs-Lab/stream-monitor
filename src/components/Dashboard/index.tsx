import { useEffect } from "react";
import { useQuery, useQueryClient, useMutation } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ChannelWithStats, DiscoveredStreamInfo } from "../../types";
import { Tooltip as CustomTooltip } from "../common/Tooltip";
import { toast } from "../../utils/toast";
import { confirm } from "../../utils/confirm";
import * as channelsApi from "../../api/channels";
import * as configApi from "../../api/config";
import * as discoveryApi from "../../api/discovery";
import * as statisticsApi from "../../api/statistics";
import { DesktopAppNotice } from "../common/DesktopAppNotice";
import { OAuthWarningBanner } from "../common/OAuthWarningBanner";
import { useAppStateStore } from "../../stores/appStateStore";

interface LiveChannelCardProps {
  channel: ChannelWithStats;
}

function LiveChannelCard({ channel }: LiveChannelCardProps) {
  const isAutoDiscovered = channel.is_auto_discovered;

  return (
    <div className="card p-4 hover:shadow-md transition-all duration-200 animate-fade-in">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center space-x-2 mb-0.5">
            <h3 className="text-base font-semibold text-gray-900 dark:text-gray-100 truncate">
              {channel.channel_name}
            </h3>
            {isAutoDiscovered && (
              <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-amber-100 dark:bg-amber-900/30 text-amber-800 dark:text-amber-300">
                自動発見
              </span>
            )}
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400 capitalize">
            {channel.platform === 'twitch' ? '🎮 Twitch' : '▶️ YouTube'}
          </p>
        </div>
        <div className="text-right ml-3">
          <div className="text-xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
            {channel.current_viewers?.toLocaleString() || 0}
          </div>
          <div className="text-xs text-gray-500 dark:text-gray-400 font-medium">視聴者</div>
        </div>
      </div>

      {channel.current_title && (
        <div className="mt-3 pt-3 border-t border-gray-200 dark:border-slate-700">
          <p className="text-xs text-gray-700 dark:text-gray-300 truncate" title={channel.current_title}>
            {channel.current_title}
          </p>
        </div>
      )}

      <div className="mt-3 flex items-center justify-between">
        <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-gradient-to-r from-green-400 to-emerald-500 text-white shadow-sm">
          <span className="w-2 h-2 bg-white rounded-full mr-2 animate-pulse"></span>
          ライブ中
        </span>
        {isAutoDiscovered && (
          <button
            onClick={async () => {
              const confirmed = await confirm({
                title: 'チャンネルの昇格',
                message: 'このチャンネルを手動登録に昇格しますか？',
                confirmText: '昇格',
                type: 'info',
              });
              
              if (confirmed) {
                try {
                  // twitch_user_idが必須
                  if (!channel.twitch_user_id) {
                    toast.error('このチャンネルにはTwitch User IDが設定されていません。');
                    return;
                  }
                  await discoveryApi.promoteDiscoveredChannel(
                    channel.twitch_user_id.toString()
                  );
                  window.location.reload();
                } catch (err) {
                  toast.error(`エラー: ${err}`);
                }
              }
            }}
            className="text-xs text-blue-600 dark:text-blue-400 hover:underline"
          >
            手動登録に昇格
          </button>
        )}
      </div>
    </div>
  );
}

interface DiscoveredStreamCardProps {
  stream: DiscoveredStreamInfo;
  onPromote: (channelId: string) => void;
  isAlreadyRegistered?: boolean;
}

function DiscoveredStreamCard({ stream, onPromote, isAlreadyRegistered = false }: DiscoveredStreamCardProps) {
  const handleOpenStream = async () => {
    try {
      await openUrl(`https://www.twitch.tv/${stream.channel_name}`);
    } catch (err) {
      console.error('Failed to open stream:', err);
    }
  };

  return (
    <div className="card p-4 hover:shadow-md transition-all duration-200">
      <div className="flex items-start gap-3">
        {/* プロフィールアイコン */}
        {stream.profile_image_url ? (
          <img 
            src={stream.profile_image_url} 
            alt={stream.display_name || stream.channel_name}
            className="w-12 h-12 rounded-full flex-shrink-0"
            onError={(e) => {
              e.currentTarget.src = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="%239CA3AF"%3E%3Cpath d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/%3E%3C/svg%3E';
            }}
          />
        ) : (
          <div className="w-12 h-12 rounded-full bg-gray-200 dark:bg-slate-700 flex items-center justify-center flex-shrink-0">
            <svg className="w-6 h-6 text-gray-400" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
            </svg>
          </div>
        )}
        
        {/* チャンネル情報 */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h4 className="text-sm font-semibold text-gray-900 dark:text-gray-100 truncate">
              {stream.display_name || stream.channel_name}
            </h4>
            {stream.broadcaster_type && stream.broadcaster_type !== '' && (
              <span className={`text-xs px-1.5 py-0.5 rounded ${
                stream.broadcaster_type === 'partner' 
                  ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300'
                  : 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300'
              }`}>
                {stream.broadcaster_type === 'partner' ? 'Partner' : 'Affiliate'}
              </span>
            )}
          </div>
          
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {stream.category || 'カテゴリ不明'}
          </p>
          
          {stream.title && (
            <p className="text-xs text-gray-600 dark:text-gray-400 mt-1 truncate" title={stream.title}>
              {stream.title}
            </p>
          )}
          
          {/* 統計情報 */}
          <div className="flex items-center gap-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
            <div className="flex items-center gap-1">
              <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
              <span className="font-semibold text-blue-600 dark:text-blue-400">
                {stream.viewer_count?.toLocaleString() || 0}
              </span>
            </div>
            
            {stream.follower_count !== null && stream.follower_count !== undefined && (
              <div className="flex items-center gap-1">
                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
                <span>{stream.follower_count.toLocaleString()}</span>
              </div>
            )}
          </div>
        </div>
      </div>
      
      <div className="mt-3 flex justify-end gap-2">
        <button
          onClick={handleOpenStream}
          className="text-xs px-3 py-1 bg-purple-500 hover:bg-purple-600 text-white rounded transition-colors flex items-center gap-1"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
          配信を開く
        </button>
        {!isAlreadyRegistered && (
          <button
            onClick={() => onPromote(stream.twitch_user_id.toString())}
            className="text-xs px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white rounded transition-colors"
          >
            手動登録に昇格
          </button>
        )}
      </div>
    </div>
  );
}

export function Dashboard() {
  const queryClient = useQueryClient();
  const backendReady = useAppStateStore((state) => state.backendReady);

  // チャンネル情報を取得し、ライブチャンネルのみをフィルタリング
  const { 
    data: allChannels, 
    isLoading: channelsLoading, 
    error: channelsError
  } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      console.log('[Dashboard] Fetching channels...');
      const result = await channelsApi.listChannels();
      console.log('[Dashboard] Fetched channels:', result?.length, 'channels');
      return result;
    },
    enabled: backendReady, // バックエンド初期化完了まで実行しない
    refetchInterval: 30000, // 30秒ごとに更新
    staleTime: 25000, // 25秒間はキャッシュを使用（refetchIntervalより短く）
    gcTime: 60000, // 1分間キャッシュを保持
    retry: 1, // リトライは1回まで
  });
  const liveChannels = allChannels?.filter(c => c.is_live) ?? [];

  // 自動発見イベントリスナー：バックエンドからのリアルタイム更新を受信
  useEffect(() => {
    console.log('[Dashboard] Setting up discovered-streams-updated event listener');
    
    const unlistenPromise = listen('discovered-streams-updated', () => {
      console.log('[Dashboard] ✅ discovered-streams-updated event received');
      // 自動発見された配信のクエリを無効化して再取得
      queryClient.invalidateQueries({ queryKey: ["discovered-streams"] });
      console.log('[Dashboard] discovered-streams query invalidated');
    });

    return () => {
      console.log('[Dashboard] Cleaning up event listeners');
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [queryClient]);

  // リアルタイムのチャットレートを取得
  const { data: realtimeChatRate } = useQuery({
    queryKey: ["realtime-chat-rate"],
    queryFn: () => statisticsApi.getRealtimeChatRate(),
    refetchInterval: 10000, // 10秒ごとに更新
  });

  // Twitch APIレート制限状態を取得
  const { data: rateLimitStatus } = useQuery({
    queryKey: ["twitch-rate-limit"],
    queryFn: configApi.getTwitchRateLimitStatus,
    refetchInterval: 5000, // 5秒ごとに更新
  });

  // 自動発見された配信を取得
  const { data: discoveredStreams, isLoading: isLoadingDiscovered } = useQuery({
    queryKey: ["discovered-streams"],
    queryFn: async () => {
      console.log('[Dashboard] Fetching discovered streams...');
      const result = await discoveryApi.getDiscoveredStreams();
      console.log('[Dashboard] Fetched discovered streams:', result?.length, 'streams');
      return result;
    },
    // enabled条件を削除：バックエンドが初期化されていなければ空配列が返る
    refetchInterval: 10000, // 10秒ごとに更新（より頻繁に）
    staleTime: 5000, // キャッシュ有効期間を短縮
    retry: 1, // リトライは1回まで
  });


  // 楽観的更新を使用したチャンネル昇格mutation
  const promoteMutation = useMutation({
    mutationFn: async (channelId: string) => {
      await discoveryApi.promoteDiscoveredChannel(channelId);
    },
    onMutate: async (channelId: string) => {
      // 既存のクエリをキャンセル
      await queryClient.cancelQueries({ queryKey: ["discovered-streams"] });
      await queryClient.cancelQueries({ queryKey: ["channels"] });

      // 現在のキャッシュを保存（ロールバック用）
      const previousDiscovered = queryClient.getQueryData<DiscoveredStreamInfo[]>(["discovered-streams"]);
      const previousChannels = queryClient.getQueryData<ChannelWithStats[]>(["channels"]);

      // 昇格するストリーム情報を取得
      const promotingStream = previousDiscovered?.find(
        s => s.twitch_user_id.toString() === channelId
      );

      // 楽観的更新: 自動発見リストから削除
      if (previousDiscovered) {
        queryClient.setQueryData<DiscoveredStreamInfo[]>(
          ["discovered-streams"],
          previousDiscovered.filter(s => s.twitch_user_id.toString() !== channelId)
        );
      }

      // 楽観的更新: ライブチャンネルリストに追加
      if (previousChannels && promotingStream) {
        const now = new Date().toISOString();
        const newChannel: ChannelWithStats = {
          id: -1, // 仮のID（サーバーからの応答で更新される）
          platform: "twitch",
          channel_id: promotingStream.channel_name,
          channel_name: promotingStream.display_name || promotingStream.channel_name,
          display_name: promotingStream.display_name || promotingStream.channel_name,
          profile_image_url: promotingStream.profile_image_url || "",
          enabled: true,
          created_at: now,
          updated_at: now,
          poll_interval: 60,
          follower_count: promotingStream.follower_count,
          broadcaster_type: promotingStream.broadcaster_type || "",
          view_count: 0,
          is_auto_discovered: false,
          discovered_at: "",
          twitch_user_id: promotingStream.twitch_user_id,
          is_live: true,
          current_viewers: promotingStream.viewer_count ?? 0,
          current_title: promotingStream.title || undefined,
        };
        queryClient.setQueryData<ChannelWithStats[]>(
          ["channels"],
          [...previousChannels, newChannel]
        );
      }

      return { previousDiscovered, previousChannels };
    },
    onError: (_err, _channelId, context) => {
      // エラー時はロールバック
      if (context?.previousDiscovered) {
        queryClient.setQueryData(["discovered-streams"], context.previousDiscovered);
      }
      if (context?.previousChannels) {
        queryClient.setQueryData(["channels"], context.previousChannels);
      }
      toast.error(`エラー: ${_err}`);
    },
    onSettled: () => {
      // 完了後にクエリを再検証して最新データを取得
      queryClient.invalidateQueries({ queryKey: ["discovered-streams"] });
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
  });

  const handlePromote = async (channelId: string) => {
    const confirmed = await confirm({
      title: 'チャンネルの昇格',
      message: 'このチャンネルを手動登録に昇格しますか？\n\n配信終了後も監視を継続します。',
      confirmText: '昇格',
      type: 'info',
    });
    
    if (confirmed) {
      promoteMutation.mutate(channelId);
    }
  };

  // チャンネルごとの統計データを整形
  // 重複を削除: channel_id + platform の組み合わせで一意にする
  const uniqueLiveChannelsMap = (liveChannels || []).reduce((acc, channel) => {
    const key = `${channel.platform}_${channel.channel_id}`;
    if (!acc.has(key)) {
      acc.set(key, channel);
    }
    return acc;
  }, new Map<string, ChannelWithStats>());

  // 視聴者数の降順でソート（楽観的更新時と再検証後で一貫した順序を保証）
  const uniqueLiveChannels = Array.from(uniqueLiveChannelsMap.values())
    .sort((a, b) => (b.current_viewers ?? 0) - (a.current_viewers ?? 0));

  const totalViewers = uniqueLiveChannels.reduce((sum, channel) => sum + (channel.current_viewers || 0), 0);

  // レート制限の色を決定
  const getRateLimitColor = (percent: number) => {
    if (percent < 50) return "bg-green-500";
    if (percent < 80) return "bg-yellow-500";
    return "bg-red-500";
  };

  const getRateLimitTextColor = (percent: number) => {
    if (percent < 50) return "text-green-600 dark:text-green-400";
    if (percent < 80) return "text-yellow-600 dark:text-yellow-400";
    return "text-red-600 dark:text-red-400";
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">ダッシュボード</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">リアルタイム統計情報</p>
        </div>
        <div className="flex items-center space-x-4">
          {/* Twitch APIレート制限インジケーター */}
          {rateLimitStatus && (
            <CustomTooltip content={
              <div className="text-xs space-y-1">
                <div className="font-semibold mb-1">Twitch API使用状況</div>
                <div>使用: {rateLimitStatus.points_used} / {rateLimitStatus.bucket_capacity} ポイント</div>
                <div>残り: {rateLimitStatus.points_remaining} ポイント</div>
                <div>リクエスト数: {rateLimitStatus.request_count}回</div>
              </div>
            }>
              <div className="flex items-center space-x-2 px-3 py-2 rounded-lg bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-700 transition-colors cursor-help">
                <div className={`w-2 h-2 rounded-full ${getRateLimitColor(rateLimitStatus.usage_percent)}`}></div>
                <div className="text-xs">
                  <div className="font-medium text-gray-600 dark:text-gray-400">API</div>
                  <div className={`font-semibold ${getRateLimitTextColor(rateLimitStatus.usage_percent)}`}>
                    {rateLimitStatus.points_used}/{rateLimitStatus.bucket_capacity}
                  </div>
                </div>
              </div>
            </CustomTooltip>
          )}
          
          <div className="text-right">
            <div className="text-sm font-medium text-gray-600 dark:text-gray-400">最終更新</div>
            <div className="text-sm text-gray-500 dark:text-gray-500">
              {new Date().toLocaleTimeString('ja-JP')}
            </div>
          </div>
        </div>
      </div>

      <OAuthWarningBanner />
      <DesktopAppNotice />

      {/* 概要統計 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">{uniqueLiveChannels.length}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">ライブ中チャンネル</p>
            </div>
          </div>
        </div>

        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in" style={{ animationDelay: '0.1s' }}>
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-green-500 to-emerald-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">{totalViewers.toLocaleString()}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">総視聴者数</p>
            </div>
          </div>
        </div>

        <div className="card p-6 hover:shadow-md transition-all duration-200 group animate-scale-in" style={{ animationDelay: '0.2s' }}>
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 bg-gradient-to-br from-purple-500 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform duration-200">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
              </div>
            </div>
            <div className="ml-4">
              <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                {(realtimeChatRate ?? 0).toLocaleString()}
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 font-medium">1分間チャット数</p>
            </div>
          </div>
        </div>
      </div>

      {/* ライブチャンネル */}
      <div className="card p-6 animate-fade-in">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">ライブ中チャンネル</h3>
            {uniqueLiveChannels.length > 0 && (
              <span className="text-xs font-medium text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-slate-700 px-2 py-1 rounded-full">
                {uniqueLiveChannels.length}件
              </span>
            )}
          </div>
          {channelsError ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <svg className="w-8 h-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>
              <p className="text-red-600 dark:text-red-400 font-medium mb-2">チャンネル情報の取得に失敗しました</p>
              <p className="text-sm text-gray-500 dark:text-gray-400">{String(channelsError)}</p>
            </div>
          ) : channelsLoading ? (
            <div className="text-center py-12">
              <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600 mx-auto"></div>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-3 font-medium">読み込み中...</p>
            </div>
          ) : uniqueLiveChannels.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {uniqueLiveChannels.map((channel) => (
                <LiveChannelCard key={channel.id ?? `${channel.platform}-${channel.channel_id}`} channel={channel} />
              ))}
            </div>
          ) : (
            <div className="text-center py-12">
              <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
                <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </div>
              <p className="text-gray-500 dark:text-gray-400 font-medium mb-1">現在ライブ中のチャンネルはありません</p>
              {allChannels && allChannels.length > 0 && (
                <p className="text-xs text-gray-400 dark:text-gray-500">
                  {allChannels.length}件のチャンネルを監視中
                </p>
              )}
            </div>
          )}
      </div>

      {/* 自動発見された配信 */}
      <div className="card p-6 animate-fade-in mt-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              自動発見された配信
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              条件に合致する上位配信を自動的に監視しています
            </p>
          </div>
          {discoveredStreams && discoveredStreams.length > 0 && (
            <span className="text-xs font-medium text-amber-600 dark:text-amber-400 bg-amber-100 dark:bg-amber-900/30 px-3 py-1 rounded-full">
              {discoveredStreams.length}件
            </span>
          )}
        </div>
        
        {isLoadingDiscovered ? (
          <div className="text-center py-12">
            <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-amber-500 mx-auto"></div>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-3 font-medium">自動発見チャンネルを取得中...</p>
          </div>
        ) : discoveredStreams && discoveredStreams.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {(() => {
              // 登録済みチャンネルのchannel_nameセットを作成
              const registeredChannelNames = new Set(
                (allChannels || []).map(ch => ch.channel_name.toLowerCase())
              );

              return discoveredStreams.map((stream) => {
                // channel_nameが登録済みかチェック
                const isAlreadyRegistered = registeredChannelNames.has(stream.channel_name.toLowerCase());

                return (
                  <DiscoveredStreamCard
                    key={`discovered-${stream.twitch_user_id}-${stream.channel_id}`}
                    stream={stream}
                    onPromote={handlePromote}
                    isAlreadyRegistered={isAlreadyRegistered}
                  />
                );
              });
            })()}
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
              <svg className="w-8 h-8 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <p className="text-gray-600 dark:text-gray-400 font-medium mb-1">自動発見された配信はありません</p>
            <p className="text-xs text-gray-400 dark:text-gray-500">
              設定から自動発見機能を有効にしてください
            </p>
          </div>
        )}
      </div>
    </div>
  );
}