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
