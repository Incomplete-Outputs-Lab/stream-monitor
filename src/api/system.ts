/**
 * システム関連API
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * Twitchコレクターを再初期化
 */
export async function reinitializeTwitchCollector(): Promise<void> {
  await invoke("reinitialize_twitch_collector");
}

/**
 * データベースを再作成
 */
export async function recreateDatabase(): Promise<any> {
  return await invoke("recreate_database");
}

/**
 * メインウィンドウを表示
 */
export async function showMainWindow(): Promise<void> {
  await invoke("show_main_window");
}

/**
 * バックエンドの準備状態を確認
 */
export async function isBackendReady(): Promise<boolean> {
  return await invoke("is_backend_ready");
}

export interface BuildInfo {
  version: string;
  commit_hash?: string;
  build_date?: string;
  developer: string;
  repository_url: string;
}

/**
 * ビルド情報を取得
 */
export async function getBuildInfo(): Promise<BuildInfo> {
  return await invoke<BuildInfo>("get_build_info");
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export interface GetLogsQuery {
  level?: string;
  search?: string;
  limit?: number;
}

/**
 * ログ一覧を取得
 */
export async function getLogs(query: GetLogsQuery): Promise<LogEntry[]> {
  return await invoke<LogEntry[]>("get_logs", { query });
}
