use crate::collectors::poller::ChannelPoller;
use crate::database::{
    models::{Channel, ChannelWithStats},
    repositories::{channel_repository::CreateChannelParams, ChannelRepository},
    DatabaseManager,
};
use crate::error::{OptionExt, ResultExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddChannelRequest {
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitch_user_id: Option<i64>, // Twitchの不変なuser ID
}

#[tauri::command]
pub async fn add_channel(
    app_handle: AppHandle,
    db_manager: State<'_, DatabaseManager>,
    request: AddChannelRequest,
) -> Result<Channel, String> {
    let poll_interval = request.poll_interval.unwrap_or(60);

    let channel = db_manager
        .with_connection(|conn| {
            // チャンネルを作成
            let channel_id = ChannelRepository::create(
                conn,
                CreateChannelParams {
                    platform: request.platform,
                    channel_id: request.channel_id,
                    channel_name: request.channel_name,
                    poll_interval,
                    twitch_user_id: request.twitch_user_id,
                },
            )
            .db_context("create channel")
            .map_err(|e| e.to_string())?;

            ChannelRepository::get_by_id(conn, channel_id)
                .db_context("get created channel")
                .map_err(|e| e.to_string())?
                .ok_or_not_found("Failed to retrieve created channel")
                .map_err(|e| e.to_string())
        })
        .await?;

    // 有効なチャンネルであればポーリングを開始
    if channel.enabled {
        if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
            let mut poller = poller.lock().await;
            if let Err(e) = poller.start_polling(channel.clone(), &db_manager, app_handle.clone()) {
                eprintln!(
                    "Failed to start polling for new channel {}: {}",
                    channel.id.unwrap_or(0),
                    e
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
        poller.stop_polling(id).await;
    }

    db_manager
        .with_connection(|conn| {
            ChannelRepository::delete_channel_and_related(conn, id)
                .db_context("delete channel and related data")
                .map_err(|e| e.to_string())
        })
        .await?;
    eprintln!(
        "[remove_channel] Successfully deleted channel {} and related data",
        id
    );
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
    let (old_channel, updated_channel) = db_manager
        .with_connection(|conn| {
            // 更新前の状態を取得（有効状態の変更を検知するため）
            let old_channel = ChannelRepository::get_by_id(conn, id)
                .db_context("get channel")
                .map_err(|e| e.to_string())?
                .ok_or_not_found("Channel not found")
                .map_err(|e| e.to_string())?;

            if channel_name.is_some() || poll_interval.is_some() || enabled.is_some() {
                ChannelRepository::update(conn, id, channel_name, poll_interval, enabled)
                    .db_context("update channel")
                    .map_err(|e| e.to_string())?;
            }

            let updated_channel = ChannelRepository::get_by_id(conn, id)
                .db_context("get updated channel")
                .map_err(|e| e.to_string())?
                .ok_or_not_found("Channel not found")
                .map_err(|e| e.to_string())?;

            Ok::<(Channel, Channel), String>((old_channel, updated_channel))
        })
        .await?;

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
                poller.stop_polling(id).await;
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
    let channels: Vec<Channel> = db_manager
        .with_connection(|conn| {
            ChannelRepository::list_all(conn)
                .db_context("list all channels")
                .map_err(|e| e.to_string())
        })
        .await?;

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

    // Twitch API クライアントを取得（タイムアウト付き）
    let twitch_collector = if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>()
    {
        // 15秒のタイムアウトでロック取得を試みる（初期化完了まで待つ）
        match tokio::time::timeout(std::time::Duration::from_secs(15), poller.lock()).await {
            Ok(poller) => poller.get_twitch_collector().cloned(),
            Err(_) => None,
        }
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
                    if let Ok(users) = api_client.get_users_by_ids(chunk).await {
                        for user in users {
                            user_info_map.insert(user.id.to_string(), user);
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
                if let Ok(users) = api_client.get_users_by_logins(chunk).await {
                    for user in users {
                        user_info_map.insert(user.id.to_string(), user);
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
                    if let Ok(streams) = api_client.get_streams_by_user_ids(chunk).await {
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
                }

                // フォロワー数をバッチ取得
                if let Ok(followers) = api_client.get_followers_batch(&user_id_refs).await {
                    for (user_id, count) in followers {
                        follower_count_map.insert(user_id, count);
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
                        channel.display_name = user.display_name.to_string();
                        channel.profile_image_url =
                            user.profile_image_url.as_deref().unwrap_or("").to_string();
                        channel.broadcaster_type = user
                            .broadcaster_type
                            .as_ref()
                            .map(|bt| match bt {
                                twitch_api::types::BroadcasterType::Partner => {
                                    "partner".to_string()
                                }
                                twitch_api::types::BroadcasterType::Affiliate => {
                                    "affiliate".to_string()
                                }
                                _ => "".to_string(),
                            })
                            .unwrap_or_else(|| "".to_string());

                        // フォロワー数を設定
                        channel.follower_count = follower_count_map
                            .get(user.id.as_str())
                            .copied()
                            .unwrap_or(0);

                        // view_count は Twitch API で非推奨となり取得不可
                        channel.view_count = 0;
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
    let (new_enabled, updated_channel) = db_manager
        .with_connection(|conn| {
            // 現在の状態を取得
            let current = ChannelRepository::get_by_id(conn, id)
                .db_context("get channel")
                .map_err(|e| e.to_string())?
                .ok_or_not_found("Channel not found")
                .map_err(|e| e.to_string())?;

            let new_enabled = !current.enabled;

            ChannelRepository::update_enabled(conn, id, new_enabled)
                .db_context("update channel")
                .map_err(|e| e.to_string())?;

            let updated_channel = ChannelRepository::get_by_id(conn, id)
                .db_context("get updated channel")
                .map_err(|e| e.to_string())?
                .ok_or_not_found("Channel not found")
                .map_err(|e| e.to_string())?;

            Ok::<(bool, Channel), String>((new_enabled, updated_channel))
        })
        .await?;

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
            poller.stop_polling(id).await;
        }
    }

    Ok(updated_channel)
}
