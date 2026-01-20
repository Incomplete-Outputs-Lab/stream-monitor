use crate::database::models::ChatMessage;
use duckdb::{Connection, Result as DuckResult, Row};

/// DuckDBの動的パラメータを処理するヘルパー関数
/// パラメータが0-15個の場合にのみサポート
pub fn execute_with_params(conn: &Connection, sql: &str, params: &[String]) -> DuckResult<usize> {
    match params.len() {
        0 => conn.execute(sql, []),
        1 => conn.execute(sql, [params[0].as_str()]),
        2 => conn.execute(sql, [params[0].as_str(), params[1].as_str()]),
        3 => conn.execute(
            sql,
            [params[0].as_str(), params[1].as_str(), params[2].as_str()],
        ),
        4 => conn.execute(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
            ],
        ),
        5 => conn.execute(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
                params[4].as_str(),
            ],
        ),
        _ => Err(duckdb::Error::InvalidParameterName(
            "Too many parameters (max 5 supported)".to_string(),
        )),
    }
}

pub fn query_map_with_params<'stmt, T, F>(
    stmt: &'stmt mut duckdb::Statement,
    params: &[String],
    f: F,
) -> DuckResult<duckdb::MappedRows<'stmt, F>>
where
    F: FnMut(&duckdb::Row) -> DuckResult<T>,
{
    match params.len() {
        0 => stmt.query_map([], f),
        1 => stmt.query_map([params[0].as_str()], f),
        2 => stmt.query_map([params[0].as_str(), params[1].as_str()], f),
        3 => stmt.query_map(
            [params[0].as_str(), params[1].as_str(), params[2].as_str()],
            f,
        ),
        4 => stmt.query_map(
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
            ],
            f,
        ),
        5 => stmt.query_map(
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
                params[4].as_str(),
            ],
            f,
        ),
        _ => Err(duckdb::Error::InvalidParameterName(
            "Too many parameters (max 5 supported)".to_string(),
        )),
    }
}

pub fn query_row_with_params<T, F>(
    conn: &Connection,
    sql: &str,
    params: &[String],
    f: F,
) -> DuckResult<T>
where
    F: FnOnce(&duckdb::Row) -> DuckResult<T>,
{
    match params.len() {
        0 => conn.query_row(sql, [], f),
        1 => conn.query_row(sql, [params[0].as_str()], f),
        2 => conn.query_row(sql, [params[0].as_str(), params[1].as_str()], f),
        3 => conn.query_row(
            sql,
            [params[0].as_str(), params[1].as_str(), params[2].as_str()],
            f,
        ),
        4 => conn.query_row(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
            ],
            f,
        ),
        5 => conn.query_row(
            sql,
            [
                params[0].as_str(),
                params[1].as_str(),
                params[2].as_str(),
                params[3].as_str(),
                params[4].as_str(),
            ],
            f,
        ),
        _ => Err(duckdb::Error::InvalidParameterName(
            "Too many parameters (max 5 supported)".to_string(),
        )),
    }
}

/// RowからChatMessageを作成するヘルパー関数
pub fn row_to_chat_message(row: &Row) -> DuckResult<ChatMessage> {
    Ok(ChatMessage {
        id: Some(row.get(0)?),
        stream_id: row.get(1)?,
        timestamp: row.get(2)?,
        platform: row.get(3)?,
        user_id: row.get::<_, Option<String>>(4)?,
        user_name: row.get(5)?,
        message: row.get(6)?,
        message_type: row.get(7)?,
    })
}

/// チャットメッセージのクエリ結果をChatMessageベクターに変換するヘルパー関数
pub fn query_chat_messages(
    conn: &Connection,
    sql: &str,
    params: &[String],
) -> DuckResult<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = query_map_with_params(&mut stmt, params, row_to_chat_message)?;
    rows.collect()
}
