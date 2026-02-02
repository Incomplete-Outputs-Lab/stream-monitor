use crate::collectors::poller::ChannelPoller;
use crate::websocket::twitch_irc::IrcConnectionStatus;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// IRC接続状態を取得
#[tauri::command]
pub async fn get_irc_connection_status(
    poller: State<'_, Arc<Mutex<ChannelPoller>>>,
) -> Result<Vec<IrcConnectionStatus>, String> {
    let poller = poller.lock().await;
    
    if let Some(twitch_collector) = poller.get_twitch_collector() {
        Ok(twitch_collector.get_irc_connection_status().await)
    } else {
        Ok(Vec::new())
    }
}

/// IRC接続を再接続
#[tauri::command]
pub async fn reconnect_irc_channel(
    channel_id: i64,
    channel_name: String,
    poller: State<'_, Arc<Mutex<ChannelPoller>>>,
) -> Result<(), String> {
    let poller = poller.lock().await;
    
    if let Some(twitch_collector) = poller.get_twitch_collector() {
        // 一度停止
        let _ = twitch_collector.stop_chat_collection(channel_id).await;
        
        // 再接続
        twitch_collector
            .start_chat_collection(channel_id, &channel_name)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Twitch collector not initialized".to_string())
    }
}
