use crate::database::models::{ChatMessage, Stream, StreamStats};
use duckdb::Connection;

pub struct DatabaseWriter;

impl DatabaseWriter {
    pub fn insert_or_update_stream(
        conn: &Connection,
        channel_id: i64,
        stream: &Stream,
    ) -> Result<i64, duckdb::Error> {
        // パフォーマンス最適化: UPSERT構文を使用して単一クエリで処理
        // streamsテーブルのUNIQUE制約 (channel_id, stream_id) を利用
        let ended_at_value = stream.ended_at.as_deref();

        let id: i64 = conn.query_row(
            r#"
            INSERT INTO streams (channel_id, stream_id, title, category, started_at, ended_at) 
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT (channel_id, stream_id) 
            DO UPDATE SET 
                title = EXCLUDED.title,
                category = EXCLUDED.category,
                ended_at = EXCLUDED.ended_at
            RETURNING id
            "#,
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
            // パフォーマンス最適化: マルチロウINSERTを使用
            // DuckDBでは大量の行を一度にINSERTする方が効率的
            
            // VALUES句を構築
            let values_placeholders: Vec<String> = messages
                .iter()
                .map(|msg| {
                    // badges を DuckDB 配列リテラル形式に変換
                    let badges_literal = match &msg.badges {
                        None => "NULL".to_string(),
                        Some(badges) if badges.is_empty() => "NULL".to_string(),
                        Some(badges) => {
                            let escaped_badges: Vec<String> = badges
                                .iter()
                                .map(|b| format!("'{}'", b.replace("'", "''")))
                                .collect();
                            format!("ARRAY[{}]", escaped_badges.join(", "))
                        }
                    };
                    format!("(?, ?, ?, ?, ?, ?, ?, ?, {}, ?)", badges_literal)
                })
                .collect();

            let sql = format!(
                "INSERT INTO chat_messages (channel_id, stream_id, timestamp, platform, user_id, user_name, message, message_type, badges, badge_info) VALUES {}",
                values_placeholders.join(", ")
            );

            // すべてのパラメータを順番に配列に格納
            let mut params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();
            for message in messages {
                params.push(Box::new(message.channel_id));
                params.push(Box::new(message.stream_id));
                params.push(Box::new(message.timestamp.clone()));
                params.push(Box::new(message.platform.clone()));
                params.push(Box::new(message.user_id.clone()));
                params.push(Box::new(message.user_name.clone()));
                params.push(Box::new(message.message.clone()));
                params.push(Box::new(message.message_type.clone()));
                // badges はリテラルで埋め込み済みのためスキップ
                params.push(Box::new(message.badge_info.clone()));
            }

            // パラメータ参照を作成
            let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&sql, param_refs.as_slice())?;

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
