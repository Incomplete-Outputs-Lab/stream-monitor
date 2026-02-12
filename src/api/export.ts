import { invoke } from '@tauri-apps/api/core';
import type { ExportQuery } from '../schemas';

/**
 * エクスポートプレビューを取得
 */
export async function previewExportData(
  query: ExportQuery,
  maxRows?: number
): Promise<string> {
  return await invoke<string>('preview_export_data', {
    query,
    max_rows: maxRows,
  });
}

/**
 * 区切り形式でエクスポート
 */
export async function exportToDelimited(
  query: ExportQuery,
  filePath: string,
  includeBom?: boolean
): Promise<string> {
  return await invoke<string>('export_to_delimited', {
    query,
    filePath,
    includeBom,
  });
}
