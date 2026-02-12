import { Channel } from "../../types";

type ExportFormat = 'csv' | 'tsv' | 'custom';
type AggregationType = 'raw' | '1min' | '5min' | '1hour';

interface ExportConfig {
  channelId: number | null;
  startDate: string;
  endDate: string;
  format: ExportFormat;
  aggregation: AggregationType;
  customDelimiter: string;
}

interface ExportFormProps {
  config: ExportConfig;
  onConfigChange: (config: ExportConfig) => void;
  channels: Channel[];
  isLoadingChannels?: boolean;
}

export function ExportForm({ config, onConfigChange, channels, isLoadingChannels = false }: ExportFormProps) {
  const updateConfig = (updates: Partial<ExportConfig>) => {
    onConfigChange({ ...config, ...updates });
  };

  const handleChannelSelect = (channelId: number) => {
    updateConfig({ channelId });
  };

  const formatOptions = [
    { value: 'csv' as const, label: 'CSV', description: 'カンマ区切り' },
    { value: 'tsv' as const, label: 'TSV', description: 'タブ区切り' },
    { value: 'custom' as const, label: 'カスタム', description: '任意の区切り文字' },
  ];

  const aggregationOptions = [
    { value: 'raw' as const, label: '生データ', description: '収集間隔ごと' },
    { value: '1min' as const, label: '1分集計', description: '1分単位で平均化' },
    { value: '5min' as const, label: '5分集計', description: '5分単位で平均化' },
    { value: '1hour' as const, label: '1時間集計', description: '1時間単位で平均化' },
  ];

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">エクスポート設定</h2>

      {/* チャンネル選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          エクスポート対象チャンネル
        </label>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 max-h-48 overflow-y-auto border border-gray-300 dark:border-slate-600 rounded-md p-3 bg-white dark:bg-slate-800">
          {isLoadingChannels ? (
            // Skeleton loading state
            <>
              {[...Array(6)].map((_, i) => (
                <div key={i} className="flex items-center space-x-2 animate-pulse">
                  <div className="w-4 h-4 bg-gray-300 dark:bg-slate-600 rounded"></div>
                  <div className="flex-1 space-y-1">
                    <div className="h-3 bg-gray-300 dark:bg-slate-600 rounded w-3/4"></div>
                    <div className="h-2 bg-gray-300 dark:bg-slate-600 rounded w-1/2"></div>
                  </div>
                </div>
              ))}
            </>
          ) : channels.length === 0 ? (
            <p className="text-gray-500 dark:text-gray-400 text-sm">チャンネルが登録されていません</p>
          ) : (
            channels.map((channel) => (
              <label key={channel.id} className="flex items-center space-x-2">
                <input
                  type="radio"
                  name="exportChannel"
                  checked={config.channelId === channel.id}
                  onChange={() => handleChannelSelect(channel.id!)}
                  className="rounded border-gray-300 dark:border-slate-600 text-blue-600 dark:text-blue-400 focus:ring-blue-500 bg-white dark:bg-slate-700"
                />
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  {channel.display_name || channel.channel_name}
                  <span className="text-xs text-gray-500 dark:text-gray-400 ml-1">
                    ({channel.platform})
                  </span>
                </span>
              </label>
            ))
          )}
        </div>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          選択中のチャンネル:{' '}
          {config.channelId != null
            ? (() => {
                const selected = channels.find((c) => c.id === config.channelId);
                return selected
                  ? `${selected.display_name || selected.channel_name} (${selected.platform})`
                  : '未選択';
              })()
            : '未選択'}
        </p>
      </div>

      {/* 日付範囲選択 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            開始日
          </label>
          <input
            type="date"
            value={config.startDate}
            onChange={(e) => updateConfig({ startDate: e.target.value })}
            className="input-field"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            終了日
          </label>
          <input
            type="date"
            value={config.endDate}
            onChange={(e) => updateConfig({ endDate: e.target.value })}
            className="input-field"
          />
        </div>
      </div>

      {/* エクスポート形式選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          エクスポート形式
        </label>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          {formatOptions.map((option) => (
            <label
              key={option.value}
              className={`relative flex cursor-pointer rounded-lg border p-4 focus:outline-none transition-colors ${
                config.format === option.value
                  ? 'border-blue-500 dark:border-blue-400 bg-blue-50 dark:bg-blue-900/30'
                  : 'border-gray-300 dark:border-slate-600 bg-white dark:bg-slate-700 hover:bg-gray-50 dark:hover:bg-slate-600'
              }`}
            >
              <input
                type="radio"
                name="format"
                value={option.value}
                checked={config.format === option.value}
                onChange={(e) => updateConfig({ format: e.target.value as ExportFormat })}
                className="sr-only"
              />
              <span className="flex flex-1">
                <span className="flex flex-col">
                  <span className={`block text-sm font-medium ${
                    config.format === option.value ? 'text-blue-900 dark:text-blue-100' : 'text-gray-900 dark:text-gray-100'
                  }`}>
                    {option.label}
                  </span>
                  <span className={`block text-sm ${
                    config.format === option.value ? 'text-blue-700 dark:text-blue-300' : 'text-gray-500 dark:text-gray-400'
                  }`}>
                    {option.description}
                  </span>
                </span>
              </span>
              {config.format === option.value && (
                <span className="absolute -inset-px rounded-lg border-2 border-blue-500 dark:border-blue-400 pointer-events-none" />
              )}
            </label>
          ))}
        </div>

        {/* カスタム区切り文字入力 */}
        {config.format === 'custom' && (
          <div className="mt-3">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              区切り文字を入力
            </label>
            <input
              type="text"
              value={config.customDelimiter}
              onChange={(e) => updateConfig({ customDelimiter: e.target.value })}
              placeholder="例: | または ;"
              maxLength={5}
              className="input-field max-w-xs"
            />
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              任意の文字列を区切り文字として使用できます（最大5文字）
            </p>
          </div>
        )}
      </div>

      {/* 集計オプション */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          データ集計
        </label>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          {aggregationOptions.map((option) => (
            <label
              key={option.value}
              className={`relative flex cursor-pointer rounded-lg border p-3 focus:outline-none transition-colors ${
                config.aggregation === option.value
                  ? 'border-green-500 dark:border-green-400 bg-green-50 dark:bg-green-900/30'
                  : 'border-gray-300 dark:border-slate-600 bg-white dark:bg-slate-700 hover:bg-gray-50 dark:hover:bg-slate-600'
              }`}
            >
              <input
                type="radio"
                name="aggregation"
                value={option.value}
                checked={config.aggregation === option.value}
                onChange={(e) => updateConfig({ aggregation: e.target.value as AggregationType })}
                className="sr-only"
              />
              <span className="flex flex-1">
                <span className="flex flex-col">
                  <span className={`block text-sm font-medium ${
                    config.aggregation === option.value ? 'text-green-900 dark:text-green-100' : 'text-gray-900 dark:text-gray-100'
                  }`}>
                    {option.label}
                  </span>
                  <span className={`block text-xs ${
                    config.aggregation === option.value ? 'text-green-700 dark:text-green-300' : 'text-gray-500 dark:text-gray-400'
                  }`}>
                    {option.description}
                  </span>
                </span>
              </span>
            </label>
          ))}
        </div>
      </div>

    </div>
  );
}