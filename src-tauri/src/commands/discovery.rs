use crate::collectors::auto_discovery::AutoDiscoveryPoller;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
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
    settings: AutoDiscoverySettings,
    auto_discovery_poller: State<'_, Arc<Mutex<Option<AutoDiscoveryPoller>>>>,
) -> Result<(), String> {
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

/// 発見された配信の一覧を取得
#[tauri::command]
pub async fn get_discovered_streams(
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<DiscoveredStreamInfo>, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT 
                c.id,
                c.channel_id,
                c.channel_name,
                c.display_name,
                c.profile_image_url,
                c.discovered_at,
                s.title,
                s.category,
                ss.viewer_count
            FROM channels c
            LEFT JOIN streams s ON s.channel_id = c.id AND s.ended_at IS NULL
            LEFT JOIN stream_stats ss ON ss.stream_id = s.id
            WHERE c.is_auto_discovered = true
            ORDER BY c.discovered_at DESC
            "#,
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let streams = stmt
        .query_map([], |row| {
            Ok(DiscoveredStreamInfo {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                channel_name: row.get(2)?,
                display_name: row.get(3).ok(),
                profile_image_url: row.get(4).ok(),
                discovered_at: row.get(5).ok(),
                title: row.get(6).ok(),
                category: row.get(7).ok(),
                viewer_count: row.get(8).ok(),
            })
        })
        .map_err(|e| format!("Failed to query streams: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect streams: {}", e))?;

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
    channel_id: i64,
) -> Result<(), String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get connection: {}", e))?;

    conn.execute(
        "UPDATE channels SET is_auto_discovered = false, discovered_at = NULL WHERE id = ?",
        [channel_id],
    )
    .map_err(|e| format!("Failed to promote channel: {}", e))?;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchGame {
    pub id: String,
    pub name: String,
    pub box_art_url: String,
}
