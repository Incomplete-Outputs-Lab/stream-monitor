import { useState, useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Settings } from "./components/Settings";
import { Dashboard } from "./components/Dashboard";
import { ChannelList } from "./components/ChannelList";
import { Statistics } from "./components/Statistics";
import { Export } from "./components/Export";
import { Logs } from "./components/Logs";
import { MultiView } from "./components/MultiView";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { ToastContainer } from "./components/common/Toast";
import { ConfirmDialog } from "./components/common/ConfirmDialog";
import { DonationModal, useDonationModal } from "./components/common/DonationModal";
import { DatabaseErrorDialog } from "./components/common/DatabaseErrorDialog";
import { useThemeStore } from "./stores/themeStore";
import "./App.css";
import { SQLViewer } from "./components/SQL";
import Timeline from "./components/Timeline";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

const queryClient = new QueryClient();

type Tab = "dashboard" | "channels" | "statistics" | "timeline" | "export" | "logs" | "settings" | "multiview" | "sqlviewer";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("dashboard");
  const [showErrorDialog, setShowErrorDialog] = useState(false);
  const [dbErrorMessage, setDbErrorMessage] = useState<string>("");
  const { theme } = useThemeStore();
  const { show: showDonation, close: closeDonation } = useDonationModal();

  useEffect(() => {
    // テーマを適用
    const root = document.documentElement;
    const effectiveTheme = theme === 'system' 
      ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
      : theme;
    
    if (effectiveTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  // メインウィンドウの描画完了時にスプラッシュを閉じて表示
  useEffect(() => {
    const showMainWindow = async () => {
      console.log("Main window rendered, closing splash and showing main window");
      try {
        await invoke("show_main_window");
      } catch (error) {
        console.error("Failed to show main window:", error);
      }
    };

    showMainWindow();
  }, []);

  // DB初期化イベントのリスナーを設定
  useEffect(() => {
    let cleanup: (() => void) | undefined;

    const setupEventListeners = async () => {
      // DB初期化成功イベント（ログのみ）
      const successUnlisten = await listen("database-init-success", () => {
        console.log("Database initialization successful");
      });

      // DB初期化エラーイベント
      const errorUnlisten = await listen("database-init-error", (event: any) => {
        console.error("Database initialization failed:", event.payload);
        setDbErrorMessage(event.payload);
        setShowErrorDialog(true);
      });

      return () => {
        successUnlisten();
        errorUnlisten();
      };
    };

    setupEventListeners().then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    return () => {
      if (cleanup) cleanup();
    };
  }, []);

  // ネイティブアプリらしい動作を設定
  useEffect(() => {
    // コンテキストメニューを無効化（右クリックメニュー）
    const handleContextMenu = (e: MouseEvent) => {
      // 開発モードでは許可（Ctrl/Cmdキーを押しながらの場合のみ）
      if (!e.ctrlKey && !e.metaKey) {
        e.preventDefault();
      }
    };

    // ドラッグ開始を無効化（画像やリンクのドラッグ防止）
    const handleDragStart = (e: DragEvent) => {
      e.preventDefault();
    };

    // 選択開始を制御（ダブルクリックでの選択を防止）
    const handleSelectStart = (e: Event) => {
      const target = e.target as HTMLElement;
      // 入力フィールド、テキストエリア、contenteditable要素では許可
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }
      e.preventDefault();
    };

    document.addEventListener('contextmenu', handleContextMenu);
    document.addEventListener('dragstart', handleDragStart);
    document.addEventListener('selectstart', handleSelectStart);

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu);
      document.removeEventListener('dragstart', handleDragStart);
      document.removeEventListener('selectstart', handleSelectStart);
    };
  }, []);

  const tabs: { id: Tab; label: string; component: React.ReactNode }[] = [
    {
      id: "dashboard",
      label: "ダッシュボード",
      component: <Dashboard />
    },
    {
      id: "channels",
      label: "チャンネル管理",
      component: <ChannelList />
    },
    {
      id: "statistics",
      label: "統計閲覧",
      component: <Statistics />
    },
    {
      id: "timeline",
      label: "タイムライン",
      component: <Timeline />
    },
    {
      id: "export",
      label: "エクスポート",
      component: <Export />
    },
    {
      id: "logs",
      label: "ログ",
      component: <Logs />
    },
    {
      id: "multiview",
      label: "マルチビュー",
      component: <MultiView />
    },
    {
      id: "sqlviewer",
      label: "SQLビューア",
      component: <SQLViewer />
    },
    {
      id: "settings",
      label: "設定",
      component: <Settings />
    },
  ];

  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <ToastContainer />
        <ConfirmDialog />
        {showDonation && <DonationModal onClose={closeDonation} />}
        <DatabaseErrorDialog
          isOpen={showErrorDialog}
          errorMessage={dbErrorMessage}
          onClose={() => {
            // キャンセル時はアプリを終了
            window.close();
          }}
          onSuccess={() => {
            // 成功時はダイアログを閉じる（メインウィンドウは既に表示済み）
            setShowErrorDialog(false);
          }}
        />
        <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900">
          {/* ナビゲーションバー */}
          <nav className="bg-white/80 dark:bg-slate-800/80 backdrop-blur-xl shadow-sm border-b border-gray-200/50 dark:border-slate-700/50 sticky top-0 z-40">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex justify-between items-center h-16">
                <div className="flex items-center space-x-3">
                  <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center shadow-lg">
                    <svg
                      className="w-6 h-6 text-white"
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
                  <div>
                    <h1 className="text-xl font-bold bg-gradient-to-r from-purple-600 to-indigo-600 bg-clip-text text-transparent">
                      Stream Monitor
                    </h1>
                    <p className="text-xs text-gray-500 dark:text-gray-400">Real Time Stream Analysis System</p>
                  </div>
                </div>
              </div>
            </div>

            {/* タブナビゲーション */}
            <div className="border-t border-gray-200/50 dark:border-slate-700/50 bg-white/50 dark:bg-slate-800/50 backdrop-blur-sm">
              <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <nav className="flex space-x-1">
                  {tabs.map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`relative py-4 px-4 font-medium text-sm transition-all duration-200 ${
                        activeTab === tab.id
                          ? "text-indigo-600 dark:text-indigo-400"
                          : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                      }`}
                    >
                      <span className="relative z-10">{tab.label}</span>
                      {activeTab === tab.id && (
                        <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-purple-500 to-indigo-600 rounded-t-full"></span>
                      )}
                    </button>
                  ))}
                </nav>
              </div>
            </div>
          </nav>

          {/* メインコンテンツ */}
          <main className="flex-1">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
              {tabs.find(tab => tab.id === activeTab)?.component}
            </div>
          </main>
        </div>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;
