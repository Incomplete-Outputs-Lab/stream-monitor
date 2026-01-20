pub mod aggregation;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Manager};

// データベース初期化のグローバル状態（一度だけ初期化）
static DB_INIT_STATE: OnceLock<Arc<Mutex<Result<(), String>>>> = OnceLock::new();
static DB_INIT_FLAG: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

pub fn get_connection(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    // データベース初期化の状態を確認
    let init_state = DB_INIT_STATE.get_or_init(|| Arc::new(Mutex::new(Ok(()))));

    // すでに初期化が失敗している場合は、すぐにエラーを返す
    {
        let state = init_state.lock().unwrap();
        if let Err(e) = &*state {
            return Err(format!("Database initialization failed previously: {}", e).into());
        }
    }

    // Tauriが完全に立ち上がるまで少し待つ（初回のみ）
    let init_flag = DB_INIT_FLAG.get_or_init(|| Arc::new(Mutex::new(false)));
    let mut initialized = init_flag.lock().unwrap();

    if !*initialized {
        eprintln!("Waiting for Tauri to fully initialize before database connection...");
        std::thread::sleep(std::time::Duration::from_millis(2000)); // 2秒待つ
        *initialized = true;
    }

    // Tauri 2.xでの適切なデータディレクトリ取得
    let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        app_data_dir.join("stream_stats.db")
    } else {
        // フォールバック：現在のディレクトリを使用
        eprintln!("Warning: Using current directory for database (app_data_dir not available)");
        let db_dir = std::env::current_dir()
            .or_else(|_| std::path::PathBuf::from(".").canonicalize())
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        std::fs::create_dir_all(&db_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        db_dir.join("stream_stats.db")
    };

    eprintln!("Opening DuckDB connection at: {}", db_path.display());
    eprintln!("Database file exists: {}", db_path.exists());

    // Use a thread with larger stack for DuckDB connection
    let db_path_clone = db_path.clone();
    let conn_result = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024) // 512MB stack
        .spawn(move || {
            eprintln!("[Thread] Opening DuckDB connection...");
            let result = Connection::open(&db_path_clone);
            eprintln!("[Thread] Connection result obtained");
            result
        })
        .expect("Failed to spawn thread for DuckDB connection")
        .join();

    eprintln!("Connection thread joined");
    let conn = match conn_result {
        Ok(Ok(c)) => {
            eprintln!("DuckDB connection opened successfully");
            c
        }
        Ok(Err(e)) => {
            return Err(format!("Failed to open database at {}: {}", db_path.display(), e).into());
        }
        Err(_) => {
            return Err("Thread panicked while opening database".into());
        }
    };

    // DuckDBのメモリ設定（2GBに設定）
    eprintln!("Setting DuckDB memory limit...");
    if let Err(e) = conn.execute("PRAGMA memory_limit='2GB'", []) {
        eprintln!("Warning: Failed to set memory limit: {}", e);
    }
    eprintln!("Memory limit set");
    if let Err(e) = conn.execute("PRAGMA threads=4", []) {
        eprintln!("Warning: Failed to set thread count: {}", e);
    }
    eprintln!("Thread count set");

    // 初回接続時のみスキーマを初期化
    eprintln!("About to call schema::init_database...");
    match schema::init_database(&conn) {
        Ok(_) => {
            eprintln!("Database schema initialized successfully");
            // 初期化成功を記録
            let mut state = DB_INIT_STATE.get().unwrap().lock().unwrap();
            *state = Ok(());
        }
        Err(e) => {
            let error_msg = format!("Failed to initialize database schema: {}", e);
            eprintln!("{}", error_msg);

            // DBファイルが破損している可能性がある場合、バックアップを作成して再試行
            if db_path.exists() {
                eprintln!(
                    "Existing database file may be corrupted, creating backup and recreating..."
                );
                let backup_path = format!("{}.backup", db_path.display());
                if let Err(backup_err) = std::fs::rename(&db_path, &backup_path) {
                    eprintln!("Failed to create backup: {}", backup_err);
                } else {
                    eprintln!("Created backup: {}", backup_path);

                    // 新しいDBファイルを作成して再試行
                    return get_connection(app_handle);
                }
            }

            // 初期化失敗を記録
            let mut state = DB_INIT_STATE.get().unwrap().lock().unwrap();
            *state = Err(error_msg.clone());
            return Err(error_msg.into());
        }
    }

    Ok(conn)
}

#[allow(dead_code)]
pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
    schema::init_database(&conn).map_err(|e| format!("Failed to initialize schema: {}", e))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[cfg_attr(
        target_os = "windows",
        ignore = "Database tests are unstable on Windows local environment"
    )]
    fn test_database_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection_with_path(db_path.clone()).unwrap();

        // データベースが作成されていることを確認
        assert!(db_path.exists());

        // テーブルが作成されていることを確認
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"channels".to_string()));
        assert!(tables.contains(&"streams".to_string()));
        assert!(tables.contains(&"stream_stats".to_string()));
    }

    #[test]
    #[cfg_attr(
        target_os = "windows",
        ignore = "Database tests are unstable on Windows local environment"
    )]
    fn test_database_schema_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_schema.db");

        // 2回初期化してもエラーにならないことを確認
        let _conn1 = get_connection_with_path(db_path.clone()).unwrap();
        let _conn2 = get_connection_with_path(db_path.clone()).unwrap();

        assert!(db_path.exists());
    }
}
