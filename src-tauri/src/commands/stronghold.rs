use tauri::{AppHandle, Manager};

#[derive(serde::Serialize)]
pub struct VaultStatus {
    initialized: bool,
    path: String,
}

/// Check if vault is initialized
#[tauri::command]
pub async fn check_vault_initialized(app: AppHandle) -> Result<VaultStatus, String> {
    let vault_path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("vault.hold");

    Ok(VaultStatus {
        initialized: vault_path.exists(),
        path: vault_path.to_string_lossy().to_string(),
    })
}

/// Initialize a new vault with password
#[tauri::command]
pub async fn initialize_vault(app: AppHandle, _password: String) -> Result<(), String> {
    let vault_path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("vault.hold");

    eprintln!("[Stronghold] Initializing vault at: {:?}", vault_path);

    // Initialize the stronghold through the plugin
    // This will be called from JavaScript, which handles the initialization
    Ok(())
}
