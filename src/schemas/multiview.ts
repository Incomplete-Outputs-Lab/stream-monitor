import { z } from 'zod';

/**
 * Multiview event flags schema
 */
export const MultiviewEventFlagsSchema = z.object({
  viewer_spike: z.boolean(),
  chat_spike: z.boolean(),
  category_change: z.boolean(),
});

/**
 * Multiview channel stats schema (snake_case for backend compatibility)
 */
export const MultiviewChannelStatsSchema = z.object({
  channel_id: z.number(),
  channel_name: z.string(),
  stream_id: z.number().nullable(),
  is_live: z.boolean(),
  viewer_count: z.number().nullable(),
  chat_rate_1min: z.number(),
  chat_rate_5s: z.number(),
  category: z.string().nullable(),
  title: z.string().nullable(),
  collected_at: z.string().nullable(),
  event_flags: MultiviewEventFlagsSchema,
});

export type MultiviewEventFlags = z.infer<typeof MultiviewEventFlagsSchema>;
export type MultiviewChannelStats = z.infer<typeof MultiviewChannelStatsSchema>;
