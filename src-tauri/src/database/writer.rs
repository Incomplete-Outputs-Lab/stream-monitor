use crate::database::models::{ChatMessage, Stream, StreamStats};
use duckdb::Connection;

pub struct DatabaseWriter;

impl DatabaseWriter {
    pub fn insert_or_update_stream(
        conn: &Connection,
        channel_id: i64,
        stream: &Stream,
    ) -> Result<i64, duckdb::Error> {
        // 既存のストリームを確認
        let mut stmt =
            conn.prepare("SELECT id FROM streams WHERE channel_id = ? AND stream_id = ?")?;

        let channel_id_str = channel_id.to_string();
        let stream_id_str = stream.stream_id.clone();

        let existing_id: Option<i64> = stmt
            .query_map([&channel_id_str as &str, &stream_id_str as &str], |row| {
                row.get(0)
            })?
            .next()
            .transpose()?;

        if let Some(id) = existing_id {
            // 更新
            let ended_at_value = stream.ended_at.as_deref();

            conn.execute(
                "UPDATE streams SET title = ?, category = ?, ended_at = ? WHERE id = ?",
                duckdb::params![
                    stream.title.as_deref().unwrap_or(""),
                    stream.category.as_deref().unwrap_or(""),
                    ended_at_value,
                    id,
                ],
            )?;
            Ok(id)
        } else {
            // 挿入 - DuckDBではRETURNING句を使用してINSERTと同時にIDを取得
            // ended_atはOptionなので、Noneの場合はNULLとして扱う
            let ended_at_value = stream.ended_at.as_deref();

            let id: i64 = conn.query_row(
                "INSERT INTO streams (channel_id, stream_id, title, category, started_at, ended_at) 
                 VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
                duckdb::params![
                    channel_id,
                    &stream.stream_id,
                    stream.title.as_deref().unwrap_or(""),
                    stream.category.as_deref().unwrap_or(""),
                    &stream.started_at,
                    ended_at_value,
                ],
                |row| row.get(0)
            )?;
            Ok(id)
        }
    }

    pub fn insert_stream_stats(
        conn: &Connection,
        stats: &StreamStats,
    ) -> Result<(), duckdb::Error> {
        conn.execute(
            "INSERT INTO stream_stats (stream_id, collected_at, viewer_count, chat_rate_1min, category, title, follower_count, twitch_user_id, channel_name)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &stats.stream_id.to_string(),
                &stats.collected_at,
                &stats
                    .viewer_count
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                &stats.chat_rate_1min.to_string(),
                stats.category.as_deref().unwrap_or(""),
                stats.title.as_deref().unwrap_or(""),
                &stats
                    .follower_count
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                stats.twitch_user_id.as_deref().unwrap_or(""),
                stats.channel_name.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(())
    }

    pub fn insert_chat_messages_batch(
        conn: &Connection,
        messages: &[ChatMessage],
    ) -> Result<(), duckdb::Error> {
        if messages.is_empty() {
            return Ok(());
        }

        // バッチインサート用のトランザクション開始
        conn.execute("BEGIN TRANSACTION", [])?;

        // エラー時のROLLBACK処理を含むスコープ
        let result = (|| {
            // プリペアドステートメントを使用した効率的なバッチインサート
            let mut stmt = conn.prepare(
                "INSERT INTO chat_messages (stream_id, timestamp, platform, user_id, user_name, message, message_type)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )?;

            for message in messages {
                stmt.execute([
                    &message.stream_id.to_string(),
                    &message.timestamp,
                    &message.platform,
                    &message.user_id.as_deref().unwrap_or("").to_string(),
                    &message.user_name,
                    &message.message,
                    &message.message_type,
                ])?;
            }

            Ok::<(), duckdb::Error>(())
        })();

        // エラーハンドリング: エラーの場合はROLLBACK、成功の場合はCOMMIT
        match result {
            Ok(_) => {
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                // ROLLBACKを試行（ROLLBACKが失敗しても元のエラーを返す）
                if let Err(rollback_err) = conn.execute("ROLLBACK", []) {
                    eprintln!("Failed to rollback transaction: {}", rollback_err);
                }
                Err(e)
            }
        }
    }
}
