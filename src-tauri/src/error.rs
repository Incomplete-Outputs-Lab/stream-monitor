use thiserror::Error;

/// アプリケーション全体のエラー型（シンプルな文字列ベース）
#[derive(Debug, Error)]
pub enum AppError {
    /// データベースエラー
    #[error("Database error: {context}: {message}")]
    Database { context: String, message: String },

    /// APIエラー
    #[error("API error: {context}: {message}")]
    Api { context: String, message: String },

    /// 認証エラー
    #[error("Authentication error: {0}")]
    Auth(String),

    /// リソースが見つからない
    #[error("Not found: {0}")]
    NotFound(String),

    /// 無効な入力
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 設定エラー
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/Oエラー
    #[error("I/O error: {context}: {message}")]
    Io { context: String, message: String },

    /// シリアライゼーションエラー
    #[error("Serialization error: {context}: {message}")]
    Serialization { context: String, message: String },

    /// 一般的なエラー
    #[error("{0}")]
    General(String),
}

/// Tauri コマンド用の String 変換
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

/// duckdb::Error からの変換
impl From<duckdb::Error> for AppError {
    fn from(err: duckdb::Error) -> Self {
        AppError::Database {
            context: "database operation".to_string(),
            message: err.to_string(),
        }
    }
}

/// std::io::Error からの変換
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io {
            context: "I/O operation".to_string(),
            message: err.to_string(),
        }
    }
}

/// コンテキスト付きエラー変換トレイト
#[allow(dead_code)]
pub trait ResultExt<T> {
    /// データベースエラーにコンテキストを追加
    fn db_context(self, context: &str) -> Result<T, AppError>;

    /// APIエラーにコンテキストを追加
    fn api_context(self, context: &str) -> Result<T, AppError>;

    /// I/Oエラーにコンテキストを追加
    fn io_context(self, context: &str) -> Result<T, AppError>;

    /// 設定エラーにコンテキストを追加
    fn config_context(self, context: &str) -> Result<T, AppError>;

    /// 一般的なエラーにコンテキストを追加
    fn context(self, context: &str) -> Result<T, AppError>;
}

impl<T> ResultExt<T> for Result<T, duckdb::Error> {
    fn db_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Database {
            context: context.to_string(),
            message: e.to_string(),
        })
    }

    fn api_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn io_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn config_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn context(self, context: &str) -> Result<T, AppError> {
        self.db_context(context)
    }
}

impl<T> ResultExt<T> for Result<T, std::io::Error> {
    fn db_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn api_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn io_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Io {
            context: context.to_string(),
            message: e.to_string(),
        })
    }

    fn config_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn context(self, context: &str) -> Result<T, AppError> {
        self.io_context(context)
    }
}

impl<T> ResultExt<T> for Result<T, Box<dyn std::error::Error>> {
    fn db_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn api_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Api {
            context: context.to_string(),
            message: e.to_string(),
        })
    }

    fn io_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(e.to_string()))
    }

    fn config_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Config(format!("{}: {}", context, e)))
    }

    fn context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(format!("{}: {}", context, e)))
    }
}

impl<T> ResultExt<T> for Result<T, String> {
    fn db_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(AppError::General)
    }

    fn api_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Api {
            context: context.to_string(),
            message: e,
        })
    }

    fn io_context(self, _context: &str) -> Result<T, AppError> {
        self.map_err(AppError::General)
    }

    fn config_context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Config(format!("{}: {}", context, e)))
    }

    fn context(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::General(format!("{}: {}", context, e)))
    }
}

/// Option を Result に変換するヘルパー
pub trait OptionExt<T> {
    /// None の場合に NotFound エラーを返す
    fn ok_or_not_found(self, message: &str) -> Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, message: &str) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::NotFound(message.to_string()))
    }
}
