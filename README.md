# Stream Stats Collector

A Tauri application for collecting and analyzing stream statistics.

## 統計情報の用語集

アプリケーションの統計画面で使用されている主な用語の解説です。

| 用語 | 意味 | 解説 |
| :--- | :--- | :--- |
| **MW** | Minutes Watched | **総視聴時間（分）**。配信中に視聴者が滞在した時間の合計です。計算式: `視聴者数 × 経過時間`。配信の影響力を測る最も重要な指標の一つです。 |
| **Avg CCU** | Average Concurrent Users | **平均同時視聴者数**。期間内の配信における平均的な視聴者数です。 |
| **Peak CCU** | Peak Concurrent Users | **最大同時視聴者数**。期間内の配信で最も視聴者が多かった瞬間の数値です。 |
| **Hours** | Hours Broadcasted | **総配信時間**。実際に配信を行っていた時間の合計です。 |
| **Streams** | Stream Count | **配信回数**。指定した期間内に行われた配信の回数です。 |
| **P/A Ratio** | Peak to Average Ratio | **ピーク集中度**。`Peak CCU / Avg CCU` で計算されます。1に近いほど視聴者が安定しており、数値が大きいほど特定のタイミング（バズりや企画など）で視聴者が集中したことを示します。 |
| **Chat Msgs** | Total Chat Messages | **総チャットメッセージ数**。収集されたチャットメッセージの合計数です。 |
| **Engagement** | Engagement Rate | **エンゲージメント率**。視聴者の参加度を示す指標です。本アプリでは `(総チャット数 / MW) × 1000`（1000分視聴あたりのチャット数）として計算されます。 |
| **MW%** | Minutes Watched Percentage | **視聴時間占有率**。特定のゲームタイトルや配信者が、全体（またはその配信者の総視聴時間）のうち何割を占めているかを示します。 |
| **Main Title** | Main Played Title | **主な配信カテゴリ**。最も多くの MW（視聴時間）を記録したゲームやカテゴリです。 |
