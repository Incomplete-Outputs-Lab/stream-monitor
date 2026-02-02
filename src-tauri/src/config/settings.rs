use crate::error::ResultExt;
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
    // Twitch自動発見機能設定
    #[serde(default)]
    pub auto_discovery: Option<AutoDiscoverySettings>,
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

/// Twitch自動発見機能設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDiscoverySettings {
    /// 自動発見機能を有効化するか
    pub enabled: bool,
    /// ポーリング間隔（秒）
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u32,
    /// 取得する最大配信数（1-100）
    #[serde(default = "default_max_streams")]
    pub max_streams: u32,
    /// フィルター設定
    #[serde(default)]
    pub filters: AutoDiscoveryFilters,
}

/// 自動発見フィルター設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoDiscoveryFilters {
    /// フィルターするゲームID（最大100件）
    #[serde(default)]
    pub game_ids: Vec<String>,
    /// フィルターする言語コード（例: ja, en）（最大100件）
    #[serde(default)]
    pub languages: Vec<String>,
    /// 最小視聴者数
    pub min_viewers: Option<u32>,
}

impl Default for AutoDiscoverySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            poll_interval: default_poll_interval(),
            max_streams: default_max_streams(),
            filters: AutoDiscoveryFilters::default(),
        }
    }
}

fn default_poll_interval() -> u32 {
    300 // 5分
}

fn default_max_streams() -> u32 {
    20 // デフォルト20件
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
            auto_discovery: None,
        }
    }
}

#[allow(dead_code)]
pub struct SettingsManager;

impl SettingsManager {
    pub fn get_settings_path(
        app_handle: &AppHandle,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        use tauri::Manager;

        // Tauri 2.xのapp_data_dir()を取得（ログファイルやDBファイルと同じ場所）
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;

        // ディレクトリが存在しない場合は作成
        std::fs::create_dir_all(&app_data_dir)
            .io_context("create app data directory")
            .map_err(|e| e.to_string())?;

        Ok(app_data_dir.join("settings.json"))
    }

    pub fn load_settings(
        app_handle: &AppHandle,
    ) -> Result<AppSettings, Box<dyn std::error::Error + Send + Sync>> {
        let settings_path = Self::get_settings_path(app_handle)?;

        if !settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let content = std::fs::read_to_string(&settings_path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    pub fn save_settings(
        app_handle: &AppHandle,
        settings: &AppSettings,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
