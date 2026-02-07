use crate::database::DatabaseManager;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub async fn get_database_info(app_handle: AppHandle) -> Result<DatabaseInfo, String> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    let path = db_manager.get_db_path();

    // Get main DB file size
    let mut total_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Add WAL file size if it exists
    let wal_path = path.with_extension("db.wal");
    if let Ok(metadata) = std::fs::metadata(&wal_path) {
        total_size += metadata.len();
    }

    // Add temporary files (*.tmp) - excluding already counted main DB and WAL files
    if let Some(parent_dir) = path.parent() {
        if let Ok(entries) = std::fs::read_dir(parent_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Only count .tmp files, not the main DB or WAL files (already counted above)
                    if file_name.ends_with(".tmp") {
                        if let Ok(metadata) = entry.metadata() {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }
    }

    Ok(DatabaseInfo {
        path: path.display().to_string(),
        size_bytes: total_size,
    })
}
