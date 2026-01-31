pub mod aggregation;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use duckdb::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// データベース接続を共有するための管理構造体
#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // データベースファイルパスの取得
        let db_path = if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;
            app_data_dir.join("stream_stats.db")
        } else {
            eprintln!("Warning: Using current directory for database");
            PathBuf::from("stream_stats.db")
        };

        // 開発環境ではインメモリDB、本番環境ではファイルベースDBを使用
        let conn = if cfg!(debug_assertions) {
            eprintln!("Development mode: Using in-memory database (hot-reload safe)");
            Connection::open_in_memory()
                .map_err(|e| format!("Failed to open in-memory database: {}", e))?
        } else {
            eprintln!("Production mode: Opening DuckDB at: {}", db_path.display());
            Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?
        };

        // DuckDBの設定
        conn.execute("PRAGMA memory_limit='1GB'", []).ok();
        conn.execute("PRAGMA threads=4", []).ok();

        // スキーマ初期化
        schema::init_database(&conn)?;

        eprintln!("Database initialized successfully");

        Ok(DatabaseManager {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// データベース接続を取得
    pub fn get_connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Failed to lock connection: {}", e))?;
        conn.try_clone()
            .map_err(|e| format!("Failed to clone connection: {}", e).into())
    }

    /// 手動バックアップを作成
    pub fn create_backup(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // インメモリDB使用時はバックアップを作成できない
        if cfg!(debug_assertions) {
            return Err(
                "Cannot create backup in development mode (using in-memory database)".into(),
            );
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .db_path
            .with_extension(format!("db.backup.{}", timestamp));

        // CHECKPOINTでWALをフラッシュしてからコピー
        if let Ok(conn) = self.conn.lock() {
            conn.execute("CHECKPOINT", []).ok();
        }

        std::fs::copy(&self.db_path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;

        eprintln!("Backup created at: {}", backup_path.display());
        Ok(backup_path)
    }

    /// データベースファイルのパスを取得
    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

// 後方互換性のための関数
pub fn get_connection(app_handle: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    db_manager.get_connection()
}

#[allow(dead_code)]
pub fn get_connection_with_path(path: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database: {}", e))?;
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
