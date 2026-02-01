use crate::collectors::poller::ChannelPoller;
use crate::database::{
    models::{Channel, ChannelWithStats},
    utils, DatabaseManager,
};
use crate::error::{OptionExt, ResultExt};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddChannelRequest {
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
    pub poll_interval: Option<i32>,
    pub twitch_user_id: Option<i64>, // Twitchの不変なuser ID
}

#[tauri::command]
pub async fn add_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    request: AddChannelRequest,
) -> Result<Channel, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let poll_interval = request.poll_interval.unwrap_or(60);

    // DuckDBではRETURNING句を使用してINSERTと同時にIDを取得
    let channel_id: i64 = conn
        .query_row(
            "INSERT INTO channels (platform, channel_id, channel_name, poll_interval, twitch_user_id) 
             VALUES (?, ?, ?, ?, ?) RETURNING id",
            duckdb::params![
                &request.platform,
                &request.channel_id,
                &request.channel_name,
                poll_interval,
                request.twitch_user_id,
            ],
            |row| row.get(0),
        )
        .db_context("insert channel")
        .map_err(|e| e.to_string())?;

    let channel = get_channel_by_id(&conn, channel_id)
        .ok_or_not_found("Failed to retrieve created channel")
        .map_err(|e| e.to_string())?;

    // 有効なチャンネルであればポーリングを開始
    if channel.enabled {
        if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
            let mut poller = poller.lock().await;
            if let Err(e) = poller.start_polling(channel.clone(), &db_manager, app_handle.clone()) {
                eprintln!(
                    "Failed to start polling for new channel {}: {}",
                    channel_id, e
                );
                // エラーが発生してもチャンネル作成は成功とする
            }
        }
    }

    Ok(channel)
}

#[tauri::command]
pub async fn remove_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    id: i64,
) -> Result<(), String> {
    // 削除前にポーリングを停止
    if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
        let mut poller = poller.lock().await;
        poller.stop_polling(id);
    }

    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let id_str = id.to_string();
    conn.execute("DELETE FROM channels WHERE id = ?", [id_str.as_str()])
        .db_context("delete channel")
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    id: i64,
    channel_name: Option<String>,
    poll_interval: Option<i32>,
    enabled: Option<bool>,
) -> Result<Channel, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    // 更新前の状態を取得（有効状態の変更を検知するため）
    let old_channel = get_channel_by_id(&conn, id)
        .ok_or_not_found("Channel not found")
        .map_err(|e| e.to_string())?;

    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(name) = channel_name {
        updates.push("channel_name = ?");
        params.push(name);
    }

    if let Some(interval) = poll_interval {
        updates.push("poll_interval = ?");
        params.push(interval.to_string());
    }

    if let Some(en) = enabled {
        updates.push("enabled = ?");
        params.push(en.to_string());
    }

    if updates.is_empty() {
        return get_channel_by_id(&conn, id)
            .ok_or_not_found("Channel not found")
            .map_err(|e| e.to_string());
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");
    params.push(id.to_string());

    let query = format!("UPDATE channels SET {} WHERE id = ?", updates.join(", "));

    utils::execute_with_params(&conn, &query, &params)
        .db_context("update channel")
        .map_err(|e| e.to_string())?;

    let updated_channel = get_channel_by_id(&conn, id)
        .ok_or_not_found("Channel not found")
        .map_err(|e| e.to_string())?;

    // 有効状態が変更された場合、ポーリングを開始/停止
    if let Some(enabled) = enabled {
        if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
            let mut poller = poller.lock().await;
            if enabled && !old_channel.enabled {
                // 無効→有効になった場合、ポーリングを開始
                if let Err(e) =
                    poller.start_polling(updated_channel.clone(), &db_manager, app_handle.clone())
                {
                    eprintln!("Failed to start polling for updated channel {}: {}", id, e);
                }
            } else if !enabled && old_channel.enabled {
                // 有効→無効になった場合、ポーリングを停止
                poller.stop_polling(id);
            }
        }
    }

    Ok(updated_channel)
}

