use axum::{
    extract::Query,
    response::{Html, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub error: Option<String>,
    pub state: Option<String>,
}

pub struct OAuthServer {
    port: u16,
    callback_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<OAuthCallback>>>>,
}

impl OAuthServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            callback_tx: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub async fn start_and_wait_for_callback(
        &self,
    ) -> Result<OAuthCallback, Box<dyn std::error::Error>> {
        let (tx, rx) = oneshot::channel();

        {
            let mut callback_tx = self.callback_tx.lock().await;
            *callback_tx = Some(tx);
        }

        let callback_tx_clone = self.callback_tx.clone();
        let port = self.port;

        // 成功ページHTML
        let success_html = r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>認証成功 - Stream Stats Collector</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex; justify-content: center; align-items: center;
            min-height: 100vh;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center; padding: 2rem;
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px); border-radius: 20px;
            box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
            max-width: 400px;
        }
        .icon { font-size: 4rem; margin-bottom: 1rem; }
        h1 { font-size: 1.5rem; margin-bottom: 0.5rem; }
        p { opacity: 0.9; line-height: 1.6; }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">✅</div>
        <h1>認証成功</h1>
        <p>このウィンドウを閉じて、アプリケーションに戻ってください。</p>
    </div>
</body>
</html>"#;

        let error_html = r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>認証エラー - Stream Stats Collector</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex; justify-content: center; align-items: center;
            min-height: 100vh;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: white;
        }
        .container {
            text-align: center; padding: 2rem;
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px); border-radius: 20px;
            box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
            max-width: 400px;
        }
        .icon { font-size: 4rem; margin-bottom: 1rem; }
        h1 { font-size: 1.5rem; margin-bottom: 0.5rem; }
        p { opacity: 0.9; line-height: 1.6; }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">❌</div>
        <h1>認証エラー</h1>
        <p>認証中にエラーが発生しました。アプリケーションに戻って再度お試しください。</p>
    </div>
</body>
</html>"#;

        let app = Router::new()
            .route(
                "/callback",
                get(move |query: Query<OAuthCallback>| async move {
                    let callback = query.0;
                    let mut callback_tx = callback_tx_clone.lock().await;

                    if let Some(tx) = callback_tx.take() {
                        let _ = tx.send(callback.clone());
                    }

                    if callback.error.is_some() {
                        Html(error_html)
                    } else if callback.code.is_some() {
                        Html(success_html)
                    } else {
                        Html(error_html)
                    }
                }),
            )
            .route("/", get(|| async { Redirect::to("/callback") }));

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        // サーバーをバックグラウンドで起動
        let server_handle = tokio::spawn(async move { axum::serve(listener, app).await });

        // コールバックを待つ
        match rx.await {
            Ok(cb) => {
                // コールバックを受信したらサーバーを停止
                server_handle.abort();
                Ok(cb)
            }
            Err(e) => {
                server_handle.abort();
                Err(format!("Failed to receive callback: {}", e).into())
            }
        }
    }

    #[allow(dead_code)]
    pub fn callback_url(&self) -> String {
        format!("http://localhost:{}/callback", self.port)
    }
}
