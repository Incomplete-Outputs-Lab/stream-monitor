import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Channel } from "../../types";

interface ChannelEditFormData {
  channel_name: string;
  poll_interval: number;
}

interface ChannelEditFormProps {
  channel: Channel;
  onSuccess: () => void;
  onCancel: () => void;
}

export function ChannelEditForm({ channel, onSuccess, onCancel }: ChannelEditFormProps) {
  const { register, handleSubmit, formState: { errors } } = useForm<ChannelEditFormData>({
    defaultValues: {
      channel_name: channel.channel_name,
      poll_interval: channel.poll_interval,
    }
  });

  const updateMutation = useMutation({
    mutationFn: async (data: ChannelEditFormData) => {
      console.log('[ChannelEditForm] Submitting update:', {
        id: channel.id,
        channel_name: data.channel_name,
        poll_interval: data.poll_interval,
        enabled: channel.enabled,
      });

      return await invoke<Channel>("update_channel", {
        id: channel.id,
        channelName: data.channel_name,
        pollInterval: data.poll_interval,
        enabled: channel.enabled,
      });
    },
    onSuccess: async () => {
      await onSuccess();
    },
    onError: (error) => {
      alert(`チャンネルの更新に失敗しました: ${String(error)}`);
    },
  });

  const onSubmit = async (data: ChannelEditFormData) => {
    await updateMutation.mutateAsync(data);
  };

  const isLoading = updateMutation.isPending;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          チャンネルを編集
        </h2>
        <button
          type="button"
          onClick={onCancel}
          className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        >
          ✕
        </button>
      </div>

      {/* チャンネル情報表示 */}
      <div className="bg-gray-50 dark:bg-slate-700 rounded-lg p-4 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-gray-600 dark:text-gray-400">プラットフォーム</span>
          <span className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {channel.platform === 'twitch' ? '🎮 Twitch' : '▶️ YouTube'}
          </span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-gray-600 dark:text-gray-400">チャンネルID</span>
          <span className="text-sm font-mono text-gray-900 dark:text-gray-100">{channel.channel_id}</span>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4">
        {/* チャンネル名 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            表示名
          </label>
          <input
            {...register("channel_name", { required: "チャンネル名を入力してください" })}
            type="text"
            placeholder="例: Shroud, チャンネル名"
            className="input-field"
          />
          {errors.channel_name && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.channel_name.message}</p>
          )}
        </div>

        {/* ポーリング間隔 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            監視間隔（秒）
          </label>
          <select
            {...register("poll_interval", {
              required: "監視間隔を選択してください",
              setValueAs: (v) => {
                const parsed = parseInt(String(v), 10);
                if (isNaN(parsed)) {
                  console.error('[ChannelEditForm] Invalid poll_interval value:', v);
                  return 60;
                }
                console.log('[ChannelEditForm] poll_interval parsed:', parsed);
                return parsed;
              }
            })}
            className="input-field"
          >
            <option value="30">30秒</option>
            <option value="60">1分</option>
            <option value="180">3分</option>
            <option value="300">5分</option>
            <option value="600">10分</option>
          </select>
          {errors.poll_interval && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.poll_interval.message}</p>
          )}
        </div>
      </div>

      {/* ボタン */}
      <div className="flex justify-end space-x-3">
        <button
          type="button"
          onClick={onCancel}
          className="btn-secondary"
          disabled={isLoading}
        >
          キャンセル
        </button>
        <button
          type="submit"
          disabled={isLoading}
          className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? "更新中..." : "更新"}
        </button>
      </div>
    </form>
  );
}
