import { invoke } from '@tauri-apps/api/core';
import type { GameCategory, UpsertGameCategoryRequest } from '../types';

/**
 * 全ゲームカテゴリを取得
 */
export async function getGameCategories(): Promise<GameCategory[]> {
  return await invoke<GameCategory[]>('get_game_categories');
}

/**
 * IDでゲームカテゴリを取得
 */
export async function getGameCategory(gameId: string): Promise<GameCategory | null> {
  return await invoke<GameCategory | null>('get_game_category', { gameId });
}

/**
 * ゲームカテゴリを挿入または更新
 */
export async function upsertGameCategory(request: UpsertGameCategoryRequest): Promise<void> {
  await invoke('upsert_game_category', { request });
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
  return await invoke<GameCategory[]>('search_game_categories', { query });
}
