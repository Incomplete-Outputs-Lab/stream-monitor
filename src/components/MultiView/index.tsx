import { useState, useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import * as channelsApi from "../../api/channels";
import * as multiviewApi from "../../api/multiview";
import { ChannelWithStats, MultiviewChannelStats } from "../../types";
import { DesktopAppNotice } from "../common/DesktopAppNotice";
import { MultiViewTile } from "./MultiViewTile";
import { MultiViewToolbar, type LayoutPreset } from "./MultiViewToolbar";
import { EventTimeline } from "./EventTimeline";

function getGridClass(layout: LayoutPreset, count: number): string {
  switch (layout) {
    case "2x2":
      return "grid-cols-2";
    case "3x2":
      return "grid-cols-3";
    case "1+3":
      return count <= 1 ? "grid-cols-1" : "grid-cols-2";
    default:
      return "grid-cols-2";
  }
}

function getTileClass(layout: LayoutPreset, index: number, count: number): string {
  if (layout === "1+3" && count > 1 && index === 0) {
    return "col-span-2 row-span-2";
  }
  return "";
}

export function MultiView() {
  const [selectedChannelIds, setSelectedChannelIds] = useState<Set<number>>(new Set());
  const [layout, setLayout] = useState<LayoutPreset>("2x2");
  const [mutedTiles, setMutedTiles] = useState<Set<number>>(new Set());

  const { data: channels = [], isLoading: channelsLoading } = useQuery({
    queryKey: ["channels"],
    queryFn: channelsApi.listChannels,
  });

  const channelIds = useMemo(
    () => Array.from(selectedChannelIds),
    [selectedChannelIds]
  );

  const { data: multiviewStats = [] } = useQuery({
    queryKey: ["multiview-realtime", channelIds],
    queryFn: () => multiviewApi.getMultiviewRealtimeStats(channelIds),
    enabled: channelIds.length > 0,
    refetchInterval: 5000,
  });

  const statsByChannelId = useMemo(() => {
    const map = new Map<number, MultiviewChannelStats>();
    for (const s of multiviewStats) {
      map.set(s.channel_id, s);
    }
    return map;
  }, [multiviewStats]);

  const selectedChannelsOrdered = useMemo(() => {
    return channelIds
      .map((id) => channels.find((c) => c.id === id))
      .filter((c): c is ChannelWithStats => c != null);
  }, [channelIds, channels]);

  const toggleMute = (channelId: number) => {
    setMutedTiles((prev) => {
      const next = new Set(prev);
      if (next.has(channelId)) {
        next.delete(channelId);
      } else {
        next.add(channelId);
      }
      return next;
    });
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          マルチビュー
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          複数のTwitch配信を同時に視聴・監視。視聴者数・チャットのスパイクやカテゴリ変更をリアルタイムで検知します。
        </p>
      </div>

      <DesktopAppNotice />

      <MultiViewToolbar
        channels={channels}
        selectedChannelIds={selectedChannelIds}
        onSelectionChange={setSelectedChannelIds}
        layout={layout}
        onLayoutChange={setLayout}
        twitchOnly
      />

      {channelsLoading ? (
        <div className="card p-12 text-center">
          <p className="text-gray-500 dark:text-gray-400">チャンネルを読み込み中...</p>
        </div>
      ) : channelIds.length === 0 ? (
        <div className="card p-12 text-center">
          <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
            <svg
              className="w-10 h-10 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
              />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
            チャンネルを選択してください
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            ツールバーからライブ中のチャンネルを選択すると、ここにプレイヤーが表示されます。
          </p>
        </div>
      ) : (
        <div className="flex gap-4">
          <div className="flex-1 min-w-0">
            <div
              className={`grid gap-4 ${getGridClass(layout, channelIds.length)}`}
            >
              {selectedChannelsOrdered.map((ch, index) => {
                const stats = statsByChannelId.get(ch.id!);
                const tileStats: MultiviewChannelStats = stats ?? {
                  channel_id: ch.id!,
                  channel_name: ch.channel_id,
                  stream_id: null,
                  is_live: ch.is_live,
                  viewer_count: ch.current_viewers ?? null,
                  chat_rate_1min: 0,
                  chat_rate_5s: 0,
                  category: null,
                  title: ch.current_title ?? null,
                  collected_at: null,
                  event_flags: {
                    viewer_spike: false,
                    chat_spike: false,
                    category_change: false,
                  },
                };
                return (
                  <div
                    key={ch.id}
                    className={getTileClass(layout, index, channelIds.length)}
                  >
                    <MultiViewTile
                      stats={tileStats}
                      isMuted={mutedTiles.has(ch.id!)}
                      onMuteToggle={() => toggleMute(ch.id!)}
                      onPopout={undefined}
                    />
                  </div>
                );
              })}
            </div>
          </div>
          <div className="w-56 flex-shrink-0">
            <EventTimeline stats={multiviewStats} />
          </div>
        </div>
      )}
    </div>
  );
}
