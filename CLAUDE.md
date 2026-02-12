# CLAUDE.md - Stream Stats Collector 開発ガイド

## 開発ガイドライン

### アーキテクチャ原則（重要）

#### 1. Repositoryパターンの徹底
**全てのデータベース操作はRepositoryを経由する**

❌ **禁止**: コマンドやその他のコードで直接SQLを実行
```rust
// NG
let count: i64 = conn.query_row("SELECT COUNT(*) FROM chat_messages", [], |row| row.get(0))?;
```

✅ **推奨**: Repositoryメソッドを使用
```rust
// OK
use crate::database::repositories::chat_message_repository::ChatMessageRepository;
let count = ChatMessageRepository::count_messages(&conn, None, None, None, None)?;
```

**理由**:
- SQLロジックの一元管理
- DuckDB特殊型（LIST, TIMESTAMP）の安全な取り扱い
- テスタビリティ向上
- 保守性向上

**配置場所**: `src-tauri/src/database/repositories/`

**既存Repository**:
- `ChannelRepository` - チャンネル関連クエリ（作成、取得、更新、削除、存在確認等）
- `ChatMessageRepository` - チャットメッセージ関連クエリ（カウント、集計、検索等）
- `StreamStatsRepository` - 配信統計関連クエリ（タイムライン、集計等）
- `AggregationRepository` - 複雑な集計クエリ（MW計算、エンゲージメント等）
- `mod.rs` - 公開インターフェース

#### 2. API呼び出しの共通化
**全てのTauri API呼び出しは共通レイヤーを経由する**

❌ **禁止**: コンポーネントで直接invoke
```typescript
// NG
const data = await invoke('get_realtime_chat_rate');
```

✅ **推奨**: 共通化されたAPIレイヤーを使用
```typescript
// OK
import * as statisticsApi from '../../api/statistics';
const data = await statisticsApi.getRealtimeChatRate();
```

**理由**:
- API呼び出しの一元管理
- 型安全性の向上
- エラーハンドリングの統一
- モックテストの容易化

**配置場所**: `src/api/`

**既存APIファイル**:
- `channels.ts` - チャンネル管理（追加、削除、更新、一覧取得）
- `config.ts` - 設定管理（トークン、OAuth設定、Twitch Device Code認証）
- `discovery.ts` - 自動発見（設定、検索、昇格）
- `sql.ts` - SQLクエリ（実行、テンプレート管理）
- `statistics.ts` - 統計・分析（分析結果、チャット統計、リアルタイム統計）
- `system.ts` - システム操作（Twitchコレクター再初期化、DB再作成、バックエンド状態確認、ウィンドウ表示）

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
- **Serdeシリアライゼーション**: デフォルトのsnake_caseを使用
  - ⚠️ `#[serde(rename_all = "camelCase")]`は使用しない（フロントエンドと不整合を起こすため）

#### TypeScript/React
- **型安全性**: 可能な限り型を明示
- **コンポーネント**: 関数コンポーネント + Hooks
- **命名規則**:
  - コンポーネント: `PascalCase`
  - 関数・変数: `camelCase`
  - 型・インターフェース: `PascalCase`
- **ファイル名**: コンポーネントは `PascalCase.tsx`、その他は `camelCase.ts`
- **API型定義**: Rustバックエンドとの互換性のため、Zodスキーマは`snake_case`を使用（例：`channel_id`, `minutes_watched`）
  - ⚠️ Rust構造体に`#[serde(rename_all = "camelCase")]`を追加しない（プロジェクト標準はsnake_case）

### データベース設計（DuckDB）

**主要テーブル:**
- `channels`: 監視対象チャンネル（platform, channel_id, enabled, is_auto_discovered等）
- `streams`: 配信情報（stream_id, title, category, started_at, ended_at）
- `stream_stats`: ポーリング統計（viewer_count, chat_rate_1min, collected_at）
- `chat_messages`: チャットログ（user_id, message, timestamp）
- `sql_templates`: SQLテンプレート

**重要:**
- バッチインサート、トランザクション、パラメータ化クエリを使用
- マイグレーションは`schema.rs`の`migrate_database_schema()`で実行
- SEQUENCEでID自動採番

### API統合

#### Twitch API
- **レート制限**: 800req/分
- **認証**: Device Code Grant Flow（Client Secret不要、リフレッシュトークン30日間有効）
- **フロー**: `start_twitch_device_auth` → ユーザーがブラウザで認証 → `poll_twitch_device_token`
- **IRC**: WebSocket実装済みだが未使用

