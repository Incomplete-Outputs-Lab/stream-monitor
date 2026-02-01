use crate::collectors::auto_discovery::AutoDiscoveryPoller;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

/// 自動発見設定を取得
#[tauri::command]
pub async fn get_auto_discovery_settings(
    app_handle: AppHandle,
) -> Result<Option<AutoDiscoverySettings>, String> {
    let settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    Ok(settings.auto_discovery)
}

/// 自動発見設定を保存
#[tauri::command]
pub async fn save_auto_discovery_settings(
    app_handle: AppHandle,
    mut settings: AutoDiscoverySettings,
    auto_discovery_poller: State<'_, Arc<Mutex<Option<AutoDiscoveryPoller>>>>,
) -> Result<(), String> {
    // max_streamsのバリデーション（1-500の範囲に制限）
    if settings.max_streams < 1 {
        settings.max_streams = 1;
    } else if settings.max_streams > 500 {
        settings.max_streams = 500;
    }

    // 設定をロード
    let mut app_settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // 自動発見設定を更新
    let was_enabled = app_settings
        .auto_discovery
        .as_ref()
        .map(|s| s.enabled)
        .unwrap_or(false);
    let is_enabled = settings.enabled;

    app_settings.auto_discovery = Some(settings);

    // 設定を保存
    SettingsManager::save_settings(&app_handle, &app_settings)
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // ポーラーの状態を更新
    let poller_guard = auto_discovery_poller.lock().await;
    if let Some(poller) = poller_guard.as_ref() {
        if is_enabled && !was_enabled {
            // 有効化された場合は開始
            poller
                .start()
                .await
                .map_err(|e| format!("Failed to start auto-discovery: {}", e))?;
        } else if !is_enabled && was_enabled {
            // 無効化された場合は停止
            poller.stop().await;
        } else if is_enabled {
            // 設定が変更された場合は再起動
            poller.stop().await;
            poller
                .start()
                .await
                .map_err(|e| format!("Failed to restart auto-discovery: {}", e))?;
        }
    }

    Ok(())
}

/// 自動発見のON/OFF切り替え
#[tauri::command]
pub async fn toggle_auto_discovery(
    app_handle: AppHandle,
    auto_discovery_poller: State<'_, Arc<Mutex<Option<AutoDiscoveryPoller>>>>,
) -> Result<bool, String> {
    // 設定をロード
    let mut settings = SettingsManager::load_settings(&app_handle)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // 現在の状態を取得
    let current_enabled = settings
        .auto_discovery
        .as_ref()
        .map(|s| s.enabled)
        .unwrap_or(false);

    // 状態を反転
    let new_enabled = !current_enabled;

    // 設定が存在しない場合はデフォルト設定を作成
    if settings.auto_discovery.is_none() {
        settings.auto_discovery = Some(AutoDiscoverySettings::default());
    }

    if let Some(ref mut ad_settings) = settings.auto_discovery {
        ad_settings.enabled = new_enabled;
    }

    // 設定を保存
    SettingsManager::save_settings(&app_handle, &settings)
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // ポーラーの状態を更新
    let poller_guard = auto_discovery_poller.lock().await;
    if let Some(poller) = poller_guard.as_ref() {
        if new_enabled {
            poller
                .start()
                .await
                .map_err(|e| format!("Failed to start auto-discovery: {}", e))?;
        } else {
            poller.stop().await;
        }
    }

    Ok(new_enabled)
}

/// 発見された配信の一覧を取得（メモリキャッシュから）
#[tauri::command]
pub async fn get_discovered_streams(
    app_handle: AppHandle,
) -> Result<Vec<DiscoveredStreamInfo>, String> {
    let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
    let streams_lock = cache.streams.lock().await;
    let streams = streams_lock.clone();
    drop(streams_lock);

    Ok(streams)
}

/// Twitchゲーム検索（フィルター設定用）
#[tauri::command]
pub async fn search_twitch_games(query: String) -> Result<Vec<TwitchGame>, String> {
    // TODO: Twitch API のSearch Categories エンドポイントを実装
    // 現時点では空の配列を返す
    // 将来的に twitch_api クレートの SearchCategoriesRequest を使用して実装
    eprintln!("[SearchGames] Search query: {}", query);
    Ok(vec![])
}

/// 自動発見チャンネルを手動登録に昇格
#[tauri::command]
pub async fn promote_discovered_channel(
    db_manager: State<'_, DatabaseManager>,
    app_handle: AppHandle,
    channel_id: String, // Twitch user_id
) -> Result<(), String> {
    // メモリキャッシュから該当するストリーム情報を取得
    let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
    let streams_lock = cache.streams.lock().await;
    let stream_info = streams_lock
        .iter()
        .find(|s| s.channel_id == channel_id)
        .cloned();
    drop(streams_lock);

    let stream_info = stream_info.ok_or_else(|| format!("Channel {} not found in cache", channel_id))?;

    // channelsテーブルに新規登録（手動登録として）
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    // 既に登録されているかチェック
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM channels WHERE platform = 'twitch' AND channel_id = ?")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;
    let count: i64 = stmt
        .query_row([&channel_id], |row| row.get(0))
        .map_err(|e| format!("Failed to query: {}", e))?;
    drop(stmt);

    if count > 0 {
        // 既に登録されている場合はis_auto_discoveredフラグを更新
        conn.execute(
            "UPDATE channels SET is_auto_discovered = false, discovered_at = NULL WHERE platform = 'twitch' AND channel_id = ?",
            [&channel_id],
        )
        .map_err(|e| format!("Failed to update channel: {}", e))?;
    } else {
        // 新規登録
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO channels (
                platform, channel_id, channel_name, display_name,
                profile_image_url, broadcaster_type, follower_count,
                enabled, poll_interval, is_auto_discovered, discovered_at,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            duckdb::params![
                "twitch",
                channel_id.as_str(),
                stream_info.channel_name.as_str(),
                stream_info.display_name.as_deref().unwrap_or(""),
                stream_info.profile_image_url.as_deref().unwrap_or(""),
                stream_info.broadcaster_type.as_deref().unwrap_or(""),
                stream_info.follower_count.unwrap_or(0),
                "true",  // enabled
                "60",    // poll_interval
                "false", // is_auto_discovered
                None::<String>, // discovered_at
                now.as_str(),
                now.as_str(),
            ],
        )
        .map_err(|e| format!("Failed to insert channel: {}", e))?;
    }

    drop(conn);

    eprintln!("[Discovery] Promoted channel {} to manual registration", channel_id);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredStreamInfo {
    pub id: i64,
    pub channel_id: String,
    pub channel_name: String,
    pub display_name: Option<String>,
    pub profile_image_url: Option<String>,
    pub discovered_at: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub viewer_count: Option<i32>,
    pub follower_count: Option<i32>,
    pub broadcaster_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchGame {
    pub id: String,
    pub name: String,
    pub box_art_url: String,
}
