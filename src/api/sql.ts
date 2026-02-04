import { invoke } from '@tauri-apps/api/core';
import type { SqlQueryResult, SqlTemplate, SaveTemplateRequest, TableInfo } from '../types';

/**
 * SQLクエリを実行
 */
export const executeSqlQuery = async (query: string): Promise<SqlQueryResult> => {
  return await invoke<SqlQueryResult>('execute_sql', { query });
};

/**
 * SQLテンプレート一覧を取得
 */
export const listSqlTemplates = async (): Promise<SqlTemplate[]> => {
  return await invoke<SqlTemplate[]>('list_sql_templates');
};

/**
 * SQLテンプレートを保存
 */
export const saveSqlTemplate = async (request: SaveTemplateRequest): Promise<void> => {
  return await invoke('save_sql_template', { request });
};

/**
 * SQLテンプレートを削除
 */
export const deleteSqlTemplate = async (id: number): Promise<void> => {
  return await invoke('delete_sql_template', { id });
};

/**
 * データベース内のテーブル一覧を取得
 */
export const listDatabaseTables = async (): Promise<TableInfo[]> => {
  return await invoke<TableInfo[]>('list_database_tables');
};

/**
 * データベース情報を取得
 */
export const getDatabaseInfo = async (): Promise<{ path: string; size_bytes: number }> => {
  return await invoke<{ path: string; size_bytes: number }>('get_database_info');
};
