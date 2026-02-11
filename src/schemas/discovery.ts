import { z } from 'zod';

/**
 * Twitch game schema
 */
export const TwitchGameSchema = z.object({
  id: z.string(),
  name: z.string(),
  box_art_url: z.string(),
});

/**
 * Selected game schema
 */
export const SelectedGameSchema = z.object({
  id: z.string(),
  name: z.string(),
});

/**
 * Discovered stream info schema
 */
export const DiscoveredStreamInfoSchema = z.object({
  id: z.number(),
  twitch_user_id: z.number(),
  channel_id: z.string(),
  channel_name: z.string(),
  display_name: z.string().optional(),
  profile_image_url: z.string().optional(),
  discovered_at: z.string().optional(),
  title: z.string().optional(),
  category: z.string().optional(),
  viewer_count: z.number().optional(),
  follower_count: z.number().optional(),
  broadcaster_type: z.string().optional(),
});

/**
 * Auto discovery filters schema
 */
export const AutoDiscoveryFiltersSchema = z.object({
  game_ids: z.array(z.string()),
  languages: z.array(z.string()),
  min_viewers: z.number().optional(),
});

/**
 * Auto discovery settings schema
 */
export const AutoDiscoverySettingsSchema = z.object({
  enabled: z.boolean(),
  poll_interval: z.number(),
  max_streams: z.number(),
  filters: AutoDiscoveryFiltersSchema,
});

// Export types
export type TwitchGame = z.infer<typeof TwitchGameSchema>;
export type SelectedGame = z.infer<typeof SelectedGameSchema>;
export type DiscoveredStreamInfo = z.infer<typeof DiscoveredStreamInfoSchema>;
export type AutoDiscoveryFilters = z.infer<typeof AutoDiscoveryFiltersSchema>;
export type AutoDiscoverySettings = z.infer<typeof AutoDiscoverySettingsSchema>;
