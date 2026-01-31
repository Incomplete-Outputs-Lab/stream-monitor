use crate::database::DatabaseManager;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn create_database_backup(app_handle: AppHandle) -> Result<String, String> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    let backup_path = db_manager.create_backup().map_err(|e| e.to_string())?;
    Ok(backup_path.display().to_string())
}

#[tauri::command]
pub async fn get_database_info(app_handle: AppHandle) -> Result<DatabaseInfo, String> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    let path = db_manager.get_db_path();

    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    Ok(DatabaseInfo {
        path: path.display().to_string(),
        size_bytes: size,
    })
}
