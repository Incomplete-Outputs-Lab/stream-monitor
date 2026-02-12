use crate::collectors::poller::ChannelPoller;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn is_backend_ready(app_handle: AppHandle) -> Result<bool, String> {
    // ChannelPollerが存在し、初期化されているかチェック
    if let Some(poller) = app_handle.try_state::<Arc<Mutex<ChannelPoller>>>() {
        // ロックを取得できるかチェック（初期化中でないか）
        match tokio::time::timeout(std::time::Duration::from_millis(100), poller.lock()).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // まだ初期化中
        }
    } else {
        Ok(false) // ChannelPollerがまだ登録されていない
    }
}
