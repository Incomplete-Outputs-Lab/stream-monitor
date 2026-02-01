use crate::api::twitch_api::TwitchRateLimitStatus;
use crate::collectors::poller::ChannelPoller;
use crate::config::settings::SettingsManager;
use crate::constants::twitch;
use crate::error::ResultExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;
use twitch_api::{
    helix::{channels::GetChannelFollowersRequest, users::GetUsersRequest, HelixClient},
    twitch_oauth2::{AccessToken, UserToken as TwitchApiUserToken},
    types,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchChannelInfo {
    pub channel_id: String,  // login (表示用)
    pub twitch_user_id: i64, // 不変なuser ID
    pub display_name: String,
    pub profile_image_url: String,
    pub description: String,
    pub follower_count: Option<i32>,
    pub broadcaster_type: Option<String>,
}

/// Validate a Twitch channel by checking if it exists via Twitch API
/// Note: access_token is passed from frontend (retrieved from Stronghold via JS API)
#[tauri::command]
pub async fn validate_twitch_channel(
    app_handle: AppHandle,
    channel_id: String,
    access_token: Option<String>,
) -> Result<TwitchChannelInfo, String> {
    // Load settings to get client_id
    let settings = SettingsManager::load_settings(&app_handle)
        .config_context("load settings")
        .map_err(|e| format!("設定の読み込みに失敗しました: {}", e))?;

    let _client_id = settings.twitch.client_id.ok_or_else(|| {
        "Twitch Client IDが設定されていません。設定画面からOAuth設定を行ってください。".to_string()
    })?;

    // Check if access token is provided
    let token_str = access_token
        .ok_or_else(|| "Twitchの認証が必要です。設定画面から認証を行ってください。".to_string())?;

    // Create Twitch API client
    let client: HelixClient<'static, reqwest::Client> = HelixClient::default();
    let token = TwitchApiUserToken::from_token(&client, AccessToken::from(token_str))
        .await
        .map_err(|e| format!("トークンの検証に失敗しました: {}", e))?;

    // Get user information from Twitch API
    let login_refs: &[&types::UserNameRef] = &[channel_id.as_str().into()];
    let request = GetUsersRequest::logins(login_refs);

    let response = client.req_get(request, &token).await.map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("401") || error_msg.contains("Unauthorized") {
            "認証トークンが無効です。設定画面から再度認証を行ってください。".to_string()
        } else {
            format!("Twitch APIエラー: {}", error_msg)
        }
    })?;

    let user = response.data.into_iter().next().ok_or_else(|| {
        format!(
            "チャンネル '{}' が見つかりません。正しいチャンネルIDを入力してください。",
            channel_id
        )
    })?;

    // Get follower count
    let broadcaster_id_ref: &types::UserIdRef = user.id.as_str().into();
    let follower_request = GetChannelFollowersRequest::broadcaster_id(broadcaster_id_ref);
    let follower_count = match client.req_get(follower_request, &token).await {
        Ok(response) => Some(response.total.unwrap_or(0) as i32),
        Err(e) => {
            eprintln!("Failed to get follower count: {}", e);
            None
        }
    };

    // Get broadcaster type
    let broadcaster_type = user
        .broadcaster_type
        .map(|bt| format!("{:?}", bt).to_lowercase());

    // Parse user_id as i64
    let twitch_user_id = user
        .id
        .as_str()
        .parse::<i64>()
        .map_err(|e| format!("Failed to parse Twitch user ID: {}", e))?;

    Ok(TwitchChannelInfo {
        channel_id: user.login.to_string(),
        twitch_user_id,
        display_name: user.display_name.to_string(),
        profile_image_url: user
            .profile_image_url
            .map(|url| url.to_string())
            .unwrap_or_default(),
        description: user.description.unwrap_or_default(),
        follower_count,
        broadcaster_type,
    })
}

/// Twitch APIレート制限の現在のステータスを取得
#[tauri::command]
pub async fn get_twitch_rate_limit_status(
    poller: State<'_, Arc<Mutex<ChannelPoller>>>,
) -> Result<TwitchRateLimitStatus, String> {
    let poller_guard = poller.lock().await;

    // TwitchCollectorからrate_limiterを取得
    let result = if let Some(twitch_collector) = poller_guard.get_twitch_collector() {
        let api_client = twitch_collector.get_api_client();
        let rate_limiter = Arc::clone(&api_client.get_rate_limiter());

        // pollerのロックを早期に解放
        drop(poller_guard);

        let status = match rate_limiter.lock() {
            Ok(limiter) => Ok(limiter.get_status()),
            Err(e) => Err(format!("レート制限トラッカーのロックに失敗しました: {}", e)),
        };
        status
    } else {
        // TwitchCollectorが初期化されていない場合、デフォルト値を返す
        Ok(TwitchRateLimitStatus {
            points_used: 0,
            bucket_capacity: twitch::RATE_LIMIT_BUCKET_CAPACITY as u32,
            points_remaining: twitch::RATE_LIMIT_BUCKET_CAPACITY as u32,
            oldest_entry_expires_in_seconds: None,
            usage_percent: 0.0,
            request_count: 0,
        })
    };

    result
}
