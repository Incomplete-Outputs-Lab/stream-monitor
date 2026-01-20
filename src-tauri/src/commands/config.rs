use crate::config::credentials::CredentialManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn save_token(platform: String, token: String) -> Result<TokenResponse, String> {
    CredentialManager::save_token(&platform, &token)
        .map_err(|e| format!("Failed to save token: {}", e))?;

    Ok(TokenResponse {
        success: true,
        message: "Token saved successfully".to_string(),
    })
}

#[tauri::command]
pub async fn get_token(platform: String) -> Result<String, String> {
    CredentialManager::get_token(&platform).map_err(|e| format!("Failed to get token: {}", e))
}

#[tauri::command]
pub async fn delete_token(platform: String) -> Result<TokenResponse, String> {
    CredentialManager::delete_token(&platform)
        .map_err(|e| format!("Failed to delete token: {}", e))?;

    Ok(TokenResponse {
        success: true,
        message: "Token deleted successfully".to_string(),
    })
}

#[tauri::command]
pub async fn has_token(platform: String) -> Result<bool, String> {
    Ok(CredentialManager::has_token(&platform))
}

#[tauri::command]
pub async fn verify_token(platform: String) -> Result<bool, String> {
    // TODO: 実際のAPIを呼び出してトークンを検証する
    // ここでは一旦、トークンが存在するかどうかのみを確認
    Ok(CredentialManager::has_token(&platform))
}
