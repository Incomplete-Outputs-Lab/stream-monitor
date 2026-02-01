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

## Prerequisites

### Windows
- **Visual Studio Build Tools** (C++ ビルドツール)
  - [Visual Studio Installer](https://visualstudio.microsoft.com/downloads/) から「C++ によるデスクトップ開発」ワークロードをインストール
  - または、[Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022) をインストール
- **CMake** (バージョン 3.15 以上)
  - [CMake 公式サイト](https://cmake.org/download/) からインストール
  - または、`winget install Kitware.CMake` でインストール

### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file \
  cmake \
  pkg-config
```

### macOS
- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```
- **CMake** (Homebrew経由)
  ```bash
  brew install cmake
  ```

### 共通
- **Node.js** (LTS版推奨)
- **Rust** (最新のstable版)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Development

### フロントエンドのみの開発
```bash
npm install
npm run dev
```

### アプリ全体の開発（Tauri含む）
```bash
npm install
npm run tauri dev
```

### ビルド

```bash
npm run tauri build
```

## ビルド時の注意

このプロジェクトは **DuckDB** を `bundled` 機能で同梱しています。初回ビルド時は DuckDB のソースコードがコンパイルされるため、時間がかかります（5-10分程度）。

- Windows: Visual Studio Build Tools と CMake が必要です
- Linux/macOS: `build-essential` (Linux) または Xcode (macOS) と CMake が必要です

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
