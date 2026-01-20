# Stream Stats Collector

A Tauri application for collecting and analyzing stream statistics.

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