#### YouTube API
- **クォータ制限**: 10,000units/日
- **認証**: ❌ OAuth未実装（oauth/youtube.rs存在せず）
- **現状**: 基本API連携のみ実装

### 自動発見機能（Auto Discovery）

指定ゲームカテゴリの人気配信を自動発見・監視。

**設定項目**: enabled, poll_interval, game_ids, min_viewer_count, max_channels, auto_promote

**フロー**: ゲームIDで検索 → フィルタリング → キャッシュ → `auto-discovery-update`イベント → auto_promote時は自動追加

**コマンド**: `get/save_auto_discovery_settings`, `toggle_auto_discovery`, `get_discovered_streams`, `promote_discovered_channel`

**注意**: ポーリング間隔60秒以上推奨（レート制限対策）

### セキュリティ
- **トークン管理**: OS keyring（Win: Credential Manager, macOS: Keychain, Linux: libsecret）
- **Twitch**: Device Code Flow（Client Secret不要）
- **データ**: ローカルのみ、外部送信なし、DB暗号化未実装

### パフォーマンス最適化
- バックエンド: DuckDBバッチインサート、非同期書き込み、Tokio並行処理
- フロントエンド: TanStack Queryキャッシング、メモ化

## 実装時の注意点

### 新機能追加時
1. **重複チェック**: `Grep`で既存機能を検索（特に`commands/`, `collectors/`, `database/`）
2. **型定義**: `src/types/index.ts`に追加
3. **エラー処理**: Rustは`Result<T, String>`、`AppLogger`でログ記録

### Tauriコマンド追加
1. `commands/<module>.rs`に実装
2. `commands/mod.rs`で公開
3. `lib.rs`の`tauri::generate_handler![]`に追加
4. 状態は`tauri::State`で取得（`DatabaseManager`, `ChannelPoller`等）
5. **重要**: 構造体を引数として受け取る場合：
   - Rust側: `#[serde(rename_all = "camelCase")]`属性で命名規則を統一
   - フロントエンド側: パラメータを構造体名でラップ（例: `{query: {...}}`）

### DBスキーマ変更
1. `schema.rs`の`migrate_database_schema()`に追加
2. `pragma_table_info()`で確認してから`ALTER TABLE`
3. バックアップ推奨
4. ⚠️ DuckDBはカラム削除等に制限あり

### DuckDB特殊型の取扱い（重要）
DuckDBの特殊型（`LIST`, `TIMESTAMP`等）をRustで扱う場合、SQLクエリ段階で型変換が必要：

**問題**: `LIST`型や`TIMESTAMP`型を直接SELECTすると、Rustのduckdbクレートで型エラーが発生
- `Invalid column type List`: 配列型を文字列として読み取れない
- `Invalid column type Timestamp`: タイムスタンプ型を文字列として読み取れない

**解決**: SQLクエリで`CAST`を使用して`VARCHAR`に変換
```sql
SELECT 
    CAST(badges AS VARCHAR) as badges,
    CAST(timestamp AS VARCHAR) as timestamp
FROM chat_messages
```

**対象カラム**:
- `chat_messages.badges`: `TEXT[]` → `VARCHAR`
- `chat_messages.timestamp`: `TIMESTAMP` → `VARCHAR`
- その他、LIST型やTIMESTAMP型のカラム全般

**ベストプラクティス**: Query Helperの使用（推奨）
```rust
use crate::database::query_helpers::chat_query;

// 個別カラムの取得
let sql = format!("SELECT {}", chat_query::badges_select("cm"));
// 生成: "SELECT CAST(cm.badges AS VARCHAR) as badges"

let sql = format!("SELECT {}", chat_query::timestamp_select("cm"));
// 生成: "SELECT CAST(cm.timestamp AS VARCHAR) as timestamp"

// 標準カラムセット（id, channel_id, stream_id, timestamp, platform, user_id, user_name, message, message_type, badges, badge_info）
let sql = format!("SELECT {} FROM chat_messages cm", 
                  chat_query::standard_columns("cm"));
```

**利点**:
- DuckDB型変換の仕様変更時、1箇所の修正で対応可能
- コードの可読性向上
- 型変換忘れによるバグを防止

### タイムスタンプの取り扱い（重要）

**問題**: DuckDBの`CURRENT_TIMESTAMP`はUTCを返すが、アプリケーションでは`Local::now()`でローカル時刻を保存している

