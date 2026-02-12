import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { getToken } from "../../utils/keyring";
import { toast } from "../../utils/toast";
import * as channelsApi from "../../api/channels";
import * as configApi from "../../api/config";

interface ChannelFormData {
  platform: 'twitch' | 'youtube';
  channel_id: string;
  channel_name: string;
  poll_interval: number;
}

interface ChannelFormProps {
  onSuccess: () => void;
  onCancel: () => void;
}

export function ChannelForm({ onSuccess, onCancel }: ChannelFormProps) {
  const [isValidating, setIsValidating] = useState(false);
  const [validatedInfo, setValidatedInfo] = useState<configApi.TwitchChannelInfo | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  const { register, handleSubmit, formState: { errors }, reset, watch, setValue } = useForm<ChannelFormData>({
    defaultValues: {
      platform: 'twitch',
      channel_id: '',
      channel_name: '',
      poll_interval: 60,
    }
  });

  const addMutation = useMutation({
    mutationFn: async (data: ChannelFormData & { display_name?: string; profile_image_url?: string; follower_count?: number; broadcaster_type?: string; twitch_user_id?: number }) => {
      return await channelsApi.addChannel({
        platform: data.platform,
        channel_id: data.channel_id,
        channel_name: data.channel_name,
        poll_interval: data.poll_interval,
        twitch_user_id: data.twitch_user_id,
      });
    },
    onSuccess: () => {
      onSuccess();
      reset();
      setValidatedInfo(null);
      setValidationError(null);
    },
  });


  // Twitchチャンネルのバリデーション
  const handleValidateTwitchChannel = async (channelId: string) => {
    if (!channelId.trim()) {
      setValidationError('チャンネルIDを入力してください');
      return;
    }

    setIsValidating(true);
    setValidationError(null);
    setValidatedInfo(null);

    try {
      // Get access token from Stronghold
      let accessToken: string | null = null;
      try {
        accessToken = await getToken('twitch');
      } catch (e) {
        console.error('Failed to get token from Stronghold:', e);
      }

      const info = await configApi.validateTwitchChannel(
        channelId.trim(),
        accessToken
      );
      
      setValidatedInfo(info);
      // 検証成功時、display_nameを自動設定
      setValue('channel_name', info.display_name);
      setValidationError(null);
    } catch (error: any) {
      const errorMessage = String(error);
      setValidationError(errorMessage);
      setValidatedInfo(null);
    } finally {
      setIsValidating(false);
    }
  };

  const onSubmit = async (data: ChannelFormData) => {
    try {
      // Twitchの場合、バリデーションが必要
      if (data.platform === 'twitch' && !validatedInfo) {
        setValidationError('チャンネルを追加する前に、「チャンネルを確認」ボタンで検証してください。');
        return;
      }

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

      // Twitchの場合、検証済みのchannel_idを使用
      if (data.platform === 'twitch' && validatedInfo) {
        channelId = validatedInfo.channel_id;
      }

      const submitData = {
        ...data,
        channel_id: channelId,
        // Twitchの場合、検証済みの情報を追加
        ...(data.platform === 'twitch' && validatedInfo ? {
          display_name: validatedInfo.display_name,
          profile_image_url: validatedInfo.profile_image_url,
          follower_count: validatedInfo.follower_count,
          broadcaster_type: validatedInfo.broadcaster_type,
          twitch_user_id: validatedInfo.twitch_user_id,
        } : {}),
      };

      await addMutation.mutateAsync(submitData);
    } catch (error: any) {
      const errorMessage = String(error);
      // 重複エラーの場合、より分かりやすいメッセージを表示
      if (errorMessage.includes('Duplicate key') || errorMessage.includes('unique constraint')) {
        toast.error(`このチャンネルは既に登録されています。\nプラットフォーム: ${data.platform}\nチャンネルID: ${data.channel_id}`);
      } else {
        toast.error(`チャンネルの追加に失敗しました: ${errorMessage}`);
      }
    }
  };

  const isLoading = addMutation.isPending;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          チャンネルを追加
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
            className="input-field"
          >
            <option value="twitch">Twitch</option>
            <option value="youtube">YouTube</option>
          </select>
          {errors.platform && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.platform.message}</p>
          )}
        </div>

        {/* チャンネルID/URL */}
        <div className="md:col-span-2">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            {watch("platform") === 'youtube' ? 'チャンネルURL' : 'チャンネルID'}
          </label>
          <div className="flex gap-2">
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
                },
                onChange: () => {
                  // 入力が変更されたら検証状態をリセット
                  setValidatedInfo(null);
                  setValidationError(null);
                }
              })}
              type="text"
              placeholder={watch("platform") === 'youtube' ? "例: https://www.youtube.com/channel/UC..." : "例: shroud"}
              className="input-field flex-1"
            />
            {watch("platform") === 'twitch' && (
              <button
                type="button"
                onClick={() => handleValidateTwitchChannel(watch("channel_id"))}
                disabled={isValidating || !watch("channel_id")}
                className="btn-secondary whitespace-nowrap disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isValidating ? (
                  <>
                    <span className="inline-block animate-spin mr-2">⏳</span>
                    確認中...
                  </>
                ) : (
                  'チャンネルを確認'
                )}
              </button>
            )}
          </div>
          {errors.channel_id && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.channel_id.message}</p>
          )}
          {validationError && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">❌ {validationError}</p>
          )}
          {validatedInfo && (
            <div className="mt-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md">
              <div className="flex items-center gap-3">
                <img 
                  src={validatedInfo.profile_image_url} 
                  alt={validatedInfo.display_name}
                  className="w-10 h-10 rounded-full"
                />
                <div>
                  <p className="text-sm font-medium text-green-800 dark:text-green-200">
                    ✅ チャンネルが見つかりました: {validatedInfo.display_name}
                  </p>
                  {validatedInfo.description && (
                    <p className="text-xs text-green-600 dark:text-green-300 mt-1">
                      {validatedInfo.description.slice(0, 100)}{validatedInfo.description.length > 100 ? '...' : ''}
                    </p>
                  )}
                </div>
              </div>
            </div>
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
            {...register("poll_interval", { 
              setValueAs: (v) => {
                const parsed = parseInt(String(v), 10);
                return isNaN(parsed) ? 60 : parsed;
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
          {isLoading ? "追加中..." : "追加"}
        </button>
      </div>
    </form>
  );
}