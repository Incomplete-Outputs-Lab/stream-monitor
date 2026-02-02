use crate::config::keyring_store::KeyringStore;
use crate::constants::{database as db_constants, twitch};
use crate::oauth::twitch::TwitchOAuth;
use chrono::{DateTime, Local};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use twitch_api::{
    helix::{
        streams::{GetStreamsRequest, Stream},
        users::{GetUsersRequest, User},
        HelixClient,
    },
    twitch_oauth2::{AccessToken, UserToken as TwitchApiUserToken},
    types,
};
use twitch_oauth2::{AppAccessToken, ClientId, ClientSecret};

pub struct TwitchApiClient {
    client: Arc<HelixClient<'static, reqwest::Client>>,
    client_id: String,
    client_secret: Option<String>,
    app_handle: Option<tauri::AppHandle>,
    rate_limiter: Arc<Mutex<TwitchRateLimitTracker>>,
}

impl TwitchApiClient {
    /// Create a new TwitchApiClient
    ///
    /// For Device Code Flow (user authentication), client_secret can be None.
    /// For App Access Token (client credentials flow), client_secret is required.
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        let client = Arc::new(HelixClient::default());

        Self {
            client,
            client_id,
            client_secret,
            app_handle: None,
            rate_limiter: Arc::new(Mutex::new(TwitchRateLimitTracker::new())),
        }
    }

    /// レート制限トラッカーの参照を取得
    pub fn get_rate_limiter(&self) -> Arc<Mutex<TwitchRateLimitTracker>> {
        Arc::clone(&self.rate_limiter)
    }

    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    async fn get_access_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        // Keyringからトークンを取得を試みる（Device Code Flowで取得したユーザートークン）
        if let Some(ref handle) = self.app_handle {
            if let Ok(token_str) =
                KeyringStore::get_token_with_app(handle, db_constants::PLATFORM_TWITCH)
            {
                return Ok(AccessToken::from(token_str));
            }
        }

        // Client Secretがある場合のみ、OAuth 2.0 Client Credentials Flowを試行
        if let Some(ref client_secret) = self.client_secret {
            let client_id = ClientId::new(self.client_id.clone());
            let client_secret = ClientSecret::new(client_secret.clone());

            let http_client = reqwest::Client::new();
            let app_token = AppAccessToken::get_app_access_token(
                &http_client,
                client_id,
                client_secret,
                vec![],
            )
            .await?;

            let access_token_str = app_token.access_token.secret().to_string();

            // トークンを保存
            if let Some(ref handle) = self.app_handle {
                KeyringStore::save_token_with_app(
                    handle,
                    db_constants::PLATFORM_TWITCH,
                    &access_token_str,
                )?;
            }

            return Ok(AccessToken::from(access_token_str));
        }

        // トークンが見つからず、Client Secretもない場合はエラー
        Err(
            "No Twitch access token found. Please authenticate using Device Code Flow first."
                .into(),
        )
    }

    /// トークンをリフレッシュ
    async fn refresh_token(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        // TwitchOAuthインスタンスを作成してリフレッシュ
        let mut oauth = TwitchOAuth::new(
            self.client_id.clone(),
            String::new(), // Device Code Flowでは不要
        );

        if let Some(ref handle) = self.app_handle {
            oauth = oauth.with_app_handle(handle.clone());
        }

        // リフレッシュ時はイベント通知なし（バックグラウンド処理のため）
        let new_token = oauth.refresh_device_token(self.app_handle.clone()).await?;
        Ok(AccessToken::from(new_token))
    }

    async fn get_user_token(&self) -> Result<TwitchApiUserToken, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;

        // トークン検証を試行（これもAPIコールなのでトラッキング）
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match TwitchApiUserToken::from_token(&*self.client, access_token).await {
            Ok(token) => Ok(token),
            Err(e) => {
                // トークン検証失敗 - リフレッシュを試行
                eprintln!(
                    "[TwitchAPI] Token validation failed: {}, attempting refresh...",
                    e
                );

                // リフレッシュトークンの存在確認
                if let Some(ref handle) = self.app_handle {
                    if KeyringStore::get_token_with_app(handle, "twitch_refresh").is_err() {
                        return Err(
                            "Refresh token not found. Please re-authenticate via Device Code Flow."
                                .into(),
                        );
                    }
                } else {
                    return Err("No app handle available for token refresh.".into());
                }

                // トークンリフレッシュ実行
                let new_token = self.refresh_token().await?;

                // 再度検証（これもAPIコールなのでトラッキング）
                if let Ok(mut limiter) = self.rate_limiter.lock() {
                    limiter.track_request();
                }

                TwitchApiUserToken::from_token(&*self.client, new_token)
                    .await
                    .map_err(|e| {
                        format!(
                            "Token validation failed after refresh: {}. Please re-authenticate.",
                            e
                        )
                        .into()
                    })
            }
        }
    }

    pub async fn authenticate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // トークンの取得と検証
        let _token = self.get_access_token().await?;
        Ok(())
    }

    /// トークンの有効期限をチェックし、期限が近い場合はリフレッシュ
    ///
    /// Returns: Ok(true) if token was refreshed, Ok(false) if refresh was not needed
    pub async fn check_and_refresh_token_if_needed(
        &self,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let handle = self.app_handle.as_ref().ok_or("No app handle available")?;

        // メタデータを取得
        let metadata = match KeyringStore::get_token_metadata_with_app(
            handle,
            db_constants::PLATFORM_TWITCH,
        ) {
            Ok(m) => m,
            Err(_) => {
                // メタデータがない場合は、トークンが古い形式で保存されている可能性
                eprintln!("[TwitchAPI] Token metadata not found, skipping proactive refresh");
                return Ok(false);
            }
        };

        let expires_at = match DateTime::parse_from_rfc3339(&metadata.expires_at) {
            Ok(dt) => dt,
            Err(e) => {
                eprintln!("[TwitchAPI] Failed to parse expiration time: {}", e);
                return Ok(false);
            }
        };

        let now = Local::now();
        let time_until_expiry = expires_at.signed_duration_since(now);
        let minutes_until_expiry = time_until_expiry.num_minutes();

        // 有効期限まで30分以内の場合はリフレッシュ
        if minutes_until_expiry < 30 {
            eprintln!(
                "[TwitchAPI] Token expires in {} minutes, refreshing proactively...",
                minutes_until_expiry
            );

            // リフレッシュトークンの存在確認
            if KeyringStore::get_token_with_app(handle, "twitch_refresh").is_err() {
                eprintln!("[TwitchAPI] No refresh token available, cannot refresh proactively");
                return Ok(false);
            }

            // トークンリフレッシュ
            match self.refresh_token().await {
                Ok(_) => {
                    eprintln!("[TwitchAPI] Token refreshed successfully (proactive)");
                    Ok(true)
                }
                Err(e) => {
                    eprintln!("[TwitchAPI] Failed to refresh token proactively: {}", e);
                    Err(e)
                }
            }
        } else {
            // まだ有効期限まで余裕がある
            Ok(false)
        }
    }

    pub async fn get_user_by_login(&self, login: &str) -> Result<User, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let login_refs: &[&types::UserNameRef] = &[login.into()];
        let request = GetUsersRequest::logins(login_refs);

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match self.client.req_get(request, &token).await {
            Ok(response) => response
                .data
                .into_iter()
                .next()
                .ok_or_else(|| "User not found".into()),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let response = self
                        .client
                        .req_get(GetUsersRequest::logins(login_refs), &refreshed_token)
                        .await?;
                    response
                        .data
                        .into_iter()
                        .next()
                        .ok_or_else(|| "User not found".into())
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub async fn get_stream_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<Stream>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let user_id_refs: &[&types::UserIdRef] = &[user_id.into()];
        let request = GetStreamsRequest::user_ids(user_id_refs);

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match self.client.req_get(request, &token).await {
            Ok(response) => Ok(response.data.into_iter().next()),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let response = self
                        .client
                        .req_get(GetStreamsRequest::user_ids(user_id_refs), &refreshed_token)
                        .await?;
                    Ok(response.data.into_iter().next())
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// 複数のユーザーIDからストリーム情報をバッチ取得
    pub async fn get_streams_by_user_ids(
        &self,
        user_ids: &[&str],
    ) -> Result<Vec<Stream>, Box<dyn std::error::Error>> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let token = self.get_user_token().await?;

        let user_id_refs: Vec<&types::UserIdRef> = user_ids.iter().map(|id| (*id).into()).collect();
        let request = GetStreamsRequest::user_ids(user_id_refs.as_slice());

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match self.client.req_get(request, &token).await {
            Ok(response) => Ok(response.data),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let response = self
                        .client
                        .req_get(
                            GetStreamsRequest::user_ids(user_id_refs.as_slice()),
                            &refreshed_token,
                        )
                        .await?;
                    Ok(response.data)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// 複数のユーザーIDからユーザー情報を取得
    pub async fn get_users_by_ids(
        &self,
        user_ids: &[&str],
    ) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let user_id_refs: Vec<&types::UserIdRef> = user_ids.iter().map(|id| (*id).into()).collect();
        let request = GetUsersRequest::ids(user_id_refs.as_slice());

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match self.client.req_get(request, &token).await {
            Ok(response) => Ok(response.data),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let response = self
                        .client
                        .req_get(
                            GetUsersRequest::ids(user_id_refs.as_slice()),
                            &refreshed_token,
                        )
                        .await?;
                    Ok(response.data)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// 複数のログイン名からユーザー情報を取得
    pub async fn get_users_by_logins(
        &self,
        logins: &[&str],
    ) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        let login_refs: Vec<&types::UserNameRef> =
            logins.iter().map(|login| (*login).into()).collect();
        let request = GetUsersRequest::logins(login_refs.as_slice());

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        match self.client.req_get(request, &token).await {
            Ok(response) => Ok(response.data),
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let response = self
                        .client
                        .req_get(
                            GetUsersRequest::logins(login_refs.as_slice()),
                            &refreshed_token,
                        )
                        .await?;
                    Ok(response.data)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// 複数のユーザーIDからフォロワー数をバッチ取得
    ///
    /// user_ids: ユーザーIDのリスト
    /// 戻り値: (user_id, follower_count) のタプルのベクター
    pub async fn get_followers_batch(
        &self,
        user_ids: &[&str],
    ) -> Result<Vec<(String, i32)>, Box<dyn std::error::Error>> {
        use twitch_api::helix::channels::GetChannelFollowersRequest;

        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let token = self.get_user_token().await?;
        let mut results = Vec::new();

        // 各ユーザーIDに対してフォロワー数を取得
        for user_id in user_ids {
            let broadcaster_id_ref: &types::UserIdRef = (*user_id).into();
            let request = GetChannelFollowersRequest::broadcaster_id(broadcaster_id_ref);

            // リクエストをトラッキング
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.track_request();
            }

            match self.client.req_get(request, &token).await {
                Ok(response) => {
                    let follower_count = response.total.unwrap_or(0) as i32;
                    results.push((user_id.to_string(), follower_count));
                }
                Err(e) => {
                    // エラーの場合は0として扱う（個別のエラーで全体を失敗させない）
                    eprintln!(
                        "[TwitchAPI] Failed to get follower count for {}: {}",
                        user_id, e
                    );
                    results.push((user_id.to_string(), 0));
                }
            }
        }

        Ok(results)
    }

    /// 上位配信を取得（自動発見機能用）
    ///
    /// game_ids: ゲームIDのリスト（Noneの場合は全ゲーム）
    /// languages: 言語コードのリスト（Noneの場合は全言語）
    /// max_results: 最大取得件数（デフォルト100、最大100）
    pub async fn get_top_streams(
        &self,
        game_ids: Option<Vec<String>>,
        languages: Option<Vec<String>>,
        max_results: Option<usize>,
    ) -> Result<Vec<Stream>, Box<dyn std::error::Error>> {
        let token = self.get_user_token().await?;

        // GetStreamsRequestを構築
        let mut request = GetStreamsRequest::default();

        // ゲームIDを設定
        if let Some(ids) = game_ids {
            let game_id_refs: Vec<types::CategoryId> =
                ids.into_iter().map(|id| id.into()).collect();
            request.game_id = game_id_refs.into();
        }

        // 言語を設定（カンマ区切りで結合）
        if let Some(langs) = languages {
            if !langs.is_empty() {
                request.language = Some(langs.join(",").into());
            }
        }

        // 最初の100件を取得（max_resultsは後でフィルタリング）
        request.first = Some(100);

        // リクエストをトラッキング
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            limiter.track_request();
        }

        let response = match self.client.req_get(request, &token).await {
            Ok(response) => response,
            Err(e) => {
                // 401エラーの場合、トークンをリフレッシュして再試行
                if e.to_string().contains(twitch::ERROR_UNAUTHORIZED)
                    || e.to_string().contains(twitch::ERROR_UNAUTHORIZED_TEXT)
                {
                    eprintln!("Token expired, attempting refresh...");
                    let _new_token = self.refresh_token().await?;
                    let refreshed_token = self.get_user_token().await?;

                    // 再試行もトラッキング
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.track_request();
                    }

                    let mut retry_request = GetStreamsRequest::default();
                    retry_request.first = Some(100);
                    self.client.req_get(retry_request, &refreshed_token).await?
                } else {
                    return Err(e.into());
                }
            }
        };

        let mut streams = response.data;

        // max_resultsが指定されている場合は制限
        if let Some(max) = max_results {
            streams.truncate(max);
        }

        Ok(streams)
    }
}

/// Twitch APIレート制限トラッカー
///
/// トークンバケットアルゴリズムをシミュレートし、直近1分間のリクエスト数を追跡します。
pub struct TwitchRateLimitTracker {
    /// リクエストごとのタイムスタンプとポイント消費を記録
    request_log: VecDeque<(Instant, u32)>,
    /// バケット容量（Twitch API レート制限）
    bucket_capacity: u32,
    /// ウィンドウサイズ
    window_duration: Duration,
}

impl TwitchRateLimitTracker {
    /// デフォルトの設定で新しいトラッカーを作成
    ///
    /// バケット容量: Twitch Developer Forumsの情報に基づく
    pub fn new() -> Self {
        Self {
            request_log: VecDeque::new(),
            bucket_capacity: twitch::RATE_LIMIT_BUCKET_CAPACITY as u32,
            window_duration: Duration::from_secs(twitch::RATE_LIMIT_WINDOW_SECS),
        }
    }

    /// カスタム設定で新しいトラッカーを作成
    #[allow(dead_code)]
    pub fn with_capacity(bucket_capacity: u32) -> Self {
        Self {
            request_log: VecDeque::new(),
            bucket_capacity,
            window_duration: Duration::from_secs(60),
        }
    }

    /// リクエストを記録（デフォルト1ポイント）
    pub fn track_request(&mut self) {
        self.track_request_with_points(1);
    }

    /// ポイント指定でリクエストを記録（将来の拡張用）
    pub fn track_request_with_points(&mut self, points: u32) {
        let now = Instant::now();
        self.request_log.push_back((now, points));
        self.cleanup_old_entries();
    }

    /// 現在のステータスを取得
    pub fn get_status(&self) -> TwitchRateLimitStatus {
        let now = Instant::now();

        // 期限切れのエントリを除外してカウント
        let valid_entries: Vec<_> = self
            .request_log
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) < self.window_duration)
            .collect();

        let points_used: u32 = valid_entries.iter().map(|(_, points)| points).sum();
        let request_count = valid_entries.len() as u32;
        let points_remaining = self.bucket_capacity.saturating_sub(points_used);
        let usage_percent = (points_used as f32 / self.bucket_capacity as f32) * 100.0;

        // 最古のエントリが期限切れになるまでの秒数を計算
        let oldest_entry_expires_in_seconds = valid_entries.first().map(|(timestamp, _)| {
            let elapsed = now.duration_since(*timestamp);
            let remaining = self.window_duration.saturating_sub(elapsed);
            remaining.as_secs() as u32
        });

        TwitchRateLimitStatus {
            points_used,
            bucket_capacity: self.bucket_capacity,
            points_remaining,
            oldest_entry_expires_in_seconds,
            usage_percent,
            request_count,
        }
    }

    /// 期限切れのエントリを削除（スライディングウィンドウ）
    fn cleanup_old_entries(&mut self) {
        let now = Instant::now();

        // 60秒以上前のエントリを削除
        while let Some((timestamp, _)) = self.request_log.front() {
            if now.duration_since(*timestamp) >= self.window_duration {
                self.request_log.pop_front();
            } else {
                break;
            }
        }
    }
}

impl Default for TwitchRateLimitTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Twitch APIレート制限のステータス情報
#[derive(Debug, Clone, Serialize)]
pub struct TwitchRateLimitStatus {
    /// 直近1分間で消費したポイント数（≒リクエスト数）
    pub points_used: u32,
    /// バケット容量（Twitch API レート制限）
    pub bucket_capacity: u32,
    /// 推定残りポイント数
    pub points_remaining: u32,
    /// 最古エントリが期限切れになるまでの秒数（バケット部分回復）
    pub oldest_entry_expires_in_seconds: Option<u32>,
    /// 使用率（0.0 - 100.0）
    pub usage_percent: f32,
    /// 直近1分間のリクエスト数
    pub request_count: u32,
}

// 既存コードとの互換性のため残す
#[allow(dead_code)]
pub struct RateLimiter {
    // 将来的に実装
}
