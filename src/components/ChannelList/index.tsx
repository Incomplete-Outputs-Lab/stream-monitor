import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Channel, ChannelWithStats } from "../../types";
import { ChannelForm } from "./ChannelForm";
import { ChannelEditForm } from "./ChannelEditForm";
import { ChannelItem } from "./ChannelItem";

export function ChannelList() {
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingChannel, setEditingChannel] = useState<Channel | null>(null);
  const [filter, setFilter] = useState<'all' | 'twitch' | 'youtube'>('all');

  const queryClient = useQueryClient();

  // チャンネル取得
  const { data: channels = [], isLoading } = useQuery({
    queryKey: ["channels"],
    queryFn: () => invoke<Channel[]>("list_channels"),
    refetchInterval: 30000, // 30秒ごとに更新
  });

  // チャンネル削除ミューテーション
  const deleteMutation = useMutation({
    mutationFn: async (channelId: number) => {
      await invoke("remove_channel", { id: channelId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
    },
  });

  // チャンネル有効/無効切り替えミューテーション
  const toggleMutation = useMutation({
    mutationFn: async (channelId: number) => {
      await invoke("toggle_channel", { id: channelId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["channels"] });
      queryClient.invalidateQueries({ queryKey: ["live-channels"] });
    },
  });

  const handleDelete = async (channelId: number) => {
    if (window.confirm("このチャンネルを削除しますか？")) {
      try {
        await deleteMutation.mutateAsync(channelId);
      } catch (error) {
        alert("チャンネルの削除に失敗しました: " + String(error));
      }
    }
  };

  const handleToggle = async (channelId: number) => {
    try {
      await toggleMutation.mutateAsync(channelId);
    } catch (error) {
      alert("チャンネルの切り替えに失敗しました: " + String(error));
    }
  };

  // フィルタリングされたチャンネル
  const filteredChannels = channels.filter(channel => {
    if (filter === 'all') return true;
    return channel.platform === filter;
  });

  const handleAddChannel = () => {
    setShowAddForm(true);
    setEditingChannel(null);
  };

  const handleEditChannel = (channel: Channel | ChannelWithStats) => {
    setEditingChannel(channel);
    setShowAddForm(false);
  };

  const handleFormClose = () => {
    setShowAddForm(false);
    setEditingChannel(null);
  };

  const handleFormSuccess = async () => {
    await queryClient.refetchQueries({ queryKey: ["channels"] });
    setShowAddForm(false);
    setEditingChannel(null);
  };

  if (isLoading && channels.length === 0) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }


  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">チャンネル管理</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">ストリームチャンネルの追加・編集・削除</p>
        </div>
        <button
          onClick={handleAddChannel}
          className="btn-primary flex items-center space-x-2"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          <span>チャンネルを追加</span>
        </button>
      </div>

      {/* フィルター */}
      <div className="flex space-x-2">
        <button
          onClick={() => setFilter('all')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            filter === 'all'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          すべて ({channels.length})
        </button>
        <button
          onClick={() => setFilter('twitch')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            filter === 'twitch'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          🎮 Twitch ({channels.filter(c => c.platform === 'twitch').length})
        </button>
        <button
          onClick={() => setFilter('youtube')}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
            filter === 'youtube'
              ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white shadow-md'
              : 'bg-white dark:bg-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-600 border border-gray-200 dark:border-slate-600'
          }`}
        >
          ▶️ YouTube ({channels.filter(c => c.platform === 'youtube').length})
        </button>
      </div>

      {/* 新規追加フォーム */}
      {showAddForm && (
        <div className="card p-6 animate-slide-up">
          <ChannelForm
            onSuccess={handleFormSuccess}
            onCancel={handleFormClose}
          />
        </div>
      )}

      {/* 編集フォーム */}
      {editingChannel && (
        <div className="card p-6 animate-slide-up">
          <ChannelEditForm
            channel={editingChannel}
            onSuccess={handleFormSuccess}
            onCancel={handleFormClose}
          />
        </div>
      )}

      {/* チャンネル一覧 */}
      <div className="space-y-4">
        {filteredChannels.length === 0 ? (
          <div className="card p-12 text-center animate-fade-in">
            <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-gray-100 dark:bg-slate-700 flex items-center justify-center">
              <svg className="w-10 h-10 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <p className="text-gray-600 dark:text-gray-400 font-medium">
              {filter === 'all'
                ? "チャンネルが登録されていません。「チャンネルを追加」ボタンから追加してください。"
                : `${filter === 'twitch' ? 'Twitch' : 'YouTube'} のチャンネルが登録されていません。`
              }
            </p>
          </div>
        ) : (
          filteredChannels.map((channel, index) => (
            <div key={channel.id ?? `${channel.platform}-${channel.channel_id}-${index}`} style={{ animationDelay: `${index * 0.05}s` }} className="animate-fade-in">
              <ChannelItem
                channel={channel}
                onEdit={handleEditChannel}
                onDelete={handleDelete}
                onToggle={handleToggle}
              />
            </div>
          ))
        )}
      </div>
    </div>
  );
}