use tauri::Manager;

/// Show the main window and close the splash screen
#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    // Get the splash window and close it
    if let Some(splash_window) = app.get_webview_window("splash") {
        splash_window
            .close()
            .map_err(|e| format!("Failed to close splash window: {}", e))?;
    }

    // Get the main window and show it
    if let Some(main_window) = app.get_webview_window("main") {
        main_window
            .show()
            .map_err(|e| format!("Failed to show main window: {}", e))?;
        main_window
            .set_focus()
            .map_err(|e| format!("Failed to focus main window: {}", e))?;
    } else {
        return Err("Main window not found".to_string());
    }

    Ok(())
}
