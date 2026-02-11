import { z } from 'zod';

/**
 * Platform enum
 */
export const PlatformSchema = z.enum(['twitch', 'youtube']);

/**
 * Channel schema (database model)
 * Note: id, created_at, updated_at are optional when creating but required when reading from DB
 */
export const ChannelSchema = z.object({
  id: z.number().optional(),
  platform: PlatformSchema,
  channel_id: z.string(),
  channel_name: z.string(),
  display_name: z.string().optional(),
  profile_image_url: z.string().optional(),
  enabled: z.boolean(),
  poll_interval: z.number(),
  follower_count: z.number().optional(),
  broadcaster_type: z.string().optional(),
  view_count: z.number().optional(),
  is_auto_discovered: z.boolean().optional(),
  discovered_at: z.string().optional(),
  twitch_user_id: z.number().optional(),
  created_at: z.string().optional(),
  updated_at: z.string().optional(),
});

/**
 * Channel with stats schema (includes live status)
 */
export const ChannelWithStatsSchema = ChannelSchema.extend({
  is_live: z.boolean(),
  current_viewers: z.number().optional(),
  current_title: z.string().optional(),
});

/**
 * Add channel request schema
 */
export const AddChannelRequestSchema = z.object({
  platform: z.string(),
  channel_id: z.string(),
  channel_name: z.string(),
  poll_interval: z.number(),
});

/**
 * Update channel request schema
 */
export const UpdateChannelRequestSchema = z.object({
  id: z.number(),
  channel_name: z.string().optional(),
  poll_interval: z.number().optional(),
  enabled: z.boolean().optional(),
});

// Export types
export type Platform = z.infer<typeof PlatformSchema>;
export type Channel = z.infer<typeof ChannelSchema>;
export type ChannelWithStats = z.infer<typeof ChannelWithStatsSchema>;
export type AddChannelRequest = z.infer<typeof AddChannelRequestSchema>;
export type UpdateChannelRequest = z.infer<typeof UpdateChannelRequestSchema>;
