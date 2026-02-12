import { z } from 'zod';

/**
 * SQL query result schema
 */
export const SqlQueryResultSchema = z.object({
  columns: z.array(z.string()),
  rows: z.array(z.array(z.unknown())),
  affected_rows: z.number(),
  execution_time_ms: z.number(),
});

/**
 * SQL template schema
 */
export const SqlTemplateSchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string(),
  query: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
});

/**
 * Save template request schema
 */
export const SaveTemplateRequestSchema = z.object({
  id: z.number(), // 0 = 新規作成
  name: z.string(),
  description: z.string(),
  query: z.string(),
});

/**
 * Table info schema
 */
export const TableInfoSchema = z.object({
  table_name: z.string(),
  column_count: z.number(),
});

// Export types
export type SqlQueryResult = z.infer<typeof SqlQueryResultSchema>;
export type SqlTemplate = z.infer<typeof SqlTemplateSchema>;
export type SaveTemplateRequest = z.infer<typeof SaveTemplateRequestSchema>;
export type TableInfo = z.infer<typeof TableInfoSchema>;
