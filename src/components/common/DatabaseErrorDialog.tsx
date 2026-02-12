import { useState } from "react";
import { toast } from "../../utils/toast";
import * as systemApi from "../../api/system";

interface DatabaseErrorDialogProps {
  isOpen: boolean;
  errorMessage: string;
  onClose: () => void;
  onSuccess: () => void;
}

export function DatabaseErrorDialog({
  isOpen,
  errorMessage,
  onClose,
  onSuccess,
}: DatabaseErrorDialogProps) {
  const [isProcessing, setIsProcessing] = useState(false);

  if (!isOpen) return null;

  const handleRecreateDatabase = async () => {
    setIsProcessing(true);
    try {
      const result = await systemApi.recreateDatabase();
      console.log("Database recreation result:", result);

      if (result.success) {
        onSuccess();
      } else {
        toast.error(`データベースの再作成に失敗しました: ${result.message}`);
      }
    } catch (error) {
      console.error("Failed to recreate database:", error);
      toast.error(`データベースの再作成中にエラーが発生しました: ${error}`);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="bg-white dark:bg-slate-800 rounded-xl shadow-2xl max-w-md w-full mx-4 p-6 space-y-6">
        {/* ヘッダー */}
        <div className="text-center">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 dark:bg-red-900/20 flex items-center justify-center">
            <svg
              className="w-8 h-8 text-red-600 dark:text-red-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
              />
            </svg>
          </div>
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            データベースエラー
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
            データベースの初期化に失敗しました
          </p>
        </div>

        {/* エラーメッセージ */}
        <div className="bg-red-50 dark:bg-red-900/10 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-sm text-red-800 dark:text-red-200 whitespace-pre-wrap select-text">
            {errorMessage}
          </p>
        </div>

        {/* オプション説明 */}
        <div className="space-y-3 text-sm text-gray-600 dark:text-gray-400">
          <p>
            以下のオプションからデータベースの問題を解決する方法を選択してください：
          </p>
          <ul className="list-disc list-inside space-y-1 ml-4">
            <li>
              <strong>バックアップして新規作成:</strong> 既存のデータをバックアップし、新しいデータベースを作成します
            </li>
            <li>
              <strong>キャンセル:</strong> アプリケーションを終了します
            </li>
          </ul>
        </div>

        {/* アクションボタン */}
        <div className="flex flex-col space-y-3">
          <button
            onClick={handleRecreateDatabase}
            disabled={isProcessing}
            className="w-full bg-indigo-600 hover:bg-indigo-700 disabled:bg-indigo-400 text-white font-medium py-3 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center space-x-2"
          >
            {isProcessing ? (
              <>
                <svg
                  className="animate-spin h-4 w-4"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                <span>処理中...</span>
              </>
            ) : (
              <>
                <svg
                  className="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                  />
                </svg>
                <span>バックアップして新規作成</span>
              </>
            )}
          </button>

          <button
            onClick={onClose}
            disabled={isProcessing}
            className="w-full bg-gray-200 hover:bg-gray-300 disabled:bg-gray-100 dark:bg-slate-700 dark:hover:bg-slate-600 dark:disabled:bg-slate-800 text-gray-800 dark:text-gray-200 font-medium py-3 px-4 rounded-lg transition-colors duration-200"
          >
            キャンセル
          </button>
        </div>
      </div>
    </div>
  );
}