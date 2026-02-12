use crate::database::{
    models::GameCategory, repositories::GameCategoryRepository, DatabaseManager,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// ゲームカテゴリの挿入/更新リクエスト
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertGameCategoryRequest {
    pub game_id: String,
    pub game_name: String,
    pub box_art_url: Option<String>,
}

/// 全ゲームカテゴリを取得
#[tauri::command]
pub async fn get_game_categories(
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<GameCategory>, String> {
    db_manager
        .with_connection(|conn| {
            GameCategoryRepository::get_all_categories(conn)
                .map_err(|e| format!("Failed to get game categories: {}", e))
        })
        .await
}

/// IDでゲームカテゴリを取得
#[tauri::command]
pub async fn get_game_category(
    db_manager: State<'_, DatabaseManager>,
    game_id: String,
) -> Result<Option<GameCategory>, String> {
    db_manager
        .with_connection(|conn| {
            GameCategoryRepository::get_category_by_id(conn, &game_id)
                .map_err(|e| format!("Failed to get game category: {}", e))
        })
        .await
}

/// ゲームカテゴリを挿入または更新
#[tauri::command]
pub async fn upsert_game_category(
    db_manager: State<'_, DatabaseManager>,
    request: UpsertGameCategoryRequest,
) -> Result<(), String> {
    db_manager
        .with_connection(|conn| {
            GameCategoryRepository::upsert_category(
                conn,
                &request.game_id,
                &request.game_name,
                request.box_art_url.as_deref(),
            )
            .map_err(|e| format!("Failed to upsert game category: {}", e))
        })
        .await
}

/// ゲームカテゴリを削除
#[tauri::command]
pub async fn delete_game_category(
    db_manager: State<'_, DatabaseManager>,
    game_id: String,
) -> Result<(), String> {
    db_manager
        .with_connection(|conn| {
            GameCategoryRepository::delete_category(conn, &game_id)
                .map_err(|e| format!("Failed to delete game category: {}", e))
        })
        .await
}

/// ゲームカテゴリを検索
#[tauri::command]
pub async fn search_game_categories(
    db_manager: State<'_, DatabaseManager>,
    query: String,
) -> Result<Vec<GameCategory>, String> {
    db_manager
        .with_connection(|conn| {
            GameCategoryRepository::search_categories(conn, &query)
                .map_err(|e| format!("Failed to search game categories: {}", e))
        })
        .await
}
