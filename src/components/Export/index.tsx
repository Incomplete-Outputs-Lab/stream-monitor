import { useMutation, useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Channel } from "../../types";
import { ExportForm } from "./ExportForm";

type ExportFormat = 'csv' | 'json';
type AggregationType = 'raw' | '1min' | '5min' | '1hour';

interface ExportConfig {
  channelIds: number[];
  startDate: string;
  endDate: string;
  format: ExportFormat;
  aggregation: AggregationType;
  includeChatData: boolean;
}

export function Export() {
  const [exportConfig, setExportConfig] = useState<ExportConfig>({
    channelIds: [],
    startDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], // 7日前
    endDate: new Date().toISOString().split('T')[0], // 今日
    format: 'csv',
    aggregation: 'raw',
    includeChatData: false,
  });

  // チャンネル一覧取得
  const { data: channels } = useQuery({
    queryKey: ["channels"],
    queryFn: async () => {
      return await invoke<Channel[]>("list_channels");
    },
  });

  // CSVエクスポートミューテーション
  const csvExportMutation = useMutation({
    mutationFn: async (config: ExportConfig) => {
      // ファイル保存ダイアログを表示（Tauriのdialog APIを使用）
      const filePath = await invoke<string>("export_to_csv", {
        query: {
          channel_id: config.channelIds.length === 1 ? config.channelIds[0] : undefined,
          start_time: new Date(config.startDate).toISOString(),
          end_time: new Date(config.endDate + 'T23:59:59').toISOString(),
        },
        file_path: "", // 空文字を渡すとダイアログが表示される
      });
      return filePath;
    },
    onSuccess: (filePath) => {
      alert(`CSVファイルをエクスポートしました: ${filePath}`);
    },
    onError: (error) => {
      alert(`エクスポートに失敗しました: ${String(error)}`);
    },
  });

  // JSONエクスポートミューテーション（実装予定）
  const jsonExportMutation = useMutation({
    mutationFn: async (_config: ExportConfig) => {
      // TODO: JSONエクスポート実装
      throw new Error("JSONエクスポートはまだ実装されていません");
    },
  });


  const handleExport = async () => {
    if (exportConfig.channelIds.length === 0) {
      alert("少なくとも1つのチャンネルを選択してください");
      return;
    }

    try {
      switch (exportConfig.format) {
        case 'csv':
          await csvExportMutation.mutateAsync(exportConfig);
          break;
        case 'json':
          await jsonExportMutation.mutateAsync(exportConfig);
          break;
      }
    } catch (error) {
      console.error("Export failed:", error);
    }
  };

  const isExporting = csvExportMutation.isPending || jsonExportMutation.isPending;

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">データエクスポート</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">収集した統計データを様々な形式でエクスポートできます</p>
        </div>
      </div>

      {/* エクスポート設定フォーム */}
      <div className="card">
        <div className="p-6">
          <ExportForm
            config={exportConfig}
            onConfigChange={setExportConfig}
            channels={channels || []}
          />
        </div>
      </div>

      {/* エクスポート実行ボタン */}
      <div className="flex justify-between items-center card p-6">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">エクスポート実行</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            選択した設定でデータをエクスポートします
          </p>
        </div>

        <button
          onClick={handleExport}
          disabled={isExporting}
          className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isExporting ? "エクスポート中..." : "エクスポート"}
        </button>
      </div>

      {/* エクスポート形式の説明 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">CSV形式</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            表計算ソフトで開ける標準的な形式です。ExcelやGoogle Sheetsで直接開けます。
          </p>
          <div className="text-xs text-green-600 dark:text-green-400 font-medium">✓ 実装済み</div>
        </div>

        <div className="card p-6">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">JSON形式</h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            プログラムで扱いやすい構造化データ形式です。API連携やデータ分析に適しています。
          </p>
          <div className="text-xs text-yellow-600 dark:text-yellow-400 font-medium">🔄 開発中</div>
        </div>

      </div>

      {/* 集計オプションの説明 */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-4">集計オプションについて</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
          <div>
            <h4 className="font-medium text-blue-800 dark:text-blue-200 mb-2">生データ (Raw)</h4>
            <p className="text-blue-700 dark:text-blue-300">
              収集した全てのデータをそのままエクスポートします。詳細な分析に適しています。
            </p>
          </div>
          <div>
            <h4 className="font-medium text-blue-800 dark:text-blue-200 mb-2">1分/5分/1時間集計</h4>
            <p className="text-blue-700 dark:text-blue-300">
              指定した時間間隔でデータを集計します。トレンド分析やレポート作成に適しています。
            </p>
          </div>
        </div>
      </div>

      {/* 注意事項 */}
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-yellow-800 dark:text-yellow-200 mb-2">注意事項</h3>
        <ul className="text-sm text-yellow-700 dark:text-yellow-300 space-y-1">
          <li>• 大量のデータをエクスポートする場合、時間がかかる可能性があります</li>
          <li>• エクスポートしたデータはローカル環境に保存されます</li>
          <li>• チャットデータを含む場合、ファイルサイズが大きくなる可能性があります</li>
        </ul>
      </div>
    </div>
  );
}