**症状**: タイムスタンプを使った検索で結果が返らない（常に0件、時差分ずれる）

**原因**:
```rust
// 保存時: ローカルタイム（例: JST）
timestamp: Local::now().to_rfc3339()  // "2024-01-01T12:00:00+09:00"

// クエリ時: UTC
WHERE timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 minute'  // UTCとの比較
```

**解決策**: Rust側でローカル時刻を計算してパラメータとして渡す
```rust
// ✅ 正しい実装
let now = chrono::Local::now();
let one_minute_ago = now - chrono::Duration::minutes(1);
let one_minute_ago_str = one_minute_ago.to_rfc3339();

let sql = "SELECT COUNT(*) FROM chat_messages WHERE timestamp >= ?";
conn.query_row(sql, [&one_minute_ago_str], |row| row.get(0))
```

❌ **避けるべき**: `CURRENT_TIMESTAMP`を直接使用
```sql
-- NG: タイムゾーンが一致しない
SELECT COUNT(*) FROM chat_messages
WHERE timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 minute'
```

**ベストプラクティス**:
- タイムスタンプ保存は常に`Local::now().to_rfc3339()`
- タイムスタンプ比較は常にRust側で`chrono::Local`を使って計算
- SQLの`CURRENT_TIMESTAMP`は使用しない

### UIコンポーネント追加
- Tailwind CSS 4使用、`dark:`でダークモード対応
- TanStack Query（サーバー状態）+ Zustand（グローバル状態）
- `ErrorBoundary`でラップ

### Collector追加
1. `StreamCollector`トレイト実装
2. `ChannelPoller`に登録
3. エラーハンドリングで他チャンネルに影響させない

### 動的再初期化（認証トークン設定後）
**問題**: 起動時にトークンがない場合、Collectorが初期化されず、後からトークンを設定しても使用できない

**解決**:
1. **TwitchCollector**: 認証成功後、フロントエンドから`reinitialize_twitch_collector`を呼び出し
   - `oauth.rs`の`reinitialize_twitch_collector`コマンド
   - 設定再読み込み→新Collectorを作成→IRC初期化→ChannelPollerに登録（上書き）

2. **AutoDiscoveryPoller**: 設定変更時に自動再初期化
   - `discovery.rs`の`save_auto_discovery_settings`で最新のTwitchクライアントを取得
   - 既存ポーラーを停止→新ポーラー作成→必要なら開始

**実装例**（フロントエンド）:
```typescript
// TwitchAuthPanel.tsx: 認証成功イベント内
await invoke('reinitialize_twitch_collector');
```

## デバッグ方法

### フロントエンド
- **コマンド**: `bun run tauri dev`（ユーザーが実行）
- **ツール**: React Query Devtools, Zustand DevTools, Network Tab

### バックエンド
- **ログ**: `eprintln!()`または`AppLogger`（`logs.txt`）、アプリ内「ログ」タブで閲覧
- **注意**: DB初期化は`database-init-success`イベント待ち、レート制限は`get_twitch_rate_limit_status`確認

### データベース
- **パス**: `%APPDATA%\stream-stats-collector\stream_stats.db` (Win)
- **確認**: DuckDB CLIまたはアプリ内SQLビューア

### イベント
- **主要イベント**: `database-init-success/error`, `backend-ready`, `channel-stats-updated`, `discovered-streams-updated`
- **フロントエンド**: `listen('event-name', callback)`
- **デバッグ**: 
  - バックエンド: `app_handle.emit("event-name", ())`の戻り値を確認、`eprintln!`でログ出力
  - フロントエンド: `console.log`でイベント受信を確認、`backendReady`状態をログ出力

## テスト・ビルド・依存関係

### テスト（最小限実装）
- Rust: `#[cfg(test)]`（一部実装: database/models.rs）
- TypeScript: Vitest検討中
- E2E/統合テスト: 未実装

### コミット前チェック
**ユーザーからgit commit & push を実指示された際は、必ず以下のチェックを行う**

#### バックエンド（Rust）
```bash
# コードフォーマット確認
cargo fmt --check

# Clippy警告チェック
cargo clippy -- -D warnings

# コンパイルチェック
cargo check
```

すべてのコマンドが警告なしで成功することを確認してください。
また、各コマンドは2回掛けして確実に修正されていることを確認してください。

## よくある問題