#[tauri::command]
pub async fn list_channels(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<ChannelWithStats>, String> {
    // DB接続とクエリをスコープ内で完了させる
    let channels: Vec<Channel> = {
        let conn = db_manager
            .get_connection()
            .db_context("get database connection")
            .map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, is_auto_discovered, CAST(discovered_at AS VARCHAR) as discovered_at, twitch_user_id, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels ORDER BY created_at DESC")
            .db_context("prepare statement")
            .map_err(|e| e.to_string())?;

        let channels: Result<Vec<Channel>, _> = stmt
            .query_map([], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: None,
                    profile_image_url: None,
                    enabled: row.get(4)?,
                    poll_interval: row.get(5)?,
                    follower_count: None,
                    broadcaster_type: None,
                    view_count: None,
                    is_auto_discovered: row.get(6)?,
                    discovered_at: row.get(7)?,
                    twitch_user_id: row.get(8)?,
                    created_at: Some(row.get(9)?),
                    updated_at: Some(row.get(10)?),
                })
            })
            .db_context("query channels")
            .map_err(|e| e.to_string())?
            .collect();

        channels
            .db_context("collect channels")
            .map_err(|e| e.to_string())?
    };

    // Twitch API情報を取得して統合
    let channels_with_stats = enrich_channels_with_twitch_info(channels, &app_handle).await;

    Ok(channels_with_stats)
}

