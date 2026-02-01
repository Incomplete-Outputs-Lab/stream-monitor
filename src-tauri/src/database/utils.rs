use crate::database::models::ChatMessage;
use duckdb::{Connection, Result as DuckResult, Row};

/// 動的パラメータ数に対応したマクロ
/// DuckDBのパラメータを処理し、適切な数のパラメータで実行する
macro_rules! dispatch_params {
    ($method:expr, $params:expr) => {{
        let params_slice = $params;
        match params_slice.len() {
            0 => $method([]),
            1 => $method([params_slice[0].as_str()]),
            2 => $method([params_slice[0].as_str(), params_slice[1].as_str()]),
            3 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
            ]),
            4 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
                params_slice[3].as_str(),
            ]),
            5 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
                params_slice[3].as_str(),
                params_slice[4].as_str(),
            ]),
            6 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
                params_slice[3].as_str(),
                params_slice[4].as_str(),
                params_slice[5].as_str(),
            ]),
            7 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
                params_slice[3].as_str(),
                params_slice[4].as_str(),
                params_slice[5].as_str(),
                params_slice[6].as_str(),
            ]),
            8 => $method([
                params_slice[0].as_str(),
                params_slice[1].as_str(),
                params_slice[2].as_str(),
                params_slice[3].as_str(),
                params_slice[4].as_str(),
                params_slice[5].as_str(),
                params_slice[6].as_str(),
                params_slice[7].as_str(),
            ]),
            _ => Err(duckdb::Error::InvalidParameterName(
                "Too many parameters (max 8 supported)".to_string(),
            )),
        }
    }};
}

/// DuckDBの動的パラメータを処理するヘルパー関数
/// パラメータが0-8個の場合にのみサポート
pub fn execute_with_params(conn: &Connection, sql: &str, params: &[String]) -> DuckResult<usize> {
    dispatch_params!(|p| conn.execute(sql, p), params)
}

pub fn query_map_with_params<'stmt, T, F>(
    stmt: &'stmt mut duckdb::Statement,
    params: &[String],
    f: F,
) -> DuckResult<duckdb::MappedRows<'stmt, F>>
where
    F: FnMut(&duckdb::Row) -> DuckResult<T>,
{
    dispatch_params!(|p| stmt.query_map(p, f), params)
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
    dispatch_params!(|p| conn.query_row(sql, p, f), params)
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
