pub mod aggregation;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

// データベース接続を共有するための管理構造体
#[derive(Clone)]
pub struct DatabaseManager {
    connection: Arc<Mutex<Option<Connection>>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // データベースパスの取得
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

        Ok(DatabaseManager {
            connection: Arc::new(Mutex::new(None)),
            db_path,
        })
    }

    // 接続を取得（必要に応じて作成）
    pub fn get_connection(&self) -> Result<duckdb::Connection, Box<dyn std::error::Error>> {
        let mut conn_guard = self.connection.lock().unwrap();

        // 接続が既に存在する場合はそれを返す
        if let Some(ref conn) = *conn_guard {
            // 接続が有効か確認（簡単なクエリを実行）
            match conn.execute("SELECT 1", []) {
                Ok(_) => {
                    return Ok((*conn).try_clone().map_err(|e| format!("Failed to clone connection: {}", e))?);
                }
                Err(_) => {
                    eprintln!("Database connection is invalid, recreating...");
                    *conn_guard = None;
                }
            }
        }

        // 新しい接続を作成
        eprintln!("Creating new database connection at: {}", self.db_path.display());
        let conn = self.create_connection()?;
        *conn_guard = Some(conn.try_clone().map_err(|e| format!("Failed to clone connection: {}", e))?);

        Ok(conn)
    }

    // 実際の接続作成処理
    fn create_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        // データベースファイルの存在チェック
        let file_exists = self.db_path.exists();
        eprintln!("Opening DuckDB connection at: {}", self.db_path.display());
        eprintln!("Database file exists: {}", file_exists);

        // ファイルが存在するが読み取り不可の場合、破損の可能性がある
        if file_exists {
            match std::fs::metadata(&self.db_path) {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        eprintln!("Warning: Database file exists but is empty (0 bytes)");
                    } else {
                        eprintln!("Database file size: {} bytes", metadata.len());
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Cannot read database file metadata: {}", e);
                }
            }
        } else {
            eprintln!("Database file does not exist, will be created");
        }

        // Use a thread with larger stack for DuckDB connection
        let db_path_clone = self.db_path.clone();
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
                return Err(format!("Failed to open database at {}: {}", self.db_path.display(), e).into());
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

        Ok(conn)
    }
}


// 後方互換性のための関数（DatabaseManagerを使用）
pub fn get_connection(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    db_manager.get_connection()
}

#[allow(dead_code)]
pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
    // Note: init_database is now called only once at application startup
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
        // テスト用に明示的にスキーマ初期化を実行
        schema::init_database(&conn).unwrap();

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
        let conn1 = get_connection_with_path(db_path.clone()).unwrap();
        let conn2 = get_connection_with_path(db_path.clone()).unwrap();

        // 明示的にスキーマ初期化を2回実行
        schema::init_database(&conn1).unwrap();
        schema::init_database(&conn2).unwrap();

        assert!(db_path.exists());
    }
}