/// チャンネル情報にTwitch API情報を統合
async fn enrich_channels_with_twitch_info(
    channels: Vec<Channel>,
    app_handle: &AppHandle,
) -> Vec<ChannelWithStats> {
    // Twitchチャンネルのみを抽出
    let twitch_channels: Vec<&Channel> = channels
        .iter()
        .filter(|c| c.platform == crate::constants::database::PLATFORM_TWITCH)
        .collect();

    // Twitch API クライアントを取得
    let twitch_collector = if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>()
    {
        let poller = poller.lock().await;
        poller.get_twitch_collector().cloned()
    } else {
        None
    };

    // Twitch API情報を取得
    let mut user_info_map = std::collections::HashMap::new();
    let mut stream_info_map = std::collections::HashMap::new();
    let mut follower_count_map = std::collections::HashMap::new();

    if let Some(collector) = twitch_collector {
        let api_client = collector.get_api_client();

        // twitch_user_idの有無でチャンネルを分類
        let (channels_with_user_id, channels_without_user_id): (Vec<&Channel>, Vec<&Channel>) =
            twitch_channels
                .iter()
                .partition(|c| c.twitch_user_id.is_some());

        // twitch_user_idがあるチャンネルはIDで取得
        if !channels_with_user_id.is_empty() {
            let user_ids: Vec<String> = channels_with_user_id
                .iter()
                .filter_map(|c| c.twitch_user_id.map(|id| id.to_string()))
                .collect();

            if !user_ids.is_empty() {
                let user_id_refs: Vec<&str> = user_ids.iter().map(|s| s.as_str()).collect();
                for chunk in user_id_refs.chunks(100) {
                    match api_client.get_users_by_ids(chunk).await {
                        Ok(users) => {
                            for user in users {
                                user_info_map.insert(user.id.to_string(), user);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[list_channels] Failed to fetch Twitch user info by ID: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        // twitch_user_idがないチャンネルはloginで取得（後方互換性）
        if !channels_without_user_id.is_empty() {
            let user_logins: Vec<&str> = channels_without_user_id
                .iter()
                .map(|c| c.channel_id.as_str())
                .collect();

            for chunk in user_logins.chunks(100) {
                match api_client.get_users_by_logins(chunk).await {
                    Ok(users) => {
                        for user in users {
                            user_info_map.insert(user.id.to_string(), user);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[list_channels] Failed to fetch Twitch user info by login: {}",
                            e
                        );
                    }
                }
            }
        }

        // ストリーム情報をバッチ取得
        if !twitch_channels.is_empty() {
            // user_idのリストを作成（twitch_user_idがあればそれを使用）
            let user_ids: Vec<String> = twitch_channels
                .iter()
                .filter_map(|c| {
                    c.twitch_user_id.map(|id| id.to_string()).or_else(|| {
                        user_info_map
                            .values()
                            .find(|u| u.login.to_string() == c.channel_id)
                            .map(|u| u.id.to_string())
                    })
                })
                .collect();

            let user_id_refs: Vec<&str> = user_ids.iter().map(|s| s.as_str()).collect();

            if !user_id_refs.is_empty() {
                // 100件ずつに分割してバッチリクエスト
                for chunk in user_id_refs.chunks(100) {
                    match api_client.get_streams_by_user_ids(chunk).await {
                        Ok(streams) => {
                            for stream in streams {
                                // user_idからチャンネルを逆引き
                                if let Some(channel) = twitch_channels.iter().find(|c| {
                                    c.twitch_user_id
                                        .map(|id| id.to_string() == stream.user_id.to_string())
                                        .unwrap_or_else(|| {
                                            user_info_map
                                                .get(&stream.user_id.to_string())
                                                .map(|u| u.login.to_string() == c.channel_id)
                                                .unwrap_or(false)
                                        })
                                }) {
                                    let key = channel
                                        .twitch_user_id
                                        .map(|id| id.to_string())
                                        .unwrap_or_else(|| channel.channel_id.clone());
                                    stream_info_map.insert(key, stream);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[list_channels] Failed to fetch stream info batch: {}", e);
                        }
                    }
                }

                // フォロワー数をバッチ取得
                match api_client.get_followers_batch(&user_id_refs).await {
                    Ok(followers) => {
                        for (user_id, count) in followers {
                            follower_count_map.insert(user_id, count);
                        }
                    }
                    Err(e) => {
                        eprintln!("[list_channels] Failed to fetch follower counts: {}", e);
                    }
                }
            }
        }
    }

    // チャンネル情報を統合（DBには保存しない）
    channels
        .into_iter()
        .map(|mut channel| {
            let mut is_live = false;
            let mut current_viewers = None;
            let mut current_title = None;

            if channel.platform == crate::constants::database::PLATFORM_TWITCH {
                // ユーザー情報を統合（twitch_user_idがあればそれで検索、なければloginで検索）
                let user_key = channel.twitch_user_id.map(|id| id.to_string()).or_else(|| {
                    user_info_map
                        .values()
                        .find(|u| u.login.to_string() == channel.channel_id)
                        .map(|u| u.id.to_string())
                });

                if let Some(key) = user_key.as_ref() {
                    if let Some(user) = user_info_map.get(key) {
                        channel.display_name = Some(user.display_name.to_string());
                        channel.profile_image_url =
                            user.profile_image_url.as_deref().map(|s| s.to_string());
                        channel.broadcaster_type =
                            user.broadcaster_type.as_ref().map(|bt| match bt {
                                twitch_api::types::BroadcasterType::Partner => {
                                    "partner".to_string()
                                }
                                twitch_api::types::BroadcasterType::Affiliate => {
                                    "affiliate".to_string()
                                }
                                _ => "".to_string(),
                            });

                        // フォロワー数を設定
                        channel.follower_count = follower_count_map.get(user.id.as_str()).copied();

                        // view_count は Twitch API で非推奨となり取得不可
                        channel.view_count = None;
                    }
                }

                // ストリーム情報を統合
                let stream_key = user_key.or_else(|| Some(channel.channel_id.clone()));
                if let Some(key) = stream_key.as_ref() {
                    if let Some(stream) = stream_info_map.get(key) {
                        is_live = true;
                        current_viewers = Some(stream.viewer_count as i32);
                        current_title = Some(stream.title.to_string());
                    }
                }
            }

            ChannelWithStats {
                channel,
                is_live,
                current_viewers,
                current_title,
            }
        })
        .collect()
}

#[tauri::command]
pub async fn toggle_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    id: i64,
) -> Result<Channel, String> {
    let conn = db_manager
        .get_connection()
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    // 現在の状態を取得
    let current = get_channel_by_id(&conn, id)
        .ok_or_not_found("Channel not found")
        .map_err(|e| e.to_string())?;

    let new_enabled = !current.enabled;

    let enabled_str = new_enabled.to_string();
    let id_str = id.to_string();
    conn.execute(
        "UPDATE channels SET enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [enabled_str.as_str(), id_str.as_str()],
    )
    .db_context("update channel")
    .map_err(|e| e.to_string())?;

    let updated_channel = get_channel_by_id(&conn, id)
        .ok_or_not_found("Channel not found")
        .map_err(|e| e.to_string())?;

    // ポーリングの開始/停止
    if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
        let mut poller = poller.lock().await;
        if new_enabled {
            // 有効化された場合、ポーリングを開始
            if let Err(e) =
                poller.start_polling(updated_channel.clone(), &db_manager, app_handle.clone())
            {
                eprintln!("Failed to start polling for channel {}: {}", id, e);
            }
        } else {
            // 無効化された場合、ポーリングを停止
            poller.stop_polling(id);
        }
    }

    Ok(updated_channel)
}

fn get_channel_by_id(conn: &Connection, id: i64) -> Option<Channel> {
    let mut stmt = conn
        .prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, twitch_user_id, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels WHERE id = ?")
        .ok()?;

    let id_str = id.to_string();
    let mut rows = stmt
        .query_map([id_str.as_str()], |row| {
            Ok(Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: None,
                profile_image_url: None,
                enabled: row.get(4)?,
                poll_interval: row.get(5)?,
                follower_count: None,
                broadcaster_type: None,
                view_count: None,
                is_auto_discovered: None,
                discovered_at: None,
                twitch_user_id: row.get(6)?,
                created_at: Some(row.get(7)?),
                updated_at: Some(row.get(8)?),
            })
        })
        .ok()?;

    rows.next()?.ok()
}

/// 既存のTwitchチャンネルのtwitch_user_idを更新（マイグレーション用）
#[tauri::command]
pub async fn migrate_twitch_user_ids(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<MigrationResult, String> {
    // twitch_user_idがNULLのTwitchチャンネルを取得
    let channels: Vec<(i64, String)> = {
        let conn = db_manager
            .get_connection()
            .db_context("get database connection")
            .map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, channel_id FROM channels WHERE platform = 'twitch' AND twitch_user_id IS NULL",
            )
            .db_context("prepare statement")
            .map_err(|e| e.to_string())?;

        let result: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .db_context("query channels")
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .db_context("collect channels")
            .map_err(|e| e.to_string())?;

        result
        // conn and stmt are dropped here
    };

    if channels.is_empty() {
        return Ok(MigrationResult {
            total: 0,
            updated: 0,
            failed: 0,
            message: "マイグレーション対象のチャンネルがありません".to_string(),
        });
    }

    eprintln!(
        "[Migration] Found {} Twitch channels without user_id",
        channels.len()
    );

    // Twitch APIクライアントを取得
    let twitch_collector = if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>()
    {
        let poller = poller.lock().await;
        poller.get_twitch_collector().cloned()
    } else {
        return Err("Twitch collector not initialized".to_string());
    };

    let collector = twitch_collector.ok_or_else(|| "Twitch collector not available".to_string())?;
    let api_client = collector.get_api_client();

    let mut updated = 0;
    let mut failed = 0;

    // 100件ずつバッチ処理
    for chunk in channels.chunks(100) {
        let logins: Vec<&str> = chunk.iter().map(|(_, login)| login.as_str()).collect();

        match api_client.get_users_by_logins(&logins).await {
            Ok(users) => {
                for user in users {
                    let login = user.login.to_string();
                    let user_id_str = user.id.to_string();
                    let user_id: i64 = match user_id_str.parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("[Migration] Failed to parse user_id for {}: {}", login, e);
                            failed += 1;
                            continue;
                        }
                    };

                    // 該当するチャンネルのIDを取得
                    if let Some((channel_id, _)) = chunk.iter().find(|(_, l)| l == &login) {
                        let conn = db_manager
                            .get_connection()
                            .db_context("get connection for update")
                            .map_err(|e| e.to_string())?;

                        match conn.execute(
                            "UPDATE channels SET twitch_user_id = ? WHERE id = ?",
                            duckdb::params![user_id, channel_id],
                        ) {
                            Ok(_) => {
                                eprintln!(
                                    "[Migration] Updated channel {} (login: {}) with user_id: {}",
                                    channel_id, login, user_id
                                );
                                updated += 1;
                            }
                            Err(e) => {
                                eprintln!(
                                    "[Migration] Failed to update channel {}: {}",
                                    channel_id, e
                                );
                                failed += 1;
                            }
                        }
                        // Drop conn before next iteration
                        drop(conn);
                    }
                }
            }
            Err(e) => {
                eprintln!("[Migration] Failed to fetch user info batch: {}", e);
                failed += logins.len();
            }
        }
    }

    Ok(MigrationResult {
        total: channels.len(),
        updated,
        failed,
        message: format!(
            "マイグレーション完了: {}/{} チャンネルを更新しました（失敗: {}）",
            updated,
            channels.len(),
            failed
        ),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationResult {
    pub total: usize,
    pub updated: usize,
    pub failed: usize,
    pub message: String,
}
