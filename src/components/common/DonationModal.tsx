import { useState, useEffect } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';

const STORAGE_KEY = 'donation-modal-shown';

export function DonationModal({ onClose }: { onClose: () => void }) {
  const handleSupport = async () => {
    await openUrl('http://subs.twitch.tv/flowingspdg');
    localStorage.setItem(STORAGE_KEY, 'true');
    onClose();
  };

  const handleLater = () => {
    localStorage.setItem(STORAGE_KEY, 'true');
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in">
      <div className="bg-white dark:bg-slate-800 rounded-xl shadow-2xl max-w-md w-full mx-4 p-6 space-y-4 animate-scale-in">
        {/* ロゴ */}
        <div className="text-center">
          <div className="w-16 h-16 mx-auto rounded-xl bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center shadow-lg">
            <svg
              className="w-10 h-10 text-white"
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
          <h2 className="mt-4 text-xl font-bold text-gray-900 dark:text-white">
            Stream Monitorへようこそ！
          </h2>
        </div>
        
        {/* メッセージ */}
        <div className="text-sm text-gray-600 dark:text-gray-300 space-y-2">
          <p>このアプリをダウンロードいただきありがとうございます。</p>
          <p>Stream Monitorは個人で開発・運営しているオープンソースプロジェクトです。</p>
          <p>もしこのアプリがお役に立てれば、Twitchのサブスクリプションで開発を支援いただけると嬉しいです。</p>
        </div>
        
        {/* ボタン */}
        <div className="flex space-x-3 pt-2">
          <button
            onClick={handleLater}
            className="flex-1 px-4 py-2 bg-gray-100 dark:bg-slate-700 text-gray-700 dark:text-gray-300 rounded-lg font-medium hover:bg-gray-200 dark:hover:bg-slate-600 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
          >
            後で
          </button>
          <button
            onClick={handleSupport}
            className="flex-1 px-4 py-2 bg-gradient-to-r from-purple-500 to-indigo-600 text-white rounded-lg font-medium shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105 active:scale-95 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 flex items-center justify-center gap-2"
          >
            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
              <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z"/>
            </svg>
            サポートする
          </button>
        </div>
      </div>

      <style>
        {`
          .animate-fade-in {
            animation: fadeIn 0.2s ease-out;
          }
          
          .animate-scale-in {
            animation: scaleIn 0.2s ease-out;
          }
          
          @keyframes fadeIn {
            from {
              opacity: 0;
            }
            to {
              opacity: 1;
            }
          }
          
          @keyframes scaleIn {
            from {
              opacity: 0;
              transform: scale(0.95);
            }
            to {
              opacity: 1;
              transform: scale(1);
            }
          }
        `}
      </style>
    </div>
  );
}

export function useDonationModal() {
  const [show, setShow] = useState(false);
  
  useEffect(() => {
    const shown = localStorage.getItem(STORAGE_KEY);
    if (!shown) {
      setShow(true);
    }
  }, []);
  
  return { show, close: () => setShow(false) };
}
