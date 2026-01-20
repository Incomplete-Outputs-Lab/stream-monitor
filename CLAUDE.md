# CLAUDE.md - Stream Stats Collector 開発ガイド

このファイルは、AIアシスタント（Claude）がこのプロジェクトを理解し、適切に支援するためのガイドラインです。

## プロジェクト概要

**Stream Stats Collector** は、TwitchとYouTubeの配信統計を定期的に収集し、視聴者数・チャットログ・配信情報を記録・分析するTauriベースのデスクトップアプリケーションです。

## 技術スタック

### フロントエンド
- **フレームワーク**: React 19 + TypeScript
- **ビルドツール**: Vite 7
- **スタイリング**: Tailwind CSS 4
- **状態管理**: 
  - Zustand (グローバル状態)
  - TanStack Query (サーバー状態)
- **フォーム**: React Hook Form
- **グラフ**: Recharts
- **Tauri API**: @tauri-apps/api v2

### バックエンド
- **フレームワーク**: Tauri 2.x
- **言語**: Rust (Edition 2021)
- **非同期ランタイム**: Tokio
- **HTTP クライアント**: reqwest
- **データベース**: DuckDB (bundled)
- **認証情報管理**: keyring
- **シリアライゼーション**: serde + serde_json
- **日時処理**: chrono
- **Twitch API**: twitch_api クレート
- **YouTube API**: google-youtube3
- **WebSocket**: tungstenite (Twitch IRC用)
- **スクレイピング**: scraper

## ディレクトリ構造

```
stream-monitor/
├── src/                          # フロントエンド (React + TypeScript)
│   ├── App.tsx                  # ルートコンポーネント
│   ├── components/              # React コンポーネント
│   │   ├── ChannelList/         # チャンネル管理UI
│   │   ├── Dashboard/           # ダッシュボード
│   │   ├── Statistics/          # 統計表示
│   │   ├── Settings/            # 設定画面（APIトークン/OAuth設定）
│   │   ├── Export/              # エクスポート機能
│   │   └── common/              # 共通コンポーネント
│   │       └── charts/          # Recharts ラッパーコンポーネント
│   ├── stores/                  # Zustand ストア（チャンネル設定・テーマ等）
│   ├── hooks/                   # カスタムフック
│   ├── types/                   # TypeScript 型定義
│   └── utils/                   # ユーティリティ関数
│
├── src-tauri/                   # バックエンド (Rust)
│   ├── src/
│   │   ├── main.rs              # エントリーポイント
│   │   ├── lib.rs               # ライブラリルート
│   │   ├── api/                 # 外部API クライアント
│   │   │   ├── twitch_api.rs
│   │   │   ├── youtube_api.rs
│   │   │   └── youtube_live_chat.rs
│   │   ├── collectors/          # データ収集モジュール
│   │   │   ├── mod.rs
│   │   │   ├── collector_trait.rs
│   │   │   ├── poller.rs        # 複数チャンネルのポーリング管理
│   │   │   ├── twitch.rs
│   │   │   └── youtube.rs
│   │   ├── database/            # DuckDB 操作
│   │   │   ├── mod.rs
│   │   │   ├── models.rs
│   │   │   ├── schema.rs
│   │   │   ├── writer.rs
│   │   │   ├── aggregation.rs   # 集計クエリ・分析用ビュー
│   │   │   └── utils.rs         # DBユーティリティ
│   │   ├── commands/            # Tauri コマンド
│   │   │   ├── channels.rs
│   │   │   ├── stats.rs
│   │   │   ├── chat.rs          # チャット関連コマンド
│   │   │   ├── export.rs
│   │   │   ├── config.rs
│   │   │   └── oauth.rs         # OAuth フロー制御
│   │   ├── config/              # 設定管理
│   │   │   ├── mod.rs
│   │   │   ├── credentials.rs   # APIクレデンシャル管理
│   │   │   └── settings.rs      # アプリ全体の設定
│   │   ├── websocket/           # WebSocket管理
│   │   │   ├── mod.rs
│   │   │   └── twitch_irc.rs
│   │   └── oauth/               # OAuth コールバックサーバー等
│   │       ├── mod.rs
│   │       ├── server.rs
│   │       ├── twitch.rs
│   │       └── youtube.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
└── package.json                 # フロントエンド依存関係
```

## 開発ガイドライン

### コーディング規約

#### Rust
- **エディション**: 2021
- **エラーハンドリング**: `Result<T, E>` を適切に使用
- **非同期処理**: `async/await` と Tokio を使用
- **モジュール構造**: 機能ごとにモジュールを分離
- **命名規則**: 
  - 関数・変数: `snake_case`
  - 型・構造体: `PascalCase`
  - 定数: `SCREAMING_SNAKE_CASE`

#### TypeScript/React
- **型安全性**: 可能な限り型を明示
- **コンポーネント**: 関数コンポーネント + Hooks
- **命名規則**:
  - コンポーネント: `PascalCase`
  - 関数・変数: `camelCase`
  - 型・インターフェース: `PascalCase`
- **ファイル名**: コンポーネントは `PascalCase.tsx`、その他は `camelCase.ts`

### データベース設計

