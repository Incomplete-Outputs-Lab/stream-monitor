use crate::collectors::poller::ChannelPoller;
use crate::database::{models::Channel, utils, DatabaseManager};
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
    pub display_name: Option<String>,
    pub profile_image_url: Option<String>,
    pub poll_interval: Option<i32>,
    pub follower_count: Option<i32>,
    pub broadcaster_type: Option<String>,
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

    let poll_interval_str = poll_interval.to_string();
    
    // パラメータを準備
    let mut sql_parts = vec!["platform", "channel_id", "channel_name", "poll_interval"];
    let mut sql_values = vec!["?", "?", "?", "?"];
    let mut params: Vec<String> = vec![
        request.platform.clone(),
        request.channel_id.clone(),
        request.channel_name.clone(),
        poll_interval_str.clone(),
    ];

    if let Some(ref display_name) = request.display_name {
        sql_parts.push("display_name");
        sql_values.push("?");
        params.push(display_name.clone());
    }

    if let Some(ref profile_image_url) = request.profile_image_url {
        sql_parts.push("profile_image_url");
        sql_values.push("?");
        params.push(profile_image_url.clone());
    }

    if let Some(follower_count) = request.follower_count {
        sql_parts.push("follower_count");
        sql_values.push("?");
        params.push(follower_count.to_string());
    }

    if let Some(ref broadcaster_type) = request.broadcaster_type {
        sql_parts.push("broadcaster_type");
        sql_values.push("?");
        params.push(broadcaster_type.clone());
    }

    let sql = format!(
        "INSERT INTO channels ({}) VALUES ({}) RETURNING id",
        sql_parts.join(", "),
        sql_values.join(", ")
    );

    // DuckDBのparamsは固定サイズの配列しか受け付けないため、動的にSQLを実行
    let channel_id: i64 = match params.len() {
        4 => conn.query_row(&sql, [params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str()], |row| row.get(0)),
        5 => conn.query_row(&sql, [params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str(), params[4].as_str()], |row| row.get(0)),
        6 => conn.query_row(&sql, [params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str(), params[4].as_str(), params[5].as_str()], |row| row.get(0)),
        7 => conn.query_row(&sql, [params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str(), params[4].as_str(), params[5].as_str(), params[6].as_str()], |row| row.get(0)),
        8 => conn.query_row(&sql, [params[0].as_str(), params[1].as_str(), params[2].as_str(), params[3].as_str(), params[4].as_str(), params[5].as_str(), params[6].as_str(), params[7].as_str()], |row| row.get(0)),
        _ => return Err(format!("Invalid number of parameters: {}", params.len())),
    }.map_err(|e| format!("Failed to insert channel: {}", e))?;

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
                if let Err(e) = poller.start_polling(updated_channel.clone(), &db_manager, app_handle.clone()) {
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
    _app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<Channel>, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, platform, channel_id, channel_name, display_name, profile_image_url, enabled, poll_interval, follower_count, broadcaster_type, view_count, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels ORDER BY created_at DESC")
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
                follower_count: row.get(8).ok(),
                broadcaster_type: row.get(9).ok(),
                view_count: row.get(10).ok(),
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })
        .map_err(|e| format!("Failed to query channels: {}", e))?
        .collect();

    channels.map_err(|e| format!("Failed to collect channels: {}", e))
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
            if let Err(e) = poller.start_polling(updated_channel.clone(), &db_manager, app_handle.clone()) {
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
        .prepare("SELECT id, platform, channel_id, channel_name, display_name, profile_image_url, enabled, poll_interval, follower_count, broadcaster_type, view_count, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels WHERE id = ?")
        .ok()?;

    let id_str = id.to_string();
    let mut rows = stmt
        .query_map([id_str.as_str()], |row| {
            Ok(Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: row.get(4)?,
                profile_image_url: row.get(5)?,
                enabled: row.get(6)?,
                poll_interval: row.get(7)?,
                follower_count: row.get(8).ok(),
                broadcaster_type: row.get(9).ok(),
                view_count: row.get(10).ok(),
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })
        .ok()?;

    rows.next()?.ok()
}
