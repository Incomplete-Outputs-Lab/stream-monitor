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
        let stream_id_str = stream.id.unwrap_or(0).to_string();

        let existing_id: Option<i64> = stmt
            .query_map([&channel_id_str as &str, &stream_id_str as &str], |row| {
                row.get(0)
            })?
            .next()
            .transpose()?;

        if let Some(id) = existing_id {
            // 更新
            conn.execute(
                "UPDATE streams SET title = ?, category = ?, ended_at = ? WHERE id = ?",
                [
                    &stream.title.clone().unwrap_or_default(),
                    &stream.category.clone().unwrap_or_default(),
                    &stream.ended_at.clone().unwrap_or_default(),
                    &id.to_string(),
                ],
            )?;
            Ok(id)
        } else {
            // 挿入
            conn.execute(
                "INSERT INTO streams (channel_id, stream_id, title, category, started_at, ended_at) 
                 VALUES (?, ?, ?, ?, ?, ?)",
                [
                    &channel_id.to_string(),
                    &stream.stream_id,
                    &stream.title.clone().unwrap_or_default(),
                    &stream.category.clone().unwrap_or_default(),
                    &stream.started_at,
                    &stream.ended_at.clone().unwrap_or_default(),
                ],
            )?;
            // DuckDBでは、last_insert_rowid()を直接取得できないため、SELECTを使用
            // DuckDBでは、last_insert_rowid()を直接取得できないため、SELECTを使用
            let id: i64 = conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?;
            Ok(id)
        }
    }

    pub fn insert_stream_stats(
        conn: &Connection,
        stats: &StreamStats,
    ) -> Result<(), duckdb::Error> {
        conn.execute(
            "INSERT INTO stream_stats (stream_id, collected_at, viewer_count, chat_rate_1min)
             VALUES (?, ?, ?, ?)",
            [
                &stats.stream_id.to_string(),
                &stats.collected_at,
                &stats
                    .viewer_count
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                &stats.chat_rate_1min.to_string(),
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn insert_chat_message(
        conn: &Connection,
        message: &ChatMessage,
    ) -> Result<(), duckdb::Error> {
        conn.execute(
            "INSERT INTO chat_messages (stream_id, timestamp, platform, user_id, user_name, message, message_type)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            [
                &message.stream_id.to_string(),
                &message.timestamp,
                &message.platform,
                message.user_id.as_deref().unwrap_or(""),
                &message.user_name,
                &message.message,
                &message.message_type,
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

        conn.execute("COMMIT", [])?;
        Ok(())
    }
}
