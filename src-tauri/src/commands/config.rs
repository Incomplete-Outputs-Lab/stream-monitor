use crate::config::keyring_store::KeyringStore;
use crate::config::settings::SettingsManager;
use crate::constants::database as db_constants;
use crate::constants::youtube;
use crate::database::{get_connection, DatabaseManager};
use crate::error::ResultExt;
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
    pub repository_url: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>, // 注意: レスポンスではマスキングされる
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfigResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn save_token(
    app_handle: AppHandle,
    platform: String,
    token: String,
) -> Result<TokenResponse, String> {
    KeyringStore::save_token_with_app(&app_handle, &platform, &token)
        .config_context("save token")
        .map_err(|e| e.to_string())?;

    Ok(TokenResponse {
        success: true,
        message: "Token saved successfully".to_string(),
    })
}

#[tauri::command]
pub async fn delete_token(
    app_handle: AppHandle,
    platform: String,
) -> Result<TokenResponse, String> {
    KeyringStore::delete_token_with_app(&app_handle, &platform)
        .config_context("delete token")
        .map_err(|e| e.to_string())?;

    Ok(TokenResponse {
        success: true,
        message: "Token deleted successfully".to_string(),
    })
}

#[tauri::command]
pub async fn verify_token(app_handle: AppHandle, platform: String) -> Result<bool, String> {
    // TODO: 実際のAPIを呼び出してトークンを検証する
    // ここでは一旦、トークンが存在するかどうかのみを確認
    Ok(KeyringStore::has_token_with_app(&app_handle, &platform))
}

#[tauri::command]
pub async fn get_oauth_config(
    app_handle: AppHandle,
    platform: String,
) -> Result<OAuthConfig, String> {
    // 設定ファイルからClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let client_id = match platform.as_str() {
        p if p == db_constants::PLATFORM_TWITCH => settings.twitch.client_id,
        p if p == youtube::PLATFORM_NAME => settings.youtube.client_id,
        _ => return Err(format!("Unsupported platform: {}", platform)),
    };

    // YouTubeの場合のみKeyringからClient Secretを取得（TwitchはDevice Code Flowでクライアント Secret不要）
    let client_secret = if platform == youtube::PLATFORM_NAME {
        KeyringStore::get_oauth_secret_with_app(&app_handle, &platform).ok()
    } else {
        None
    };

    Ok(OAuthConfig {
        client_id,
        client_secret,
    })
}

#[tauri::command]
pub async fn save_oauth_config(
    app_handle: AppHandle,
    platform: String,
    client_id: String,
    client_secret: Option<String>,
) -> Result<OAuthConfigResponse, String> {
    // 現在の設定を読み込み
    let mut settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    // Client IDを設定ファイルに保存
    match platform.as_str() {
        p if p == db_constants::PLATFORM_TWITCH => {
            settings.twitch.client_id = Some(client_id);
        }
        p if p == youtube::PLATFORM_NAME => {
            settings.youtube.client_id = Some(client_id);
        }
        _ => return Err(format!("Unsupported platform: {}", platform)),
    }

    // 設定ファイルを保存
    SettingsManager::save_settings(&app_handle, &settings)
        .config_context("save settings")
        .map_err(|e| e.to_string())?;

    // YouTubeの場合のみClient SecretをKeyringに保存（TwitchはDevice Code FlowでClient Secret不要）
    if platform == youtube::PLATFORM_NAME {
        if let Some(secret) = client_secret {
            if !secret.trim().is_empty() {
                KeyringStore::save_oauth_secret_with_app(&app_handle, &platform, &secret)
                    .config_context("save OAuth secret")
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(OAuthConfigResponse {
        success: true,
        message: format!("OAuth configuration saved for {}", platform),
    })
}

#[tauri::command]
pub async fn delete_oauth_config(
    app_handle: AppHandle,
    platform: String,
) -> Result<OAuthConfigResponse, String> {
    // 現在の設定を読み込み
    let mut settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    // Client IDを設定ファイルから削除
    match platform.as_str() {
        p if p == db_constants::PLATFORM_TWITCH => {
            settings.twitch.client_id = None;
        }
        p if p == youtube::PLATFORM_NAME => {
            settings.youtube.client_id = None;
        }
        _ => return Err(format!("Unsupported platform: {}", platform)),
    }

    // 設定ファイルを保存
    SettingsManager::save_settings(&app_handle, &settings)
        .config_context("save settings")
        .map_err(|e| e.to_string())?;

    // YouTubeの場合のみClient SecretをKeyringから削除（TwitchはDevice Code FlowでClient Secret不要）
    if platform == youtube::PLATFORM_NAME {
        KeyringStore::delete_token_with_app(&app_handle, &format!("{}_oauth_secret", platform))
            .config_context("delete OAuth secret")
            .map_err(|e| e.to_string())?;
    }

    Ok(OAuthConfigResponse {
        success: true,
        message: format!("OAuth configuration deleted for {}", platform),
    })
}

#[tauri::command]
pub async fn has_oauth_config(app_handle: AppHandle, platform: String) -> Result<bool, String> {
    // 設定ファイルからClient IDの存在を確認
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let has_client_id = match platform.as_str() {
        p if p == db_constants::PLATFORM_TWITCH => settings.twitch.client_id.is_some(),
        p if p == youtube::PLATFORM_NAME => settings.youtube.client_id.is_some(),
        _ => return Err(format!("Unsupported platform: {}", platform)),
    };

    // TwitchはDevice Code Flowを使用するため、Client IDのみで十分
    // YouTubeの場合はClient Secretも必要
    match platform.as_str() {
        p if p == db_constants::PLATFORM_TWITCH => Ok(has_client_id),
        p if p == youtube::PLATFORM_NAME => {
            let has_client_secret =
                KeyringStore::get_oauth_secret_with_app(&app_handle, &platform).is_ok();
            Ok(has_client_id && has_client_secret)
        }
        _ => Err(format!("Unsupported platform: {}", platform)),
    }
}

#[command]
pub async fn get_build_info() -> Result<BuildInfo, String> {
    Ok(BuildInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_hash: option_env!("GIT_COMMIT_HASH").map(|s| s.to_string()),
        build_date: option_env!("BUILD_DATE").map(|s| s.to_string()),
        developer: "未完成成果物研究所".to_string(),
        repository_url: "https://github.com/Incomplete-Outputs-Lab/stream-monitor".to_string(),
    })
}

#[tauri::command]
pub async fn recreate_database(app_handle: AppHandle) -> Result<DatabaseInitResult, String> {
    eprintln!("[Database Recreate] Starting database recreation...");

    // データベースパスを取得
    let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        std::fs::create_dir_all(&app_data_dir)
            .io_context("create app data directory")
            .map_err(|e| e.to_string())?;
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
    match get_connection(&app_handle).await {
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
                    eprintln!(
                        "[Database Recreate] Schema initialization failed: {}",
                        schema_err
                    );
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
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<DbInitStatus, String> {
    // データベース接続を試行して初期化状態を確認
    match db_manager.get_connection().await {
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
