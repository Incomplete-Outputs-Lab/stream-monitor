use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// ブラウザでURLを開く
#[tauri::command]
pub async fn open_url(app_handle: AppHandle, url: String) -> Result<(), String> {
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}
