use crate::config::settings::SettingsManager;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use twitch_api::{
    helix::{
        channels::GetChannelFollowersRequest,
        users::GetUsersRequest,
        HelixClient,
    },
    twitch_oauth2::{AccessToken, UserToken as TwitchApiUserToken},
    types,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchChannelInfo {
    pub channel_id: String,
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
        .map_err(|e| format!("設定の読み込みに失敗しました: {}", e))?;

    let _client_id = settings
        .twitch
        .client_id
        .ok_or_else(|| "Twitch Client IDが設定されていません。設定画面からOAuth設定を行ってください。".to_string())?;

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

    let response = client
        .req_get(request, &token)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("401") || error_msg.contains("Unauthorized") {
                "認証トークンが無効です。設定画面から再度認証を行ってください。".to_string()
            } else {
                format!("Twitch APIエラー: {}", error_msg)
            }
        })?;

    let user = response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| format!("チャンネル '{}' が見つかりません。正しいチャンネルIDを入力してください。", channel_id))?;

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
    let broadcaster_type = user.broadcaster_type.map(|bt| format!("{:?}", bt).to_lowercase());

    Ok(TwitchChannelInfo {
        channel_id: user.login.to_string(),
        display_name: user.display_name.to_string(),
        profile_image_url: user.profile_image_url.map(|url| url.to_string()).unwrap_or_default(),
        description: user.description.unwrap_or_default(),
        follower_count,
        broadcaster_type,
    })
}
