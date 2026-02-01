use crate::collectors::poller::ChannelPoller;
use crate::database::{
    models::{Channel, ChannelWithStats},
    utils, DatabaseManager,
};
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
}

#[tauri::command]
pub async fn add_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    request: AddChannelRequest,
) -> Result<Channel, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let poll_interval = request.poll_interval.unwrap_or(60);

    // DuckDBではRETURNING句を使用してINSERTと同時にIDを取得
    let channel_id: i64 = conn
        .query_row(
            "INSERT INTO channels (platform, channel_id, channel_name, poll_interval) 
             VALUES (?, ?, ?, ?) RETURNING id",
            duckdb::params![
                &request.platform,
                &request.channel_id,
                &request.channel_name,
                poll_interval,
            ],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to insert channel: {}", e))?;

    let channel = get_channel_by_id(&conn, channel_id)
        .ok_or_else(|| "Failed to retrieve created channel".to_string())?;

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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let id_str = id.to_string();
    conn.execute("DELETE FROM channels WHERE id = ?", [id_str.as_str()])
        .map_err(|e| format!("Failed to delete channel: {}", e))?;

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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // 更新前の状態を取得（有効状態の変更を検知するため）
    let old_channel =
        get_channel_by_id(&conn, id).ok_or_else(|| "Channel not found".to_string())?;

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
        return get_channel_by_id(&conn, id).ok_or_else(|| "Channel not found".to_string());
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");
    params.push(id.to_string());

    let query = format!("UPDATE channels SET {} WHERE id = ?", updates.join(", "));

    utils::execute_with_params(&conn, &query, &params)
        .map_err(|e| format!("Failed to update channel: {}", e))?;

    let updated_channel =
        get_channel_by_id(&conn, id).ok_or_else(|| "Channel not found".to_string())?;

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
    let channels = {
        let conn = db_manager
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, platform, channel_id, channel_name, display_name, profile_image_url, enabled, poll_interval, follower_count, broadcaster_type, view_count, is_auto_discovered, CAST(discovered_at AS VARCHAR) as discovered_at, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let channels: Result<Vec<Channel>, _> = stmt
            .query_map([], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: row.get(4)?,
                    profile_image_url: row.get(5)?,
                    enabled: row.get(6)?,
                    poll_interval: row.get(7)?,
                    follower_count: row.get(8)?,
                    broadcaster_type: row.get(9)?,
                    view_count: row.get(10)?,
                    is_auto_discovered: row.get(11)?,
                    discovered_at: row.get(12)?,
                    created_at: Some(row.get(13)?),
                    updated_at: Some(row.get(14)?),
                })
            })
            .map_err(|e| format!("Failed to query channels: {}", e))?
            .collect();

        channels.map_err(|e| format!("Failed to collect channels: {}", e))?
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
    let twitch_channels: Vec<&Channel> =
        channels.iter().filter(|c| c.platform == "twitch").collect();

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

    if let Some(collector) = twitch_collector {
        let api_client = collector.get_api_client();

        // ユーザー情報をバッチ取得（最大100件ずつ）
        if !twitch_channels.is_empty() {
            let user_logins: Vec<&str> = twitch_channels
                .iter()
                .map(|c| c.channel_id.as_str())
                .collect();

            // 100件ずつに分割してリクエスト
            for chunk in user_logins.chunks(100) {
                match api_client.get_users_by_logins(chunk).await {
                    Ok(users) => {
                        for user in users {
                            user_info_map.insert(user.login.to_string(), user);
                        }
                    }
                    Err(e) => {
                        eprintln!("[list_channels] Failed to fetch Twitch user info: {}", e);
                    }
                }
            }
        }

        // ストリーム情報をバッチ取得
        if !twitch_channels.is_empty() {
            // user_idのリストを作成
            let user_ids: Vec<&str> = twitch_channels
                .iter()
                .filter_map(|c| user_info_map.get(&c.channel_id).map(|u| u.id.as_str()))
                .collect();

            if !user_ids.is_empty() {
                // 100件ずつに分割してバッチリクエスト
                for chunk in user_ids.chunks(100) {
                    match api_client.get_streams_by_user_ids(chunk).await {
                        Ok(streams) => {
                            for stream in streams {
                                // user_idからchannel_idを逆引き
                                if let Some(channel) = twitch_channels.iter().find(|c| {
                                    user_info_map
                                        .get(&c.channel_id)
                                        .map(|u| u.id.as_str() == stream.user_id.as_str())
                                        .unwrap_or(false)
                                }) {
                                    stream_info_map.insert(channel.channel_id.clone(), stream);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[list_channels] Failed to fetch stream info batch: {}", e);
                        }
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

            if channel.platform == "twitch" {
                // ユーザー情報を統合
                if let Some(user) = user_info_map.get(&channel.channel_id) {
                    channel.display_name = Some(user.display_name.to_string());
                    channel.profile_image_url =
                        user.profile_image_url.as_deref().map(|s| s.to_string());
                    channel.broadcaster_type = user.broadcaster_type.as_ref().map(|bt| match bt {
                        twitch_api::types::BroadcasterType::Partner => "partner".to_string(),
                        twitch_api::types::BroadcasterType::Affiliate => "affiliate".to_string(),
                        _ => "".to_string(),
                    });
                    // follower_count は Twitch Helix API で直接取得不可（別エンドポイントが必要）
                    // view_count は Twitch API で非推奨となり取得不可
                    channel.follower_count = None;
                    channel.view_count = None;
                }

                // ストリーム情報を統合
                if let Some(stream) = stream_info_map.get(&channel.channel_id) {
                    is_live = true;
                    current_viewers = Some(stream.viewer_count as i32);
                    current_title = Some(stream.title.to_string());
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    // 現在の状態を取得
    let current = get_channel_by_id(&conn, id).ok_or_else(|| "Channel not found".to_string())?;

    let new_enabled = !current.enabled;

    let enabled_str = new_enabled.to_string();
    let id_str = id.to_string();
    conn.execute(
        "UPDATE channels SET enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        [enabled_str.as_str(), id_str.as_str()],
    )
    .map_err(|e| format!("Failed to update channel: {}", e))?;

    let updated_channel =
        get_channel_by_id(&conn, id).ok_or_else(|| "Channel not found".to_string())?;

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
        .prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels WHERE id = ?")
        .ok()?;

    let id_str = id.to_string();
    let mut rows = stmt
        .query_map([id_str.as_str()], |row| {
            Ok(Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: None, // TODO: データベースから取得
                profile_image_url: None,
                enabled: row.get(4)?,
                poll_interval: row.get(5)?,
                follower_count: None,
                broadcaster_type: None,
                view_count: None,
                is_auto_discovered: None,
                discovered_at: None,
                created_at: Some(row.get(6)?),
                updated_at: Some(row.get(7)?),
            })
        })
        .ok()?;

    rows.next()?.ok()
}
