import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Channel } from "../../types";

interface ChannelFormData {
  platform: 'twitch' | 'youtube';
  channel_id: string;
  channel_name: string;
  poll_interval: number;
}

interface ChannelFormProps {
  channel?: Channel | null;
  onSuccess: () => void;
  onCancel: () => void;
}

export function ChannelForm({ channel, onSuccess, onCancel }: ChannelFormProps) {
  const { register, handleSubmit, formState: { errors }, reset, watch } = useForm<ChannelFormData>({
    defaultValues: channel ? {
      platform: channel.platform as 'twitch' | 'youtube',
      channel_id: channel.channel_id,
      channel_name: channel.channel_name,
      poll_interval: channel.poll_interval,
    } : {
      platform: 'twitch',
      channel_id: '',
      channel_name: '',
      poll_interval: 60,
    }
  });

  const addMutation = useMutation({
    mutationFn: async (data: ChannelFormData) => {
      return await invoke<Channel>("add_channel", {
        request: {
          platform: data.platform,
          channel_id: data.channel_id,
          channel_name: data.channel_name,
          poll_interval: data.poll_interval,
        },
      });
    },
    onSuccess: () => {
      onSuccess();
      reset();
    },
  });

  const updateMutation = useMutation({
    mutationFn: async (data: ChannelFormData) => {
      if (!channel?.id) return;
      return await invoke<Channel>("update_channel", {
        id: channel.id,
        channel_name: data.channel_name,
        poll_interval: data.poll_interval,
        enabled: channel.enabled,
      });
    },
    onSuccess: () => {
      onSuccess();
    },
  });

  const onSubmit = async (data: ChannelFormData) => {
    try {
      // YouTube URLからチャンネルIDを抽出
      let channelId = data.channel_id;
      if (data.platform === 'youtube' && data.channel_id.includes('youtube.com')) {
        // URLからチャンネルIDを抽出
        const urlMatch = data.channel_id.match(/(?:channel\/|user\/|c\/)([a-zA-Z0-9_-]+)/);
        if (urlMatch) {
          channelId = urlMatch[1];
        } else {
          // 短縮URLの場合
          const shortUrlMatch = data.channel_id.match(/youtu\.be\/([a-zA-Z0-9_-]+)/);
          if (shortUrlMatch) {
            channelId = shortUrlMatch[1];
          }
        }
      }

      const submitData = {
        ...data,
        channel_id: channelId,
      };

      if (channel) {
        await updateMutation.mutateAsync(submitData);
      } else {
        await addMutation.mutateAsync(submitData);
      }
    } catch (error: any) {
      const errorMessage = String(error);
      // 重複エラーの場合、より分かりやすいメッセージを表示
      if (errorMessage.includes('Duplicate key') || errorMessage.includes('unique constraint')) {
        alert(`このチャンネルは既に登録されています。\nプラットフォーム: ${data.platform}\nチャンネルID: ${data.channel_id}`);
      } else {
        alert(`チャンネルの${channel ? '更新' : '追加'}に失敗しました: ${errorMessage}`);
      }
    }
  };

  const isLoading = addMutation.isPending || updateMutation.isPending;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {channel ? 'チャンネルを編集' : 'チャンネルを追加'}
        </h2>
        <button
          type="button"
          onClick={onCancel}
          className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
        >
          ✕
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* プラットフォーム選択 */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            プラットフォーム
          </label>
          <select
            {...register("platform", { required: "プラットフォームを選択してください" })}
            disabled={!!channel} // 編集時は変更不可
            className="input-field disabled:bg-gray-100 dark:disabled:bg-slate-700"
          >
            <option value="twitch">Twitch</option>
            <option value="youtube">YouTube</option>
          </select>
          {errors.platform && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.platform.message}</p>
          )}
        </div>

        {/* チャンネルID/URL */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            {watch("platform") === 'youtube' ? 'チャンネルURL' : 'チャンネルID'}
            {channel && <span className="text-xs text-gray-500 dark:text-gray-400 ml-1">（変更不可）</span>}
          </label>
          <input
            {...register("channel_id", {
              required: watch("platform") === 'youtube' ? "チャンネルURLを入力してください" : "チャンネルIDを入力してください",
              validate: (value) => {
                if (watch("platform") === 'youtube') {
                  // YouTube URLの検証
                  const urlPattern = /^(https?:\/\/)?(www\.)?(youtube\.com\/(channel\/|user\/|c\/)|youtu\.be\/)/;
                  if (!urlPattern.test(value)) {
                    return "有効なYouTubeチャンネルURLを入力してください";
                  }
                } else {
                  // Twitch IDの検証
                  const idPattern = /^[a-zA-Z0-9_-]+$/;
                  if (!idPattern.test(value)) {
                    return "チャンネルIDは英数字、ハイフン、アンダースコアのみ使用できます";
                  }
                }
                return true;
              }
            })}
            disabled={!!channel} // 編集時は変更不可
            type="text"
            placeholder={watch("platform") === 'youtube' ? "例: https://www.youtube.com/channel/UC..." : "例: shroud"}
            className="input-field disabled:bg-gray-100 dark:disabled:bg-slate-700"
          />
          {errors.channel_id && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.channel_id.message}</p>
          )}
        </div>

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
            {...register("poll_interval", { valueAsNumber: true })}
            className="input-field"
          >
            <option value={30}>30秒</option>
            <option value={60}>1分</option>
            <option value={180}>3分</option>
            <option value={300}>5分</option>
            <option value={600}>10分</option>
          </select>
        </div>
      </div>

      {/* ヘルプテキスト */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-3">
        <div className="text-sm text-blue-800 dark:text-blue-200">
          <strong>注意:</strong> プラットフォームとチャンネルIDは後から変更できません。
          正しい情報を入力してください。
          {watch("platform") === 'youtube' && (
            <span className="block mt-1">YouTubeの場合は、チャンネルURLを入力してください。</span>
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
          {isLoading ? "保存中..." : (channel ? "更新" : "追加")}
        </button>
      </div>
    </form>
  );
}