| 問題 | 原因 | 解決 |
|:---|:---|:---|
| DuckDBビルドエラー | CMake/C++ツール不足 | README参照、VS Build Tools完全インストール |
| DB初期化エラー | スタックオーバーフロー | lib.rs内で512MB設定済み |
| コマンド呼出不可 | `invoke_handler`未登録 | `tauri::generate_handler![]`追加 |
| レート制限エラー | 頻度高すぎ | 間隔延長、`get_twitch_rate_limit_status`確認 |
| メモリ使用量高 | チャットログ蓄積 | バッチインサート済み、アーカイブ未実装 |
| Twitch認証失敗 | Client ID未設定/期限切れ | Device Code Flowで再認証 |
| チャット未記録 | IRC未統合 | 現在はチャットレートのみ |
| DuckDB型変換エラー | LIST/TIMESTAMP型を直接取得 | SQLで`CAST(column AS VARCHAR)` |
| Zodバリデーションエラー | バックエンドから`null`が返されるのに、フロントエンドで`nullable()`がない | バックエンドで`Option<>`を削除し、デフォルト値を設定 |
| マイグレーションでスタックオーバーフロー | `NOT NULL DEFAULT`を既存テーブルに追加 | `NOT NULL`を削除し、`DEFAULT`のみで追加、SELECT時に`COALESCE()`でデフォルト値を保証 |
| Tauriコマンド引数エラー | 構造体引数のラッピング不足 | フロントエンド: `{query: {...}}`でラップ |
| serdeデシリアライズ失敗 | 命名規則の不一致(camelCase/snake_case) | `#[serde(rename_all = "camelCase")]`追加 |
| タイムスタンプ比較で常に0 | `CURRENT_TIMESTAMP`(UTC)とLocal時刻の時差 | `chrono::Local::now()`で計算してパラメータ渡し |
| トークン設定後API使用不可 | 起動時のみCollector初期化 | 認証成功後`reinitialize_twitch_collector`実行 |
| 自動発見が無限ローディング | AutoDiscoveryPollerが古いクライアント使用 | `save_auto_discovery_settings`で再初期化 |
| チャンネル編集が反映されない | フロントエンドがAPI層を経由しない | `src/api/channels.ts`経由で呼び出し |
| 統計閲覧でデータ表示されない | `toISOString()`がUTC時刻を返し前日の日付に | ローカル日付取得関数で`getFullYear/getMonth/getDate`使用 |
| 配信者/ゲーム分析でフィールドがundefined | バックエンド構造体のcamelCase指定とフロントエンドのsnake_case期待の不一致 | バックエンドから`#[serde(rename_all = "camelCase")]`を削除してsnake_caseに統一 |
| 自動発見された配信が表示されない | `toggle_auto_discovery`がpollerを再初期化しない、またはイベント/クエリが実行されない | デバッグログで原因特定（`backend-ready`イベント、クエリ実行、キャッシュ初期化を確認） |

## 実装状況

### ✅ 実装済み
**バックエンド:**
- Twitch: Device Code Flow, Helix API, レート制限管理、動的Collector再初期化
- YouTube: 基本API連携、配信/チャット取得（❌ OAuth未実装）
- DuckDB: スキーマ、マイグレーション、バッチインサート、バックアップ
- 収集: ChannelPoller, TwitchCollector, YouTubeCollector, AutoDiscoveryPoller（設定変更時の動的再初期化対応）
- 統計: MW計算、配信者/ゲーム別、日次統計
- コマンド: チャンネル管理、統計取得、エクスポート、SQL実行、自動発見、`reinitialize_twitch_collector`
- 設定: keyring、JSON設定

**フロントエンド:**
- UI: Dashboard, Statistics, Settings, Export, SQL, Logs, MultiView
- 共通: Rechartsラッパー、ErrorBoundary、LoadingSpinner
- 状態: Zustand + TanStack Query
- その他: ダークモード、レスポンシブ

### ⚠️ 部分実装
- Twitch IRC WebSocket: コード存在、TwitchCollectorで未使用

### ❌ 未実装
**高優先度:**
- YouTube OAuth（oauth/youtube.rs不在）
- Twitch IRC統合

**中優先度:**
- チャット匿名化、包括的テスト、エラーリカバリー強化

**低優先度:**
- データアーカイブ、通知機能、プラグインシステム

## 参考リソース

- [Tauri Documentation](https://v2.tauri.app/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [DuckDB Documentation](https://duckdb.org/docs/)
- [Twitch API Documentation](https://dev.twitch.tv/docs/api/)
- [YouTube Data API Documentation](https://developers.google.com/youtube/v3)
