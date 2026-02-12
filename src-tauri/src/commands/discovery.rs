use crate::collectors::auto_discovery::AutoDiscoveryPoller;
use crate::config::settings::{AutoDiscoverySettings, SettingsManager};
use crate::constants::database as db_constants;
use crate::database::{repositories::ChannelRepository, DatabaseManager};
use crate::error::ResultExt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

/// 自動発見設定を取得
#[tauri::command]
pub async fn get_auto_discovery_settings(
    app_handle: AppHandle,
) -> Result<Option<AutoDiscoverySettings>, String> {
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    Ok(settings.auto_discovery)
}

/// 自動発見設定を保存
#[tauri::command]
pub async fn save_auto_discovery_settings(
    app_handle: AppHandle,
    mut settings: AutoDiscoverySettings,
    auto_discovery_poller: State<'_, Arc<Mutex<Option<AutoDiscoveryPoller>>>>,
    channel_poller: State<'_, Arc<Mutex<crate::collectors::poller::ChannelPoller>>>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<(), String> {
    // max_streamsのバリデーション（1-500の範囲に制限）
    settings.max_streams = settings.max_streams.clamp(1, 500);

    // game_idsのバリデーション（最大100件に制限）
    if settings.filters.game_ids.len() > 100 {
        return Err("ゲームIDは最大100件までです".to_string());
    }

    // languagesのバリデーション（最大10件に制限）
    if settings.filters.languages.len() > 10 {
        return Err("言語は最大10件までです".to_string());
    }

    // 設定をロード
    let mut app_settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    // 自動発見設定を更新
    let is_enabled = settings.enabled;

    app_settings.auto_discovery = Some(settings);

    // 設定を保存
    SettingsManager::save_settings(&app_handle, &app_settings)
        .config_context("save settings")
        .map_err(|e| e.to_string())?;

    // 設定変更時はキャッシュをクリア
    if is_enabled {
        let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
        let mut streams_lock = cache.streams.lock().await;
        streams_lock.clear();
        drop(streams_lock);
        eprintln!("[AutoDiscovery] Cache cleared due to settings change");
    }

    // ポーラーを停止（存在する場合）
    {
        let mut poller_guard = auto_discovery_poller.lock().await;
        if let Some(ref poller) = *poller_guard {
            poller.stop().await;
        }

        // 新しいTwitchクライアントを取得（最新のトークンを使用）
        let twitch_api_client = if app_settings.twitch.client_id.is_some() {
            let channel_poller_guard = channel_poller.lock().await;
            channel_poller_guard
                .get_twitch_collector()
                .map(|tc| Arc::clone(tc.get_api_client()))
        } else {
            None
        };

        eprintln!(
            "[AutoDiscovery] Reinitializing AutoDiscoveryPoller with Twitch client: {}",
            if twitch_api_client.is_some() {
                "available"
            } else {
                "unavailable"
            }
        );

        // 新しいAutoDiscoveryPollerを作成
        let new_discovery_poller = AutoDiscoveryPoller::new(
            twitch_api_client,
            Arc::new(db_manager.inner().clone()),
            app_handle.clone(),
        );

        // 設定が有効ならstart
        if is_enabled {
            new_discovery_poller
                .start()
                .await
                .map_err(|e| format!("Auto-discovery start failed: {}", e))?;
            eprintln!("[AutoDiscovery] Started successfully");
        }

        // 状態を更新
        *poller_guard = Some(new_discovery_poller);
    }

    Ok(())
}

