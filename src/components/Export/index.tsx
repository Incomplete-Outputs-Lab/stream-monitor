import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { ExportForm } from './ExportForm';
import type { Channel, ExportQuery } from '../../types';

export function Export() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [isExporting, setIsExporting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [previewData, setPreviewData] = useState<string>('');
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);

  const [config, setConfig] = useState({
    channelIds: [] as number[],
    startDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    endDate: new Date().toISOString().split('T')[0],
    format: 'tsv' as 'csv' | 'tsv' | 'custom',
    aggregation: 'raw' as 'raw' | '1min' | '5min' | '1hour',
    customDelimiter: '|',
  });

  // Fetch channels on mount
  useEffect(() => {
    const fetchChannels = async () => {
      try {
        const result = await invoke<Channel[]>('list_channels');
        setChannels(result);
      } catch (error) {
        console.error('Failed to fetch channels:', error);
        setMessage({ type: 'error', text: 'チャンネルの取得に失敗しました' });
      }
    };

    fetchChannels();
  }, []);

  // Clear message after 5 seconds
  useEffect(() => {
    if (message) {
      const timer = setTimeout(() => setMessage(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [message]);

  // Fetch preview when config changes
  useEffect(() => {
    const fetchPreview = async () => {
      if (config.channelIds.length === 0) {
        setPreviewData('');
        return;
      }

      if (config.format === 'custom' && !config.customDelimiter) {
        setPreviewData('');
        return;
      }

      setIsLoadingPreview(true);
      try {
        // Determine delimiter
        let delimiter: string;
        switch (config.format) {
          case 'csv':
            delimiter = ',';
            break;
          case 'tsv':
            delimiter = '\t';
            break;
          case 'custom':
            delimiter = config.customDelimiter;
            break;
          default:
            delimiter = ',';
        }

        // Use first selected channel for preview
        const query: ExportQuery = {
          channel_id: config.channelIds[0],
          start_time: `${config.startDate}T00:00:00Z`,
          end_time: `${config.endDate}T23:59:59Z`,
          aggregation: config.aggregation === 'raw' ? undefined : config.aggregation,
          delimiter,
        };

        const preview = await invoke<string>('preview_export_data', {
          query,
          maxRows: 10,
        });

        setPreviewData(preview);
      } catch (error) {
        console.error('Failed to fetch preview:', error);
        setPreviewData('プレビューの取得に失敗しました');
      } finally {
        setIsLoadingPreview(false);
      }
    };

    // Debounce preview fetch
    const timer = setTimeout(fetchPreview, 500);
    return () => clearTimeout(timer);
  }, [config.channelIds, config.startDate, config.endDate, config.format, config.aggregation, config.customDelimiter]);

  const handleExport = async () => {
    // Validation
    if (config.channelIds.length === 0) {
      setMessage({ type: 'error', text: 'エクスポート対象のチャンネルを選択してください' });
      return;
    }

    if (!config.startDate || !config.endDate) {
      setMessage({ type: 'error', text: '開始日と終了日を指定してください' });
      return;
    }

    if (config.format === 'custom' && !config.customDelimiter) {
      setMessage({ type: 'error', text: 'カスタム区切り文字を入力してください' });
      return;
    }

    try {
      setIsExporting(true);
      setMessage(null);

      // Determine file extension and delimiter
      let fileExtension = 'txt';
      let delimiter: string;

      switch (config.format) {
        case 'csv':
          fileExtension = 'csv';
          delimiter = ',';
          break;
        case 'tsv':
          fileExtension = 'tsv';
          delimiter = '\t';
          break;
        case 'custom':
          fileExtension = 'txt';
          delimiter = config.customDelimiter;
          break;
      }

      // Show save dialog
      const filePath = await save({
        defaultPath: `export_${new Date().toISOString().split('T')[0]}.${fileExtension}`,
        filters: [{
          name: `${config.format.toUpperCase()} Files`,
          extensions: [fileExtension]
        }]
      });

      if (!filePath) {
        setIsExporting(false);
        return; // User cancelled
      }

      // Export for each selected channel
      const results: string[] = [];
      for (const channelId of config.channelIds) {
        const query: ExportQuery = {
          channel_id: channelId,
          start_time: `${config.startDate}T00:00:00Z`,
          end_time: `${config.endDate}T23:59:59Z`,
          aggregation: config.aggregation === 'raw' ? undefined : config.aggregation,
          delimiter,
        };

        const result = await invoke<string>('export_to_delimited', {
          query,
          filePath: config.channelIds.length === 1 ? filePath : `${filePath.replace(/\.[^.]+$/, '')}_ch${channelId}.${fileExtension}`,
          includeBom: true, // Add BOM for Excel compatibility
        });
        results.push(result);
      }

      setMessage({
        type: 'success',
        text: config.channelIds.length === 1
          ? 'エクスポートが完了しました'
          : `${config.channelIds.length}個のファイルをエクスポートしました`
      });
    } catch (error) {
      console.error('Export error:', error);
      setMessage({
        type: 'error',
        text: `エクスポートに失敗しました: ${error}`
      });
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="space-y-6 animate-fade-in">
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">データエクスポート</h1>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">収集した統計データを様々な形式でエクスポート</p>
      </div>

      {/* Message Display */}
      {message && (
        <div className={`card p-4 ${
          message.type === 'success'
            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
            : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
        }`}>
          <p className={`text-sm ${
            message.type === 'success'
              ? 'text-green-800 dark:text-green-200'
              : 'text-red-800 dark:text-red-200'
          }`}>
            {message.text}
          </p>
        </div>
      )}

      {/* Export Form */}
      <div className="card p-6">
        <ExportForm
          config={config}
          onConfigChange={setConfig}
          channels={channels}
        />

        {/* Preview Section */}
        {previewData && (
          <div className="mt-6 pt-6 border-t border-gray-200 dark:border-slate-600">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">
                データプレビュー（最初の10行）
              </h3>
              {isLoadingPreview && (
                <span className="text-xs text-gray-500 dark:text-gray-400">読み込み中...</span>
              )}
            </div>
            
            {/* Code block style container */}
            <div className="rounded-lg overflow-hidden border border-gray-300 dark:border-slate-600 shadow-sm">
              {/* Header bar like markdown code blocks */}
              <div className="flex items-center justify-between bg-gray-100 dark:bg-slate-700 px-4 py-2 border-b border-gray-300 dark:border-slate-600">
                <span className="text-xs font-medium text-gray-600 dark:text-gray-400 uppercase tracking-wide">
                  {config.format === 'csv' ? 'CSV' : config.format === 'tsv' ? 'TSV' : 'Custom'}
                </span>
                <button
                  onClick={() => {
                    navigator.clipboard.writeText(previewData);
                    setMessage({ type: 'success', text: 'プレビューをクリップボードにコピーしました' });
                  }}
                  className="text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors flex items-center gap-1"
                  title="クリップボードにコピー"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  コピー
                </button>
              </div>
              
              {/* Code content */}
              <div className="bg-white dark:bg-slate-900 p-4 overflow-x-auto max-h-96">
                <pre className="text-xs font-mono text-gray-900 dark:text-gray-100 whitespace-pre leading-relaxed">
                  {previewData}
                </pre>
              </div>
            </div>
            
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
              ※ 実際のエクスポートでは全データが出力されます
            </p>
          </div>
        )}

        {/* Export Button */}
        <div className="mt-6 pt-6 border-t border-gray-200 dark:border-slate-600">
          <button
            onClick={handleExport}
            disabled={isExporting || config.channelIds.length === 0}
            className="btn-primary w-full md:w-auto px-8 py-3 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isExporting ? (
              <span className="flex items-center justify-center">
                <svg className="animate-spin -ml-1 mr-3 h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                エクスポート中...
              </span>
            ) : (
              'エクスポート'
            )}
          </button>
        </div>
      </div>
    </div>
  );
}