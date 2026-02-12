import { z } from 'zod';

/**
 * OAuth config schema
 */
export const OAuthConfigSchema = z.object({
  client_id: z.string().nullable(),
  client_secret: z.string().nullable(),
});

/**
 * Database init status schema
 */
export const DbInitStatusSchema = z.object({
  initialized: z.boolean(),
  message: z.string(),
});

/**
 * Device auth status schema
 */
export const DeviceAuthStatusSchema = z.object({
  user_code: z.string(),
  verification_uri: z.string(),
  expires_in: z.number(),
  device_code: z.string(),
  interval: z.number(),
});

/**
 * Collector status schema
 */
export const CollectorStatusSchema = z.object({
  channel_id: z.number(),
  channel_name: z.string(),
  platform: z.string(),
  is_running: z.boolean(),
  last_poll_at: z.string().optional(),
  last_success_at: z.string().optional(),
  last_error: z.string().optional(),
  poll_count: z.number(),
  error_count: z.number(),
});

/**
 * Twitch rate limit status schema
 */
export const TwitchRateLimitStatusSchema = z.object({
  points_used: z.number(),
  bucket_capacity: z.number(),
  points_remaining: z.number(),
  oldest_entry_expires_in_seconds: z.number().nullable(),
  usage_percent: z.number(),
  request_count: z.number(),
});

// Export types
export type OAuthConfig = z.infer<typeof OAuthConfigSchema>;
export type DbInitStatus = z.infer<typeof DbInitStatusSchema>;
export type DeviceAuthStatus = z.infer<typeof DeviceAuthStatusSchema>;
export type CollectorStatus = z.infer<typeof CollectorStatusSchema>;
export type TwitchRateLimitStatus = z.infer<typeof TwitchRateLimitStatusSchema>;