/// 自動発見のON/OFF切り替え
#[tauri::command]
pub async fn toggle_auto_discovery(
    app_handle: AppHandle,
    auto_discovery_poller: State<'_, Arc<Mutex<Option<AutoDiscoveryPoller>>>>,
    channel_poller: State<'_, Arc<Mutex<crate::collectors::poller::ChannelPoller>>>,
    db_manager: State<'_, DatabaseManager>,
) -> Result<bool, String> {
    // 設定をロード
    let mut settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

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
        .config_context("save settings")
        .map_err(|e| e.to_string())?;

    eprintln!(
        "[AutoDiscovery] Toggle: {} -> {}",
        current_enabled, new_enabled
    );

    // 設定変更時はキャッシュをクリア
    if new_enabled {
        let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
        let mut streams_lock = cache.streams.lock().await;
        streams_lock.clear();
        drop(streams_lock);
        eprintln!("[AutoDiscovery] Cache cleared due to toggle");
    }

    // ポーラーを停止（存在する場合）してから再初期化
    {
        let mut poller_guard = auto_discovery_poller.lock().await;

        // 既存のポーラーを停止
        if let Some(ref poller) = *poller_guard {
            poller.stop().await;
        }

        // 新しいTwitchクライアントを取得（最新のトークンを使用）
        let twitch_api_client = if settings.twitch.client_id.is_some() {
            let channel_poller_guard = channel_poller.lock().await;
            channel_poller_guard
                .get_twitch_collector()
                .map(|tc| Arc::clone(tc.get_api_client()))
        } else {
            None
        };

        eprintln!(
            "[AutoDiscovery] Reinitializing AutoDiscoveryPoller with Twitch client: {}",
            if twitch_api_client.is_some() {
                "available"
            } else {
                "unavailable"
            }
        );

        // 新しいAutoDiscoveryPollerを作成
        let new_discovery_poller = AutoDiscoveryPoller::new(
            twitch_api_client,
            Arc::new(db_manager.inner().clone()),
            app_handle.clone(),
        );

        // 設定が有効ならstart
        if new_enabled {
            new_discovery_poller
                .start()
                .await
                .map_err(|e| format!("Auto-discovery start failed: {}", e))?;
            eprintln!("[AutoDiscovery] Started successfully");
        }

        // 状態を更新
        *poller_guard = Some(new_discovery_poller);
    }

    Ok(new_enabled)
}

/// 発見された配信の一覧を取得（メモリキャッシュから）
/// 既に登録されているチャンネルは除外して返す
#[tauri::command]
pub async fn get_discovered_streams(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<DiscoveredStreamInfo>, String> {
    eprintln!("[Discovery] === get_discovered_streams called ===");

    // 自動発見が有効かチェック
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let auto_discovery_enabled = settings
        .auto_discovery
        .as_ref()
        .map(|s| s.enabled)
        .unwrap_or(false);

    eprintln!(
        "[Discovery] Auto-discovery enabled: {}",
        auto_discovery_enabled
    );

    // 自動発見が無効の場合は即座に空配列を返す
    if !auto_discovery_enabled {
        eprintln!("[Discovery] Auto-discovery is disabled, returning empty list");
        return Ok(Vec::new());
    }

    // 初期化待機（最大2秒 - デッドロック防止のため短く設定）
    let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
    let is_initialized = cache.initialized.load(Ordering::SeqCst);

    eprintln!("[Discovery] Cache initialized: {}", is_initialized);

    if !is_initialized {
        // 初期化未完了の場合は即座に空配列を返す
        // （discovered-streams-updatedイベントでフロントエンドが自動更新される）
        eprintln!("[Discovery] Auto-discovery not yet initialized, returning empty list");
        return Ok(Vec::new());
    }

    // 1. 既に登録されているチャンネルのtwitch_user_idを取得
    let registered_user_ids: HashSet<i64> = db_manager
        .with_connection(|conn| {
            ChannelRepository::get_all_twitch_user_ids(conn)
                .db_context("get twitch user ids")
                .map_err(|e| e.to_string())
                .map(|ids| ids.into_iter().collect())
        })
        .await
        .db_context("get connection")
        .map_err(|e| e.to_string())?;

    // 2. メモリキャッシュから配信を取得（既に取得済みのcache変数を再利用）
    let streams = {
        let streams_lock = cache.streams.lock().await;
        let streams = streams_lock.clone();
        drop(streams_lock);
        streams
    };

    // 3. 既に登録されているチャンネルを除外
    let filtered_streams: Vec<DiscoveredStreamInfo> = streams
        .into_iter()
        .filter(|s| !registered_user_ids.contains(&s.twitch_user_id))
        .collect();

    eprintln!(
        "[Discovery] Returning {} discovered streams (filtered out {} registered channels)",
        filtered_streams.len(),
        registered_user_ids.len()
    );

    Ok(filtered_streams)
}

/// Twitchゲーム検索（フィルター設定用）
#[tauri::command]
pub async fn search_twitch_games(
    app_handle: AppHandle,
    query: String,
) -> Result<Vec<TwitchGame>, String> {
    use crate::api::twitch_api::TwitchApiClient;
    use crate::config::settings::SettingsManager;

    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    eprintln!("[SearchGames] Search query: {}", query);

    // 設定からClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let client_id = settings
        .twitch
        .client_id
        .as_ref()
        .ok_or_else(|| "Twitch Client ID not configured".to_string())?;

    // TwitchApiClientを作成
    let api_client = TwitchApiClient::new(client_id.clone(), None).with_app_handle(app_handle);

    // カテゴリを検索（最大20件）
    let categories = api_client
        .search_categories(&query, Some(20))
        .await
        .map_err(|e| format!("Failed to search categories: {}", e))?;

    // TwitchGame型にマッピング
    let games = categories
        .into_iter()
        .map(|cat| TwitchGame {
            id: cat.id.to_string(),
            name: cat.name.to_string(),
            box_art_url: cat.box_art_url.to_string(),
        })
        .collect();

    Ok(games)
}

/// ゲームIDからゲーム情報を取得（既存設定の表示用）
#[tauri::command]
pub async fn get_games_by_ids(
    app_handle: AppHandle,
    game_ids: Vec<String>,
) -> Result<Vec<TwitchGame>, String> {
    use crate::api::twitch_api::TwitchApiClient;
    use crate::config::settings::SettingsManager;

    if game_ids.is_empty() {
        return Ok(vec![]);
    }

    eprintln!("[GetGames] Get games for IDs: {:?}", game_ids);

    // 設定からClient IDを取得
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| e.to_string())?;

    let client_id = settings
        .twitch
        .client_id
        .as_ref()
        .ok_or_else(|| "Twitch Client ID not configured".to_string())?;

    // TwitchApiClientを作成
    let api_client = TwitchApiClient::new(client_id.clone(), None).with_app_handle(app_handle);

    // ゲームIDを&str参照のベクターに変換
    let game_id_refs: Vec<&str> = game_ids.iter().map(|s| s.as_str()).collect();

    // ゲーム情報を取得
    let categories = api_client
        .get_games_by_ids(&game_id_refs)
        .await
        .map_err(|e| format!("Failed to get games: {}", e))?;

    // TwitchGame型にマッピング
    let games = categories
        .into_iter()
        .map(|cat| TwitchGame {
            id: cat.id.to_string(),
            name: cat.name.to_string(),
            box_art_url: cat.box_art_url.to_string(),
        })
        .collect();

    Ok(games)
}

