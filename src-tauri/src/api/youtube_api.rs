// Keyring is not used in this file as it doesn't have AppHandle access
use crate::constants::youtube;
use google_youtube3::{hyper_rustls, hyper_util, yup_oauth2, YouTube};
use hyper_util::client::legacy::connect::HttpConnector;
use std::sync::Arc;

#[allow(dead_code)]
pub struct YouTubeApiClient {
    hub: Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>>,
    access_token: Option<String>,
}

#[allow(dead_code)]
impl YouTubeApiClient {
    pub async fn new(
        client_id: String,
        client_secret: String,
        _redirect_uri: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // google_youtube3のyup_oauth2を使用
        let secret = yup_oauth2::ApplicationSecret {
            client_id,
            client_secret,
            auth_uri: youtube::OAUTH_AUTH_URL.to_string(),
            token_uri: youtube::OAUTH_TOKEN_URL.to_string(),
            ..Default::default()
        };

        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
            secret.clone(),
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await?;

        let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("Failed to load native roots")
            .https_or_http()
            .enable_http1()
            .build();

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(https_connector);

        let hub = Arc::new(YouTube::new(client, auth));

        // Note: Token retrieval requires AppHandle which this struct doesn't have
        let access_token = None;

        Ok(Self { hub, access_token })
    }

    // アクセストークンの取得は不要（hubに組み込まれている）
    // async fn get_access_token(&mut self) -> Result<AccessToken, Box<dyn std::error::Error>> {
    //     ...
    // }

    pub async fn get_channel_by_username(
        &mut self,
        username: &str,
    ) -> Result<Option<google_youtube3::api::Channel>, Box<dyn std::error::Error>> {
        // 認証はhubに組み込まれているため、直接呼び出す
        let part = vec![
            youtube::PART_ID.to_string(),
            youtube::PART_SNIPPET.to_string(),
            youtube::PART_CONTENT_DETAILS.to_string(),
        ];
        let (_, response) = self
            .hub
            .channels()
            .list(&part)
            .for_username(username)
            .doit()
            .await?;

        Ok(response.items.and_then(|items| items.into_iter().next()))
    }

    pub async fn get_live_stream(
        &mut self,
        channel_id: &str,
    ) -> Result<Option<google_youtube3::api::Video>, Box<dyn std::error::Error>> {
        // チャンネルのライブストリームを検索
        let part = vec![
            youtube::PART_ID.to_string(),
            youtube::PART_SNIPPET.to_string(),
            "liveStreamingDetails".to_string(),
            "statistics".to_string(),
        ];

        let (_, response) = self
            .hub
            .search()
            .list(&part)
            .channel_id(channel_id)
            .event_type(youtube::EVENT_TYPE_LIVE)
            .add_type(youtube::TYPE_VIDEO)
            .max_results(youtube::MAX_RESULTS_DEFAULT)
            .doit()
            .await?;

        if let Some(items) = response.items {
            if let Some(search_result) = items.into_iter().next() {
                if let Some(video_id) = search_result.id.and_then(|id| id.video_id) {
                    // 動画の詳細を取得
                    let (_, video_response) = self
                        .hub
                        .videos()
                        .list(&part)
                        .add_id(&video_id)
                        .doit()
                        .await?;

                    return Ok(video_response
                        .items
                        .and_then(|items| items.into_iter().next()));
                }
            }
        }

        Ok(None)
    }

    pub async fn get_channel_by_id(
        &mut self,
        channel_id: &str,
    ) -> Result<Option<google_youtube3::api::Channel>, Box<dyn std::error::Error>> {
        let part = vec![
            "id".to_string(),
            "snippet".to_string(),
            "statistics".to_string(),
        ];
        let (_, response) = self
            .hub
            .channels()
            .list(&part)
            .add_id(channel_id)
            .doit()
            .await?;

        Ok(response.items.and_then(|items| items.into_iter().next()))
    }

    pub fn get_hub(&self) -> Arc<YouTube<hyper_rustls::HttpsConnector<HttpConnector>>> {
        Arc::clone(&self.hub)
    }

    pub fn set_access_token(&mut self, token: String) {
        // Note: Token saving requires AppHandle which this struct doesn't have
        // The token will be saved through other means (e.g., via commands)
        self.access_token = Some(token);
    }

    pub async fn authenticate_with_token(
        &mut self,
        token: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_access_token(token);
        Ok(())
    }
}
