import { z } from 'zod';

/**
 * Platform enum
 */
export const PlatformSchema = z.enum(['twitch', 'youtube']);

/**
 * Channel schema (database model)
 */
export const ChannelSchema = z.object({
  id: z.number(),
  platform: PlatformSchema,
  channel_id: z.string(),
  channel_name: z.string(),
  display_name: z.string(),
  profile_image_url: z.string(),
  enabled: z.boolean(),
  poll_interval: z.number(),
  follower_count: z.number(),
  broadcaster_type: z.string(),
  view_count: z.number(),
  is_auto_discovered: z.boolean(),
  discovered_at: z.string(),
  twitch_user_id: z.number().optional(),
  created_at: z.string(),
  updated_at: z.string(),
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
  twitch_user_id: z.number().optional(),
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
