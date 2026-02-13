/// マルチビュー向けコマンド
///
/// 複数チャンネルのリアルタイム統計・イベント検知APIを提供
use crate::database::repositories::multiview_repository::{
    MultiviewChannelStats, MultiviewRepository,
};
use crate::database::DatabaseManager;
use crate::error::ResultExt;
use tauri::State;

#[tauri::command]
pub async fn get_multiview_realtime_stats(
    db_manager: State<'_, DatabaseManager>,
    channel_ids: Vec<i64>,
) -> Result<Vec<MultiviewChannelStats>, String> {
    db_manager
        .with_connection(|conn| {
            MultiviewRepository::get_realtime_stats(conn, &channel_ids)
                .db_context("get multiview realtime stats")
                .map_err(|e| e.to_string())
        })
        .await
}
