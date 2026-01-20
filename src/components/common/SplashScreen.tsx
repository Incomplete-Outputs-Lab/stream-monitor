import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { DatabaseErrorDialog } from "./DatabaseErrorDialog";
import { DbInitStatus } from "../../types";

export function SplashScreen({ onComplete }: { onComplete: () => void }) {
  const [progress, setProgress] = useState(0);
  const [fadeOut, setFadeOut] = useState(false);
  const [dbInitStatus, setDbInitStatus] = useState<"initializing" | "success" | "error">("initializing");
  const [dbErrorMessage, setDbErrorMessage] = useState<string>("");
  const [showErrorDialog, setShowErrorDialog] = useState(false);

  useEffect(() => {
    let progressInterval: number;
    let fadeTimeout: number;
    let statusCheckInterval: number;

    // DB初期化状態をチェックする関数
    const checkDbInitStatus = async () => {
      try {
        // バックエンドの初期化状態を確認
        const status = await invoke<DbInitStatus>("get_database_init_status");

        if (status.initialized && dbInitStatus === "initializing") {
          console.log("Database initialization status confirmed:", status.message);
          setDbInitStatus("success");
          setProgress(100);

          // 成功時は少し待ってから完了
          fadeTimeout = setTimeout(() => {
            setFadeOut(true);
            setTimeout(() => {
              onComplete();
            }, 500);
          }, 1000);

          // ステータスチェックを停止
          if (statusCheckInterval) {
            clearInterval(statusCheckInterval);
          }
        } else if (!status.initialized) {
          console.log("Database still initializing:", status.message);
        }
      } catch (error) {
        console.error("Failed to check database init status:", error);
      }
    };

    // DB初期化イベントのリスナーを設定
    const setupEventListeners = async () => {
      try {
        // DB初期化成功イベント
        const successUnlisten = await listen("database-init-success", () => {
          console.log("Database initialization successful (via event)");
          setDbInitStatus("success");
          setProgress(100);

          // 成功時は少し待ってから完了
          fadeTimeout = setTimeout(() => {
            setFadeOut(true);
            setTimeout(() => {
              onComplete();
            }, 500);
          }, 1000);

          // ステータスチェックを停止
          if (statusCheckInterval) {
            clearInterval(statusCheckInterval);
          }
        });

        // DB初期化エラーイベント
        const errorUnlisten = await listen("database-init-error", (event: any) => {
          console.error("Database initialization failed:", event.payload);
          setDbInitStatus("error");
          setDbErrorMessage(event.payload);
          setShowErrorDialog(true);
          setProgress(100);

          // ステータスチェックを停止
          if (statusCheckInterval) {
            clearInterval(statusCheckInterval);
          }
        });

        // クリーンアップ関数を返す
        return () => {
          successUnlisten();
          errorUnlisten();
        };
      } catch (error) {
        console.error("Failed to setup event listeners:", error);
      }
    };

    // プログレスバーのアニメーション
    progressInterval = setInterval(() => {
      setProgress((prev) => {
        if (prev >= 90) { // DB初期化が終わるまで90%まで
          return 90;
        }
        // 最初は速く、後半は遅く
        const increment = prev < 50 ? 2 + Math.random() * 3 : 0.5 + Math.random() * 1;
        return Math.min(prev + increment, 90);
      });
    }, 50);

    // イベントリスナーを設定
    let cleanup: (() => void) | undefined;
    setupEventListeners().then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    // 定期的にDB初期化状態を確認（フォールバック）
    statusCheckInterval = setInterval(checkDbInitStatus, 1000); // 1秒ごとにチェック

    return () => {
      if (progressInterval) clearInterval(progressInterval);
      if (fadeTimeout) clearTimeout(fadeTimeout);
      if (statusCheckInterval) clearInterval(statusCheckInterval);
      if (cleanup) cleanup();
    };
  }, [onComplete]);

  return (
    <div
      className={`fixed inset-0 z-50 flex items-center justify-center transition-opacity duration-500 ${
        fadeOut ? "opacity-0" : "opacity-100"
      }`}
      style={{
        background: "linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%)",
      }}
    >
      <div className="text-center space-y-8 px-8">
        {/* ロゴ/アイコン */}
        <div className="relative">
          <div className="absolute inset-0 animate-ping opacity-20">
            <div className="w-32 h-32 mx-auto rounded-full bg-white"></div>
          </div>
          <div className="relative w-32 h-32 mx-auto rounded-full bg-white/90 backdrop-blur-sm flex items-center justify-center shadow-2xl transform transition-transform duration-300 hover:scale-110">
            <svg
              className="w-20 h-20 text-purple-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
              />
            </svg>
          </div>
        </div>

        {/* タイトル */}
        <div className="space-y-2">
          <h1 className="text-5xl font-bold text-white drop-shadow-lg animate-fade-in">
            Stream Monitor
          </h1>
          <p className="text-xl text-white/90 font-light animate-fade-in-delay">
            ストリーム統計収集ツール
          </p>
        </div>

        {/* DB初期化状態表示 */}
        <div className="text-center space-y-2">
          {dbInitStatus === "initializing" && (
            <p className="text-sm text-white/80 animate-fade-in-delay">
              データベースを初期化しています...
            </p>
          )}
          {dbInitStatus === "success" && (
            <p className="text-sm text-green-300 animate-fade-in-delay">
              データベース初期化完了 ✓
            </p>
          )}
          {dbInitStatus === "error" && (
            <p className="text-sm text-red-300 animate-fade-in-delay">
              データベース初期化エラー ✗
            </p>
          )}
        </div>

        {/* プログレスバー */}
        <div className="w-64 mx-auto space-y-2">
          <div className="h-1.5 bg-white/20 rounded-full overflow-hidden">
            <div
              className="h-full bg-white rounded-full transition-all duration-300 ease-out shadow-lg"
              style={{ width: `${progress}%` }}
            />
          </div>
          <p className="text-sm text-white/80 font-medium">{Math.round(progress)}%</p>
        </div>

        {/* 装飾的な要素 */}
        <div className="flex justify-center space-x-2 pt-4">
          {[0, 1, 2].map((i) => (
            <div
              key={i}
              className="w-2 h-2 bg-white rounded-full animate-bounce"
              style={{
                animationDelay: `${i * 0.2}s`,
                animationDuration: "1s",
              }}
            />
          ))}
        </div>
      </div>

      {/* Database Error Dialog */}
      <DatabaseErrorDialog
        isOpen={showErrorDialog}
        errorMessage={dbErrorMessage}
        onClose={() => {
          // キャンセル時はアプリを終了
          window.close();
        }}
        onSuccess={() => {
          // 成功時はスプラッシュ画面を完了
          setShowErrorDialog(false);
          setFadeOut(true);
          setTimeout(() => {
            onComplete();
          }, 500);
        }}
      />
    </div>
  );
}
