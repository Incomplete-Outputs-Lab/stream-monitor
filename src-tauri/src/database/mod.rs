pub mod aggregation;
pub mod analytics;
pub mod chat_analytics;
pub mod data_science_analytics;
pub mod models;
pub mod schema;
pub mod utils;
pub mod writer;

use crate::error::ResultExt;
use duckdb::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// 起動前のリカバリ処理（一時ファイルのクリーンアップ）
fn cleanup_stale_files(db_path: &Path) {
    // DuckDBの一時ファイルパス
    let wal_path = db_path.with_extension("wal");
    let tmp_path = db_path.with_extension("tmp");

    // .wal ファイルが存在する場合は警告ログ（DuckDBが自動リカバリ）
    if wal_path.exists() {
        eprintln!(
            "[DB Recovery] WAL file found at {}, DuckDB will auto-recover",
            wal_path.display()
        );
    }

    // .tmp ファイルは削除（不完全な操作の残骸）
    if tmp_path.exists() {
        eprintln!(
            "[DB Recovery] Removing stale tmp file: {}",
            tmp_path.display()
        );
        if let Err(e) = std::fs::remove_file(&tmp_path) {
            eprintln!("[DB Recovery] Failed to remove tmp file: {}", e);
        }
    }
}

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
                .io_context("create app data directory")
                .map_err(|e| e.to_string())?;
            app_data_dir.join("stream_stats.db")
        } else {
            eprintln!("Warning: Using current directory for database");
            PathBuf::from("stream_stats.db")
        };

        // 起動時のリカバリ処理
        cleanup_stale_files(&db_path);

        // 開発環境と本番環境で統一してファイルベースDBを使用
        eprintln!("Opening DuckDB at: {}", db_path.display());
        let conn = Connection::open(&db_path)
            .db_context("open database")
            .map_err(|e| e.to_string())?;

        // DuckDBの設定
        conn.execute("PRAGMA memory_limit='1GB'", []).ok();
        conn.execute("PRAGMA threads=4", []).ok();
        conn.execute("PRAGMA wal_autocheckpoint='1000'", []).ok(); // 1000ページごとに自動チェックポイント

        // スキーマ初期化
        schema::init_database(&conn)?;

        eprintln!("Database initialized successfully");

        Ok(DatabaseManager {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// データベース接続を取得
    pub fn get_connection(&self) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Database connection lock failed: {}", e))?;
        conn.try_clone()
            .db_context("clone connection")
            .map_err(|e| e.to_string().into())
    }

    /// 手動バックアップを作成
    /// データベースファイルのパスを取得
    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// グレースフルシャットダウン - WALをフラッシュ
    pub fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("[DB Shutdown] Starting graceful shutdown...");

        let conn = self
            .conn
            .lock()
            .map_err(|e| format!("Lock failed: {}", e))?;

        // WALチェックポイントを強制実行（全データをメインDBにフラッシュ）
        match conn.execute("CHECKPOINT", []) {
            Ok(_) => eprintln!("[DB Shutdown] CHECKPOINT completed successfully"),
            Err(e) => eprintln!("[DB Shutdown] CHECKPOINT failed: {}", e),
        }

        eprintln!("[DB Shutdown] Shutdown completed");
        Ok(())
    }

    /// 定期的なチェックポイント（データ安全性向上）
    #[allow(dead_code)]
    pub fn checkpoint(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.get_connection()?;
        conn.execute("CHECKPOINT", [])?;
        Ok(())
    }
}

// 後方互換性のための関数
pub fn get_connection(
    app_handle: &AppHandle,
) -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
    let db_manager: tauri::State<'_, DatabaseManager> = app_handle.state();
    db_manager.get_connection()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // テスト用のヘルパー関数
    fn get_connection_with_path(
        db_path: PathBuf,
    ) -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = Connection::open(&db_path)
            .db_context("open test database")
            .map_err(|e| e.to_string())?;
        Ok(conn)
    }

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
