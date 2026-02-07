use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_LOG_SIZE: u64 = 1_048_576; // 1MB

pub struct AppLogger {
    log_path: PathBuf,
    writer: Arc<Mutex<Option<BufWriter<File>>>>,
}

impl AppLogger {
    pub fn new(log_path: PathBuf) -> Result<Self, std::io::Error> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let logger = Self {
            log_path,
            writer: Arc::new(Mutex::new(None)),
        };

        logger.init_writer()?;
        Ok(logger)
    }

    fn init_writer(&self) -> Result<(), std::io::Error> {
        // ファイルサイズをチェックしてローテーション
        if self.log_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&self.log_path) {
                if metadata.len() > MAX_LOG_SIZE {
                    // ログファイルが大きすぎる場合は削除
                    std::fs::remove_file(&self.log_path)?;
                }
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let buf_writer = BufWriter::new(file);
        let mut writer = self.writer.blocking_lock();
        *writer = Some(buf_writer);

        Ok(())
    }

    pub fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_line = format!("[{}] {}: {}\n", timestamp, level, message);

        // コンソールにも出力
        if level == "ERROR" {
            eprint!("{}", log_line);
        } else {
            print!("{}", log_line);
        }

        // ファイルに書き込み
        let mut writer_guard = self.writer.blocking_lock();
        if let Some(writer) = writer_guard.as_mut() {
            let _ = writer.write_all(log_line.as_bytes());
            let _ = writer.flush();
        }
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

impl Clone for AppLogger {
    fn clone(&self) -> Self {
        Self {
            log_path: self.log_path.clone(),
            writer: Arc::clone(&self.writer),
        }
    }
}
