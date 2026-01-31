use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub twitch: TwitchSettings,
    pub youtube: YouTubeSettings,
    // 将来の機能: YouTubeスクレイピング設定（設定ファイルを直接編集しないと有効化できない）
    #[serde(default = "default_scraping_settings")]
    pub youtube_scraping: Option<YouTubeScrapingSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchSettings {
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeSettings {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// 将来の機能: YouTubeスクレイピング設定
/// Chromiumを使用してYouTubeページをロードし、ゲームタイトル該当の要素文字列を抜き出す機能
/// 設定ファイルを直接編集しないと有効化できない隠しオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeScrapingSettings {
    /// スクレイピング機能を有効化するか
    pub enabled: bool,
    /// Chromiumの実行可能ファイルパス（未指定の場合は自動検出）
    pub chromium_path: Option<String>,
    /// ゲームタイトル要素のセレクタ
    pub game_title_selector: Option<String>,
    /// タイムアウト（秒）
    pub timeout_seconds: Option<u64>,
}

fn default_scraping_settings() -> Option<YouTubeScrapingSettings> {
    None // デフォルトでは無効
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            twitch: TwitchSettings { client_id: None },
            youtube: YouTubeSettings {
                client_id: None,
                client_secret: None,
            },
            youtube_scraping: None,
        }
    }
}

#[allow(dead_code)]
pub struct SettingsManager;

impl SettingsManager {
    pub fn get_settings_path(
        _app_handle: &AppHandle,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Tauri 2.xのapp_data_dir()を取得
        // 一時的な実装：現在のディレクトリに設定ファイルを作成
        let app_data_dir = std::env::current_dir()
            .or_else(|_| std::path::PathBuf::from(".").canonicalize())
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Ok(app_data_dir.join("settings.json"))
    }

    pub fn load_settings(
        app_handle: &AppHandle,
    ) -> Result<AppSettings, Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path(app_handle)?;

        if !settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = std::fs::read_to_string(&settings_path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn save_settings(
        app_handle: &AppHandle,
        settings: &AppSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path(app_handle)?;

        // ディレクトリが存在しない場合は作成
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&settings_path, content)?;
        Ok(())
    }
}

// 将来の機能: YouTubeスクレイピング実装のプレースホルダー
#[allow(dead_code)]
pub mod youtube_scraping {
    use super::YouTubeScrapingSettings;

    /// YouTubeページからゲームタイトルをスクレイピングする（将来実装）
    pub async fn scrape_game_title(
        _video_url: &str,
        _settings: &YouTubeScrapingSettings,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // TODO: 将来的に実装
        // 1. Chromiumヘッドレスブラウザを起動
        // 2. YouTubeページをロード
        // 3. ゲームタイトル該当の要素を取得
        // 4. 文字列を抽出して返す

        Err("YouTube scraping feature not yet implemented".into())
    }

    /// Chromiumの実行可能ファイルを自動検出する（将来実装）
    pub fn detect_chromium_path() -> Option<String> {
        // TODO: 将来的に実装
        // Windows: "C:\Program Files\Google\Chrome\Application\chrome.exe" など
        // macOS: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" など
        // Linux: "google-chrome" または "chromium-browser" など

        None
    }
}
