import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  SqlQueryResultSchema,
  SqlTemplateSchema,
  SaveTemplateRequestSchema,
  TableInfoSchema,
  type SqlQueryResult,
  type SqlTemplate,
  type SaveTemplateRequest,
  type TableInfo,
} from '../schemas';

const DatabaseInfoSchema = z.object({
  path: z.string(),
  size_bytes: z.number(),
});

/**
 * SQLクエリを実行
 */
export const executeSqlQuery = async (query: string): Promise<SqlQueryResult> => {
  const result = await invoke<unknown>('execute_sql', { query });
  return SqlQueryResultSchema.parse(result);
};

/**
 * SQLテンプレート一覧を取得
 */
export const listSqlTemplates = async (): Promise<SqlTemplate[]> => {
  const result = await invoke<unknown>('list_sql_templates');
  return z.array(SqlTemplateSchema).parse(result);
};

/**
 * SQLテンプレートを保存
 */
export const saveSqlTemplate = async (request: SaveTemplateRequest): Promise<void> => {
  const validatedRequest = SaveTemplateRequestSchema.parse(request);
  await invoke('save_sql_template', { request: validatedRequest });
};

/**
 * SQLテンプレートを削除
 */
export const deleteSqlTemplate = async (id: number): Promise<void> => {
  await invoke('delete_sql_template', { id });
};

/**
 * データベース内のテーブル一覧を取得
 */
export const listDatabaseTables = async (): Promise<TableInfo[]> => {
  const result = await invoke<unknown>('list_database_tables');
  return z.array(TableInfoSchema).parse(result);
};

/**
 * データベース情報を取得
 */
export const getDatabaseInfo = async (): Promise<{ path: string; size_bytes: number }> => {
  const result = await invoke<unknown>('get_database_info');
  return DatabaseInfoSchema.parse(result);
};
