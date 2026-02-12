import { z } from 'zod';

/**
 * Export query schema
 */
export const ExportQuerySchema = z.object({
  channel_id: z.number(),
  start_time: z.string().optional(),
  end_time: z.string().optional(),
  aggregation: z.string().optional(),
  include_chat: z.boolean().optional(),
  delimiter: z.string().optional(),
});

// Export types
export type ExportQuery = z.infer<typeof ExportQuerySchema>;
