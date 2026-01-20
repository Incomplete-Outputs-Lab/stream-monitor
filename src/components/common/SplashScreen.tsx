import { useEffect, useState } from "react";

export function SplashScreen({ onComplete }: { onComplete: () => void }) {
  const [progress, setProgress] = useState(0);
  const [fadeOut, setFadeOut] = useState(false);

  useEffect(() => {
    // プログレスバーのアニメーション
    const progressInterval = setInterval(() => {
      setProgress((prev) => {
        if (prev >= 100) {
          clearInterval(progressInterval);
          return 100;
        }
        // 最初は速く、後半は遅く
        const increment = prev < 70 ? 2 + Math.random() * 3 : 0.5 + Math.random() * 1;
        return Math.min(prev + increment, 100);
      });
    }, 50);

    // フェードアウトと完了
    const fadeTimeout = setTimeout(() => {
      setFadeOut(true);
      setTimeout(() => {
        onComplete();
      }, 500);
    }, 2500); // 2.5秒後にフェードアウト開始

    return () => {
      clearInterval(progressInterval);
      clearTimeout(fadeTimeout);
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
    </div>
  );
}
