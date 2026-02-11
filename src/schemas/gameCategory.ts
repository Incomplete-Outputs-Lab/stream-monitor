import { z } from 'zod';

/**
 * Game category schema
 */
export const GameCategorySchema = z.object({
  gameId: z.string(),
  gameName: z.string(),
  boxArtUrl: z.string().optional(),
  lastUpdated: z.string().optional(),
});

/**
 * Upsert game category request schema
 */
export const UpsertGameCategoryRequestSchema = z.object({
  gameId: z.string(),
  gameName: z.string(),
  boxArtUrl: z.string().optional(),
});

// Export types
export type GameCategory = z.infer<typeof GameCategorySchema>;
export type UpsertGameCategoryRequest = z.infer<typeof UpsertGameCategoryRequestSchema>;