#### 主要テーブル
- **channels**: 監視対象チャンネル設定
- **streams**: 配信基本情報
- **stream_stats**: 定期収集統計データ
- **chat_messages**: チャット全ログ

#### DuckDB 使用時の注意
- バッチインサートを活用してパフォーマンスを最適化
- トランザクションを適切に使用
- パラメータ化クエリでSQLインジェクション対策

### API統合

#### Twitch API
- **レート制限**: 800リクエスト/分
- **認証**: OAuth 2.0 Client Credentials Flow
- **エンドポイント**: Helix API を使用
- **IRC接続**: WebSocket経由で最大50チャンネル同時接続

#### YouTube API
- **クォータ制限**: 10,000ユニット/日
- **認証**: OAuth 2.0
- **Live Chat API**: ポーリング方式でチャット取得
- **スクレイピング**: 控えめな頻度（5分以上）で使用

### セキュリティ

#### APIトークン管理
- **保存場所**: OSネイティブのキーチェーン（keyring クレート）
- **メモリ**: 暗号化して保持
- **ログ**: トークンはマスキングして出力

#### データ保護
- ローカルストレージのみ使用
- 外部送信なし
- チャットログの匿名化オプション（将来実装）

### パフォーマンス最適化

#### バックエンド
- DuckDB のバッチインサート活用
- チャットメッセージの非同期書き込み
- 複数チャンネルの並行監視（Tokioタスク）

#### フロントエンド
- TanStack Query によるキャッシング
- 大量データ表示時は仮想スクロール検討
- コンポーネントの適切なメモ化

## 実装時の注意点

### 新機能追加時
1. **既存機能の確認**: 重複実装を避けるため、類似機能がないか確認
2. **型定義**: `src/types/index.ts` に型を追加
3. **エラーハンドリング**: 適切なエラーメッセージとログ出力
4. **テスト**: 可能な限りユニットテストを追加

### Tauriコマンド追加時
1. `src-tauri/src/commands/` に新しいモジュールを作成
2. `commands/mod.rs` でモジュールを公開
3. `main.rs` でコマンドを登録
4. フロントエンドから `invoke()` で呼び出し

### データベーススキーマ変更時
1. `database/schema.rs` でマイグレーション処理を実装
2. 既存データの互換性を考慮
3. バックアップ推奨

### UIコンポーネント追加時
1. `src/components/` 配下に適切なディレクトリを作成
2. Tailwind CSS クラスを使用
3. アクセシビリティを考慮（aria-label等）
4. エラーバウンダリでラップ

## デバッグ方法

### フロントエンド
```bash
# 開発サーバーのみ起動
npm run dev

# ブラウザの開発者ツールで確認
# Console, Network, React DevTools を活用
```

### バックエンド
```bash
# Tauri開発モード（フロントエンド + バックエンド）
npm run tauri dev

# Rust のログは標準出力に表示
# デバッグビルド: cargo build
# リリースビルド: cargo build --release
```

### データベース確認
- DuckDB CLI を使用して直接クエリ可能
- データベースファイル: `src-tauri/data/stream_stats.db` (想定)

## テスト戦略

### ユニットテスト
- **Rust**: `#[cfg(test)]` モジュールでテスト
- **TypeScript**: Vitest の導入を検討

### 統合テスト
- TauriコマンドのE2Eテスト
- データ収集フローのテスト

### 手動テスト
- 各プラットフォーム（Windows/macOS/Linux）での動作確認
- 大量データでのパフォーマンステスト

## ビルドとデプロイ

### 開発環境
```bash
# 依存関係インストール
npm install

# 開発モード起動
npm run tauri dev
```

### 本番ビルド
```bash
# リリースビルド
npm run tauri build
```

### ビルド時の注意
- **DuckDB bundled**: 初回ビルド時は5-10分かかる可能性
- **Windows**: Visual Studio Build Tools と CMake が必要
- **Linux/macOS**: build-essential (Linux) または Xcode (macOS) と CMake が必要

## 依存関係の追加

### Rust依存関係
`src-tauri/Cargo.toml` の `[dependencies]` セクションに追加

### フロントエンド依存関係
```bash
npm install <package-name>
# または
pnpm add <package-name>
```

## よくある問題と解決策

### DuckDB ビルドエラー
- **原因**: CMake または C++ ビルドツールが不足
- **解決**: 前提条件を確認（README.md参照）

### Tauriコマンドが呼び出せない
- **原因**: コマンドが `main.rs` で登録されていない
- **解決**: `tauri::Builder` でコマンドを登録

### APIレート制限エラー
- **原因**: リクエスト頻度が高すぎる
- **解決**: レート制限管理ロジックを確認・調整

### メモリ使用量が高い
- **原因**: チャットログの大量蓄積
- **解決**: バッチインサート、古いデータのアーカイブ

## 参考リソース

- [Tauri Documentation](https://v2.tauri.app/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [DuckDB Documentation](https://duckdb.org/docs/)
- [Twitch API Documentation](https://dev.twitch.tv/docs/api/)
- [YouTube Data API Documentation](https://developers.google.com/youtube/v3)

## ライセンス

MIT License

---

**注意**: このプロジェクトは個人利用目的です。各プラットフォーム（Twitch/YouTube）の利用規約を遵守してください。