/// 自動発見チャンネルを手動登録に昇格
#[tauri::command]
pub async fn promote_discovered_channel(
    db_manager: State<'_, DatabaseManager>,
    app_handle: AppHandle,
    channel_id: String, // Twitch user_id
) -> Result<(), String> {
    use crate::commands::channels::{add_channel, AddChannelRequest};

    // メモリキャッシュから該当するストリーム情報を取得
    let cache: tauri::State<'_, Arc<crate::DiscoveredStreamsCache>> = app_handle.state();
    let streams_lock = cache.streams.lock().await;
    let stream_info = streams_lock
        .iter()
        .find(|s| s.twitch_user_id.to_string() == channel_id)
        .cloned();
    drop(streams_lock);

    let stream_info =
        stream_info.ok_or_else(|| format!("Channel {} not found in cache", channel_id))?;

    let login_name = stream_info.channel_id.clone();

    // 重複チェック: 既に登録されているか確認
    let already_exists = db_manager
        .with_connection(|conn| {
            ChannelRepository::exists(conn, "twitch", &login_name)
                .db_context("check channel exists")
                .map_err(|e| e.to_string())
        })
        .await
        .db_context("get connection")
        .map_err(|e| e.to_string())?;

    if already_exists {
        // 既に登録されている場合はis_auto_discoveredフラグを更新
        db_manager
            .with_connection(|conn| {
                ChannelRepository::update_auto_discovered(
                    conn,
                    "twitch",
                    &login_name,
                    false,
                    Some(stream_info.twitch_user_id),
                )
                .db_context("update channel")
                .map_err(|e| e.to_string())
            })
            .await
            .db_context("get connection")
            .map_err(|e| e.to_string())?;

        eprintln!(
            "[Discovery] Updated existing channel {} (user_id: {}) to manual registration",
            login_name, channel_id
        );
    } else {
        // 新規登録: add_channel コマンドを使用して統一
        let request = AddChannelRequest {
            platform: db_constants::PLATFORM_TWITCH.to_string(),
            channel_id: stream_info.channel_id.clone(),
            channel_name: stream_info
                .display_name
                .clone()
                .unwrap_or(stream_info.channel_name.clone()),
            poll_interval: Some(60),
            twitch_user_id: Some(stream_info.twitch_user_id),
        };

        add_channel(app_handle.clone(), db_manager.clone(), request).await?;

        eprintln!(
            "[Discovery] Promoted channel {} (user_id: {}) to manual registration using add_channel",
            login_name, channel_id
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredStreamInfo {
    pub id: i64,
    pub twitch_user_id: i64, // 不変なTwitch user ID（内部識別者）
    pub channel_id: String,  // login（表示用）
    pub channel_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer_count: Option<i32>,
    pub follower_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcaster_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchGame {
    pub id: String,
    pub name: String,
    pub box_art_url: String,
}
