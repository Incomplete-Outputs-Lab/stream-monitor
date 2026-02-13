import { MultiviewChannelStats } from "../../types";

interface MultiViewTileProps {
  stats: MultiviewChannelStats;
  isMuted?: boolean;
  onMuteToggle?: () => void;
  onPopout?: () => void;
}

/** Twitch embed parent - must match the embedding page origin */
const getTwitchEmbedParent = (): string => {
  if (typeof window !== "undefined" && window.location?.hostname) {
    return window.location.hostname;
  }
  return "localhost";
};

export function MultiViewTile({
  stats,
  isMuted = true,
  onMuteToggle,
  onPopout,
}: MultiViewTileProps) {
  const embedUrl = stats.is_live
    ? `https://player.twitch.tv/?channel=${encodeURIComponent(stats.channel_name)}&parent=${getTwitchEmbedParent()}&muted=${isMuted}`
    : null;

  const hasEvent =
    stats.event_flags.viewer_spike ||
    stats.event_flags.chat_spike ||
    stats.event_flags.category_change;

  return (
    <div
      className={`relative rounded-lg overflow-hidden bg-gray-900 border-2 transition-all duration-300 ${
        hasEvent
          ? "border-amber-500 shadow-lg shadow-amber-500/20 ring-2 ring-amber-400/50"
          : "border-gray-700 dark:border-slate-600"
      }`}
    >
      {/* Header overlay */}
      <div className="absolute top-0 left-0 right-0 z-10 bg-gradient-to-b from-black/70 to-transparent p-2 flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-semibold text-white truncate">
              {stats.channel_name}
            </span>
            {stats.is_live && (
              <>
                <span className="text-xs text-gray-300">
                  {stats.viewer_count?.toLocaleString() ?? "-"} 視聴者
                </span>
                <span className="text-xs text-gray-400">
                  {stats.chat_rate_1min}/分 チャット
                </span>
              </>
            )}
          </div>
          {stats.category && (
            <p className="text-xs text-gray-400 truncate mt-0.5">
              {stats.category}
              {stats.event_flags.category_change && (
                <span className="ml-1 text-amber-400">(変更)</span>
              )}
            </p>
          )}
        </div>
        <div className="flex items-center gap-1">
          {stats.is_live && onMuteToggle && (
            <button
              onClick={onMuteToggle}
              className="p-1.5 rounded bg-black/50 hover:bg-black/70 text-white"
              title={isMuted ? "ミュート解除" : "ミュート"}
            >
              {isMuted ? (
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.617.076L4.235 12H2a1 1 0 01-1-1V9a1 1 0 011-1h2.235l4.148-3.924z"
                    clipRule="evenodd"
                  />
                </svg>
              ) : (
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M7 4a3 3 0 016 0v4a3 3 0 11-6 0V4zm4 10.93A7.001 7.001 0 0017 8a1 1 0 10-2 0A5 5 0 015 8a1 1 0 00-2 0 7.001 7.001 0 006 6.93V17H6a1 1 0 100 2h8a1 1 0 100-2h-3v-2.07z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </button>
          )}
          {onPopout && (
            <button
              onClick={onPopout}
              className="p-1.5 rounded bg-black/50 hover:bg-black/70 text-white"
              title="別ウィンドウで開く"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Event badges */}
      {(stats.event_flags.viewer_spike || stats.event_flags.chat_spike) && (
        <div className="absolute top-12 right-2 z-10 flex gap-1">
          {stats.event_flags.viewer_spike && (
            <span className="px-2 py-0.5 rounded text-xs font-medium bg-blue-600/90 text-white">
              視聴者↑
            </span>
          )}
          {stats.event_flags.chat_spike && (
            <span className="px-2 py-0.5 rounded text-xs font-medium bg-green-600/90 text-white">
              チャット↑
            </span>
          )}
        </div>
      )}

      {/* Player or offline placeholder */}
      <div className="aspect-video w-full bg-black">
        {embedUrl ? (
          <iframe
            src={embedUrl}
            title={`${stats.channel_name} - Twitch`}
            className="w-full h-full"
            allowFullScreen
            allow="autoplay; fullscreen"
          />
        ) : (
          <div className="w-full h-full flex flex-col items-center justify-center text-gray-500">
            <svg
              className="w-16 h-16 mb-2 opacity-50"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
              />
            </svg>
            <span className="text-sm">オフライン</span>
          </div>
        )}
      </div>

      {/* Mini sparkline placeholder - future BI enhancement */}
      {stats.is_live && stats.chat_rate_1min > 0 && (
        <div className="absolute bottom-0 left-0 right-0 h-6 bg-gradient-to-t from-black/60 to-transparent flex items-end justify-center gap-0.5 px-1 pb-1">
          <div
            className="w-1 bg-green-500/80 rounded-sm"
            style={{ height: `${Math.min(100, (stats.chat_rate_5s / 5) * 10)}%` }}
            title={`${stats.chat_rate_5s} msg/5s`}
          />
          <div
            className="w-1 bg-blue-500/80 rounded-sm"
            style={{ height: `${Math.min(100, (stats.chat_rate_1min / 60) * 20)}%` }}
            title={`${stats.chat_rate_1min} msg/min`}
          />
        </div>
      )}
    </div>
  );
}
