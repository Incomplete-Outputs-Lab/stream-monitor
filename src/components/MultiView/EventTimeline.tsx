import { MultiviewChannelStats } from "../../types";

interface EventTimelineProps {
  stats: MultiviewChannelStats[];
  maxItems?: number;
}

interface TimelineEvent {
  channelName: string;
  type: "viewer_spike" | "chat_spike" | "category_change";
  label: string;
}

export function EventTimeline({ stats, maxItems = 20 }: EventTimelineProps) {
  const events: TimelineEvent[] = [];

  for (const s of stats) {
    if (s.event_flags.viewer_spike) {
      events.push({
        channelName: s.channel_name,
        type: "viewer_spike",
        label: "視聴者スパイク",
      });
    }
    if (s.event_flags.chat_spike) {
      events.push({
        channelName: s.channel_name,
        type: "chat_spike",
        label: "チャットスパイク",
      });
    }
    if (s.event_flags.category_change) {
      events.push({
        channelName: s.channel_name,
        type: "category_change",
        label: "カテゴリ変更",
      });
    }
  }

  const displayEvents = events.slice(-maxItems).reverse();

  if (displayEvents.length === 0) {
    return (
      <div className="p-3 rounded-lg bg-gray-100 dark:bg-slate-800 h-32 flex items-center justify-center">
        <span className="text-sm text-gray-500 dark:text-gray-400">
          イベント履歴
        </span>
      </div>
    );
  }

  return (
    <div className="p-3 rounded-lg bg-gray-100 dark:bg-slate-800">
      <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
        イベント履歴
      </h4>
      <div className="space-y-1 max-h-32 overflow-y-auto">
        {displayEvents.map((ev, i) => (
          <div
            key={`${ev.channelName}-${ev.type}-${i}`}
            className="flex items-center gap-2 text-xs"
          >
            <span
              className={`w-2 h-2 rounded-full flex-shrink-0 ${
                ev.type === "viewer_spike"
                  ? "bg-blue-500"
                  : ev.type === "chat_spike"
                    ? "bg-green-500"
                    : "bg-amber-500"
              }`}
            />
            <span className="text-gray-600 dark:text-gray-400 truncate">
              {ev.channelName}
            </span>
            <span className="text-gray-500 dark:text-gray-500">{ev.label}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
