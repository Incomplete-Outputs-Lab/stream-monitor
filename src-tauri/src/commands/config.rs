use crate::config::credentials::CredentialManager;
use crate::database::{get_connection, DatabaseManager};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub commit_hash: Option<String>,
    pub build_date: Option<String>,
    pub developer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbInitStatus {
    pub initialized: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInitResult {
    pub success: bool,
    pub message: String,
    pub backup_created: Option<String>,
    pub error_type: Option<String>,
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

#[command]
pub async fn get_build_info() -> Result<BuildInfo, String> {
    Ok(BuildInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_hash: option_env!("GIT_COMMIT_HASH").map(|s| s.to_string()),
        build_date: option_env!("BUILD_DATE").map(|s| s.to_string()),
        developer: "Flowing".to_string(),
    })
}

#[tauri::command]
pub async fn initialize_database(app_handle: AppHandle) -> Result<DatabaseInitResult, String> {
    eprintln!("[Database Init] Starting database initialization...");

    match get_connection(&app_handle) {
        Ok(conn) => {
            // 明示的にスキーマ初期化を実行
            match crate::database::schema::init_database(&conn) {
                Ok(_) => {
                    eprintln!("[Database Init] Database initialized successfully");
                    Ok(DatabaseInitResult {
                        success: true,
                        message: "Database initialized successfully".to_string(),
                        backup_created: None,
                        error_type: None,
                    })
                }
                Err(schema_err) => {
                    eprintln!("[Database Init] Schema initialization failed: {}", schema_err);
                    Ok(DatabaseInitResult {
                        success: false,
                        message: format!("Schema initialization failed: {}", schema_err),
                        backup_created: None,
                        error_type: Some("schema_error".to_string()),
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("[Database Init] Database initialization failed: {}", e);

            // エラーの種類を判定
            let error_type = if e.to_string().contains("Failed to initialize database schema") {
                "schema_error"
            } else if e.to_string().contains("Failed to open database") {
                "connection_error"
            } else {
                "unknown_error"
            };

            Ok(DatabaseInitResult {
                success: false,
                message: format!("Database initialization failed: {}", e),
                backup_created: None, // バックアップはまだ作成していない
                error_type: Some(error_type.to_string()),
            })
        }
    }
}

#[tauri::command]
pub async fn recreate_database(app_handle: AppHandle) -> Result<DatabaseInitResult, String> {
    eprintln!("[Database Recreate] Starting database recreation...");

    // データベースパスを取得
    let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        app_data_dir.join("stream_stats.db")
    } else {
        return Err("Failed to get app data directory".to_string());
    };

    // 既存のデータベースファイルが存在する場合、バックアップを作成
    let backup_path = if db_path.exists() {
        let backup_path = format!("{}.backup", db_path.display());
        match std::fs::rename(&db_path, &backup_path) {
            Ok(_) => {
                eprintln!("[Database Recreate] Created backup: {}", backup_path);
                Some(backup_path)
            }
            Err(e) => {
                eprintln!("[Database Recreate] Failed to create backup: {}", e);
                return Err(format!("Failed to create backup: {}", e));
            }
        }
    } else {
        None
    };

    // 新しいデータベース接続を取得（これで新しいDBが作成される）
    match get_connection(&app_handle) {
        Ok(conn) => {
            // 明示的にスキーマ初期化を実行
            match crate::database::schema::init_database(&conn) {
                Ok(_) => {
                    eprintln!("[Database Recreate] Database recreated successfully");
                    Ok(DatabaseInitResult {
                        success: true,
                        message: "Database recreated successfully".to_string(),
                        backup_created: backup_path,
                        error_type: None,
                    })
                }
                Err(schema_err) => {
                    eprintln!("[Database Recreate] Schema initialization failed: {}", schema_err);
                    Ok(DatabaseInitResult {
                        success: false,
                        message: format!("Schema initialization failed: {}", schema_err),
                        backup_created: backup_path,
                        error_type: Some("schema_error".to_string()),
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("[Database Recreate] Database recreation failed: {}", e);
            Ok(DatabaseInitResult {
                success: false,
                message: format!("Database recreation failed: {}", e),
                backup_created: backup_path, // バックアップは作成できたかもしれない
                error_type: Some("recreation_error".to_string()),
            })
        }
    }
}

#[command]
pub async fn get_database_init_status(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>
) -> Result<DbInitStatus, String> {
    // データベース接続を試行して初期化状態を確認
    match db_manager.get_connection() {
        Ok(_) => {
            // 接続成功 = 初期化済み
            Ok(DbInitStatus {
                initialized: true,
                message: "Database is initialized and ready".to_string(),
            })
        }
        Err(e) => {
            // 接続失敗 = 未初期化またはエラー
            Ok(DbInitStatus {
                initialized: false,
                message: format!("Database initialization in progress or failed: {}", e),
            })
        }
    }
}
