use crate::collectors::poller::ChannelPoller;
use crate::database::{get_connection, models::Channel, utils};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
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
    request: AddChannelRequest,
) -> Result<Channel, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let poll_interval = request.poll_interval.unwrap_or(60);

    let poll_interval_str = poll_interval.to_string();
    conn.execute(
        "INSERT INTO channels (platform, channel_id, channel_name, poll_interval) 
         VALUES (?, ?, ?, ?)",
        [
            request.platform.as_str(),
            request.channel_id.as_str(),
            request.channel_name.as_str(),
            poll_interval_str.as_str(),
        ],
    )
    .map_err(|e| format!("Failed to insert channel: {}", e))?;

    // DuckDBでは、last_insert_rowid()を直接取得できないため、SELECTを使用
    let channel_id: i64 = conn
        .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get last insert rowid: {}", e))?;

    let channel = get_channel_by_id(&conn, channel_id)
        .ok_or_else(|| "Failed to retrieve created channel".to_string())?;

    // 有効なチャンネルであればポーリングを開始
    if channel.enabled {
        if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
            let mut poller = poller.lock().await;
            if let Err(e) = poller.start_polling(channel.clone()) {
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
pub async fn remove_channel(app_handle: AppHandle, id: i64) -> Result<(), String> {
    // 削除前にポーリングを停止
    if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
        let mut poller = poller.lock().await;
        poller.stop_polling(id);
    }

    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let id_str = id.to_string();
    conn.execute("DELETE FROM channels WHERE id = ?", [id_str.as_str()])
        .map_err(|e| format!("Failed to delete channel: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_channel(
    app_handle: AppHandle,
    id: i64,
    channel_name: Option<String>,
    poll_interval: Option<i32>,
    enabled: Option<bool>,
) -> Result<Channel, String> {
    let conn = get_connection(&app_handle)
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
                if let Err(e) = poller.start_polling(updated_channel.clone()) {
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
pub async fn list_channels(app_handle: AppHandle) -> Result<Vec<Channel>, String> {
    let conn = get_connection(&app_handle)
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, platform, channel_id, channel_name, enabled, poll_interval, CAST(created_at AS VARCHAR) as created_at, CAST(updated_at AS VARCHAR) as updated_at FROM channels ORDER BY created_at DESC")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let channels: Result<Vec<Channel>, _> = stmt
        .query_map([], |row| {
            Ok(Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: None,
                enabled: row.get(4)?,
                poll_interval: row.get(5)?,
                created_at: Some(row.get(6)?),
                updated_at: Some(row.get(7)?),
            })
        })
        .map_err(|e| format!("Failed to query channels: {}", e))?
        .collect();

    channels.map_err(|e| format!("Failed to collect channels: {}", e))
}

#[tauri::command]
pub async fn toggle_channel(app_handle: AppHandle, id: i64) -> Result<Channel, String> {
    let conn = get_connection(&app_handle)
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
            if let Err(e) = poller.start_polling(updated_channel.clone()) {
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
                enabled: row.get(4)?,
                poll_interval: row.get(5)?,
                created_at: Some(row.get(6)?),
                updated_at: Some(row.get(7)?),
            })
        })
        .ok()?;

    rows.next()?.ok()
}
