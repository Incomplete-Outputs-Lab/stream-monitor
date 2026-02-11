import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import {
  GameCategorySchema,
  UpsertGameCategoryRequestSchema,
  type GameCategory,
  type UpsertGameCategoryRequest,
} from '../schemas';

/**
 * 全ゲームカテゴリを取得
 */
export async function getGameCategories(): Promise<GameCategory[]> {
  const result = await invoke<unknown>('get_game_categories');
  return z.array(GameCategorySchema).parse(result);
}

/**
 * IDでゲームカテゴリを取得
 */
export async function getGameCategory(gameId: string): Promise<GameCategory | null> {
  const result = await invoke<unknown>('get_game_category', { gameId });
  return result === null ? null : GameCategorySchema.parse(result);
}

/**
 * ゲームカテゴリを挿入または更新
 */
export async function upsertGameCategory(request: UpsertGameCategoryRequest): Promise<void> {
  const validatedRequest = UpsertGameCategoryRequestSchema.parse(request);
  await invoke('upsert_game_category', { request: validatedRequest });
}

/**
 * ゲームカテゴリを削除
 */
export async function deleteGameCategory(gameId: string): Promise<void> {
  await invoke('delete_game_category', { gameId });
}

/**
 * ゲームカテゴリを検索
 */
export async function searchGameCategories(query: string): Promise<GameCategory[]> {
  const result = await invoke<unknown>('search_game_categories', { query });
  return z.array(GameCategorySchema).parse(result);
}
