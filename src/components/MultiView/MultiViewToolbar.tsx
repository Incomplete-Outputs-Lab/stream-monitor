import { ChannelWithStats } from "../../types";

export type LayoutPreset = "2x2" | "3x2" | "1+3";

interface MultiViewToolbarProps {
  channels: ChannelWithStats[];
  selectedChannelIds: Set<number>;
  onSelectionChange: (ids: Set<number>) => void;
  layout: LayoutPreset;
  onLayoutChange: (layout: LayoutPreset) => void;
  twitchOnly?: boolean;
}

const LAYOUT_OPTIONS: { value: LayoutPreset; label: string }[] = [
  { value: "2x2", label: "2×2" },
  { value: "3x2", label: "3×2" },
  { value: "1+3", label: "メイン+3" },
];

export function MultiViewToolbar({
  channels,
  selectedChannelIds,
  onSelectionChange,
  layout,
  onLayoutChange,
  twitchOnly = true,
}: MultiViewToolbarProps) {
  const twitchChannels = twitchOnly
    ? channels.filter((c) => c.platform === "twitch")
    : channels;
  const liveChannels = twitchChannels.filter((c) => c.is_live);

  const toggleChannel = (id: number) => {
    const next = new Set(selectedChannelIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    onSelectionChange(next);
  };

  const selectAllLive = () => {
    onSelectionChange(new Set(liveChannels.map((c) => c.id!)));
  };

  const clearSelection = () => {
    onSelectionChange(new Set());
  };

  return (
    <div className="flex flex-wrap items-center gap-4 p-3 bg-gray-100 dark:bg-slate-800 rounded-lg mb-4">
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          チャンネル:
        </span>
        <div className="flex flex-wrap gap-1 max-w-md">
          {liveChannels.map((ch) => (
            <button
              key={ch.id}
              onClick={() => toggleChannel(ch.id!)}
              className={`px-2 py-1 rounded text-xs font-medium transition-colors ${
                selectedChannelIds.has(ch.id!)
                  ? "bg-blue-600 text-white"
                  : "bg-gray-200 dark:bg-slate-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-slate-500"
              }`}
            >
              {ch.channel_name}
            </button>
          ))}
        </div>
        <button
          onClick={selectAllLive}
          className="text-xs text-blue-600 dark:text-blue-400 hover:underline"
        >
          ライブ全選択
        </button>
        <button
          onClick={clearSelection}
          className="text-xs text-gray-500 dark:text-gray-400 hover:underline"
        >
          クリア
        </button>
      </div>
      <div className="flex items-center gap-2 border-l border-gray-300 dark:border-slate-600 pl-4">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          レイアウト:
        </span>
        {LAYOUT_OPTIONS.map((opt) => (
          <button
            key={opt.value}
            onClick={() => onLayoutChange(opt.value)}
            className={`px-2 py-1 rounded text-xs font-medium transition-colors ${
              layout === opt.value
                ? "bg-gray-800 dark:bg-slate-500 text-white"
                : "bg-gray-200 dark:bg-slate-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-slate-500"
            }`}
          >
            {opt.label}
          </button>
        ))}
      </div>
    </div>
  );
}
