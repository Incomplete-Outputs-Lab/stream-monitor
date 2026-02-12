use crate::database::DatabaseManager;
use crate::error::ResultExt;
use chrono::{Local, TimeZone};
use duckdb::{params, types::TimeUnit, types::ValueRef, Connection, Row};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

/// SQLクエリが有効かどうかを基本的にチェック
fn validate_sql_query(query: &str) -> Result<(), String> {
    let query_trimmed = query.trim();

    if query_trimmed.is_empty() {
        return Err("クエリが空です".to_string());
    }

    // コメントをスキップして最初のキーワードを取得
    let first_keyword = query_trimmed
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("--"))
        .flat_map(|line| line.split_whitespace())
        .find(|word| !word.is_empty())
        .unwrap_or("")
        .to_uppercase();

    // 有効なSQLキーワードのリスト
    let valid_keywords = [
        "SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "TRUNCATE", "PRAGMA",
        "SHOW", "DESCRIBE", "DESC", "EXPLAIN", "WITH", "COPY", "IMPORT", "EXPORT", "ATTACH",
        "DETACH", "BEGIN", "COMMIT", "ROLLBACK", "SET", "CALL", "LOAD",
    ];

    if !valid_keywords.contains(&first_keyword.as_str()) {
        return Err(format!(
            "無効なSQLクエリです: '{}' は認識されないSQLキーワードです。\n有効なキーワード: SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, PRAGMA, SHOW, DESCRIBE, WITH など",
            first_keyword
        ));
    }

    Ok(())
}

/// クエリを前処理してLIST型カラムを文字列に変換
fn preprocess_query_for_list_columns(conn: &Connection, query: &str) -> Result<String, String> {
    // SELECT * FROM table のようなクエリの場合、LIST型カラムを検出して自動変換
    // より複雑なクエリの場合は元のクエリをそのまま使用

    // クエリを正規化
    let query_normalized = query.trim();

    // 複数のステートメントがある場合は前処理をスキップ
    let semicolon_count = query_normalized.matches(';').count();
    if semicolon_count > 1 || (semicolon_count == 1 && !query_normalized.trim_end().ends_with(';'))
    {
        eprintln!("[SQL] Multiple statements detected, skipping LIST preprocessing");
        return Ok(query.to_string());
    }

    // セミコロンを削除して単一のステートメントとして処理
    let single_statement = query_normalized.trim_end_matches(';').trim();

    // シンプルなヒューリスティック: "SELECT * FROM" パターンを検出
    let trimmed_upper = single_statement.to_uppercase();
    if !trimmed_upper.starts_with("SELECT * FROM") {
        // 複雑なクエリはそのまま返す
        return Ok(query.to_string());
    }

    // テーブル名を抽出（簡易版）
    let parts: Vec<&str> = single_statement.split_whitespace().collect();
    let table_name = if let Some(idx) = parts.iter().position(|&p| p.to_uppercase() == "FROM") {
        if idx + 1 < parts.len() {
            // テーブル名から記号を削除
            parts[idx + 1]
                .trim_end_matches(';')
                .trim_end_matches(',')
                .trim()
        } else {
            return Ok(query.to_string());
        }
    } else {
        return Ok(query.to_string());
    };

    // テーブルのスキーマを取得してLIST型カラムを検出
    let schema_query = format!(
        "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = '{}'",
        table_name
    );

    let mut stmt = match conn.prepare(&schema_query) {
        Ok(s) => s,
        Err(_) => return Ok(query.to_string()), // エラー時は元のクエリを返す
    };

    let mut list_columns = Vec::new();
    let rows_result = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    });

    if let Ok(rows) = rows_result {
        for row in rows.flatten() {
            let (col_name, data_type) = row;
            if data_type.contains("[]") || data_type.to_uppercase().contains("LIST") {
                list_columns.push(col_name);
            }
        }
    }

    // LIST型カラムがない場合は元のクエリを返す
    if list_columns.is_empty() {
        return Ok(query.to_string());
    }

    // SELECT * をカラム名に展開し、LIST型カラムにlist_to_string()を適用
    let all_columns_query = format!("SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position", table_name);
    let mut stmt = match conn.prepare(&all_columns_query) {
        Ok(s) => s,
        Err(_) => return Ok(query.to_string()),
    };

    let mut columns = Vec::new();
    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
        for row in rows.flatten() {
            if list_columns.contains(&row) {
                // LIST型カラムは CAST で VARCHAR に変換
                // DuckDBではLIST型をVARCHARにCASTすると ['elem1', 'elem2'] 形式の文字列になる
                columns.push(format!("CAST({} AS VARCHAR) as {}", row, row));
            } else {
                columns.push(row);
            }
        }
    }

    if columns.is_empty() {
        return Ok(query.to_string());
    }

    // クエリを再構築
    let select_clause = columns.join(", ");

    // FROM以降の部分を安全に取得（single_statementから）
    let from_pos = match single_statement.to_uppercase().find("FROM") {
        Some(pos) => pos,
        None => return Ok(query.to_string()),
    };

    let rest_of_query = &single_statement[from_pos..];
    let new_query = format!("SELECT {} {}", select_clause, rest_of_query);

    eprintln!(
        "[SQL] Transformed query to handle LIST columns: {}",
        new_query
    );
    Ok(new_query)
}

/// DuckDBのLIST型をJSON配列に変換
fn extract_list_from_row(row: &Row, col_index: usize) -> serde_json::Value {
    // ValueRefとして取得
    match row.get_ref(col_index) {
        Ok(ValueRef::Null) => serde_json::Value::Null,
        Ok(ValueRef::List(_, _)) => {
            // LIST型の場合、文字列化を試みる（自動変換が適用されていない場合）
            serde_json::Value::String("[LIST型カラム]".to_string())
        }
        Ok(ValueRef::Text(s)) => {
            // すでに文字列化されている場合（CAST AS VARCHAR適用済み）
            let text = String::from_utf8_lossy(s).to_string();
            // DuckDBの配列文字列形式 ['elem1', 'elem2'] をパース
            parse_duckdb_list_to_json(&text)
        }
        Ok(_) => {
            // その他の型は文字列として取得を試みる
            match row.get::<_, String>(col_index) {
                Ok(s) => {
                    // DuckDBの配列文字列形式をパース
                    parse_duckdb_list_to_json(&s)
                }
                Err(_) => serde_json::Value::String("<型変換エラー>".to_string()),
            }
        }
        Err(_) => serde_json::Value::String("<取得エラー>".to_string()),
    }
}

/// DuckDBのList文字列をJSON配列に変換
/// 例: ['elem1', 'elem2'] または ["elem1", "elem2"] → ["elem1", "elem2"]
fn parse_duckdb_list_to_json(s: &str) -> serde_json::Value {
    let trimmed = s.trim();

    // 空配列チェック
    if trimmed == "[]" || trimmed.is_empty() {
        return serde_json::Value::Array(vec![]);
    }

    // DuckDBのList形式を解析: ['elem1', 'elem2']
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];

        // 要素をパース
        let mut elements = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let mut escape_next = false;

        for ch in inner.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                }
                '\'' | '"' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                c if c == quote_char && in_quotes => {
                    in_quotes = false;
                    // クォートを閉じた後、要素を追加
                }
                ',' if !in_quotes => {
                    // 要素の区切り
                    let elem = current.trim();
                    if !elem.is_empty() {
                        elements.push(serde_json::Value::String(elem.to_string()));
                    }
                    current.clear();
                }
                _ if in_quotes => {
                    current.push(ch);
                }
                _ if !in_quotes && !ch.is_whitespace() => {
                    current.push(ch);
                }
                _ => {}
            }
        }

        // 最後の要素を追加
        let elem = current.trim();
        if !elem.is_empty() {
            elements.push(serde_json::Value::String(elem.to_string()));
        }

        return serde_json::Value::Array(elements);
    }

    // パースに失敗した場合は文字列として返す
    serde_json::Value::String(s.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: usize,
    pub execution_time_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlTemplate {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub query: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveTemplateRequest {
    pub id: i64, // 0 = 新規作成
    pub name: String,
    pub description: String,
    pub query: String,
}

/// 任意のSQLクエリを実行し、結果を返す
#[tauri::command]
pub async fn execute_sql(
    db_manager: State<'_, DatabaseManager>,
    query: String,
) -> Result<SqlQueryResult, String> {
    let start_time = Instant::now();

    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    eprintln!(
        "[SQL] Database path: {}",
        db_manager.get_db_path().display()
    );

    // クエリをトリムして、空の場合はエラーを返す
    let query = query.trim();
    if query.is_empty() {
        return Err("クエリが空です".to_string());
    }

    eprintln!("[SQL] Executing query: {}", query);

    // 基本的なクエリ検証：セミコロンで複数のステートメントに分割されている場合、
    // 最初のステートメントのみを実行（セキュリティとエラー防止のため）
    let query_parts: Vec<&str> = query.split(';').collect();
    let query_to_execute = if query_parts.len() > 1 {
        let first_stmt = query_parts[0].trim();
        // 2番目以降のステートメントが空でない場合は警告
        if query_parts.iter().skip(1).any(|s| !s.trim().is_empty()) {
            eprintln!(
                "[SQL WARN] Multiple statements detected. Only executing the first statement."
            );
            eprintln!("[SQL WARN] Original query: {}", query);
            eprintln!("[SQL WARN] Executing: {}", first_stmt);
        }
        first_stmt
    } else {
        query
    };

    // SQLクエリの基本的な妥当性チェック（DuckDBに渡す前に検証）
    if let Err(e) = validate_sql_query(query_to_execute) {
        eprintln!("[SQL ERROR] Query validation failed: {}", e);
        return Err(e);
    }

    // クエリの種類を判定（SELECT系かそれ以外か）
    // コメント（-- や /* */）をスキップして最初のSQLキーワードを取得
    let query_type = query_to_execute
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("--"))
        .flat_map(|line| line.split_whitespace())
        .find(|word| !word.is_empty())
        .unwrap_or("")
        .to_uppercase();

    let result = if query_type == "SELECT"
        || query_type == "WITH"
        || query_type == "SHOW"
        || query_type == "DESCRIBE"
        || query_type == "PRAGMA"
    {
        // SELECT系クエリの処理
        // LIST型カラムを自動変換するための前処理
        let processed_query = match preprocess_query_for_list_columns(&conn, query_to_execute) {
            Ok(q) => q,
            Err(e) => {
                eprintln!(
                    "[SQL WARN] Failed to preprocess query: {}, using original query",
                    e
                );
                query_to_execute.to_string()
            }
        };

        let mut stmt = match conn.prepare(&processed_query) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[SQL ERROR] Failed to prepare: {}", e);
                return Err(format!("SQL構文エラー: {}", e));
            }
        };

        eprintln!("[SQL] Statement prepared successfully");

        // クエリを実行してRowsを取得
        let mut rows = match stmt.query([]) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[SQL ERROR] Failed to execute query: {}", e);
                return Err(format!("クエリ実行エラー: {}", e));
            }
        };

        eprintln!("[SQL] Query executed, collecting rows...");

        // カラム情報と行データを収集
        let mut columns: Vec<String> = Vec::new();
        let mut row_data = Vec::new();
        let mut column_count = 0;

        // 最初の行からカラム情報を取得
        let first_row_result = match rows.next() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[SQL ERROR] Failed to fetch first row: {}", e);
                return Err(format!("行データ取得エラー: {}", e));
            }
        };

        if let Some(first_row) = first_row_result {
            column_count = first_row.as_ref().column_count();
            eprintln!("[SQL] Column count: {}", column_count);

            columns = (0..column_count)
                .filter_map(|i| {
                    first_row
                        .as_ref()
                        .column_name(i)
                        .ok()
                        .map(|s| s.to_string())
                })
                .collect();

            eprintln!("[SQL] Columns: {:?}", columns);

            // 最初の行のデータを処理
            let mut row_values = Vec::new();
            for i in 0..column_count {
                let value = match first_row.get_ref(i) {
                    Ok(ValueRef::Null) => serde_json::Value::Null,
                    Ok(ValueRef::Boolean(b)) => serde_json::Value::Bool(b),
                    Ok(ValueRef::TinyInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::SmallInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::Int(i)) => serde_json::json!(i),
                    Ok(ValueRef::BigInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::HugeInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UTinyInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::USmallInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UBigInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::Float(f)) => serde_json::json!(f),
                    Ok(ValueRef::Double(f)) => serde_json::json!(f),
                    Ok(ValueRef::Decimal(d)) => serde_json::json!(d.to_string()),
                    Ok(ValueRef::Timestamp(unit, value)) => {
                        // Timestampを文字列に変換
                        let datetime = match unit {
                            TimeUnit::Second => Local.timestamp_opt(value, 0).single(),
                            TimeUnit::Millisecond => Local.timestamp_millis_opt(value).single(),
                            TimeUnit::Microsecond => Local.timestamp_micros(value).single(),
                            TimeUnit::Nanosecond => {
                                let secs = value / 1_000_000_000;
                                let nsecs = (value % 1_000_000_000) as u32;
                                Local.timestamp_opt(secs, nsecs).single()
                            }
                        };
                        match datetime {
                            Some(dt) => serde_json::Value::String(
                                dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
                            ),
                            None => {
                                serde_json::Value::String(format!("<Invalid Timestamp: {}>", value))
                            }
                        }
                    }
                    Ok(ValueRef::Text(s)) => {
                        serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    Ok(ValueRef::Blob(b)) => {
                        serde_json::Value::String(format!("<BLOB {} bytes>", b.len()))
                    }
                    Ok(ValueRef::Date32(_)) => {
                        // Dateを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
                        }
                    }
                    Ok(ValueRef::Time64(_, _)) => {
                        // Timeを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
                        }
                    }
                    Ok(ValueRef::Interval { .. }) => {
                        serde_json::Value::String("<INTERVAL>".to_string())
                    }
                    Ok(ValueRef::List(_, _)) => {
                        // 専用の関数でListを処理
                        extract_list_from_row(first_row, i)
                    }
                    Ok(ValueRef::Enum(_, _)) => {
                        // Enumを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<ENUM>".to_string()),
                        }
                    }
                    Ok(ValueRef::Struct(..)) => {
                        // Structを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<STRUCT>".to_string()),
                        }
                    }
                    Ok(ValueRef::Union(_, _)) => serde_json::Value::String("<UNION>".to_string()),
                    Ok(ValueRef::Map(_, _)) => serde_json::Value::String("<MAP>".to_string()),
                    Ok(ValueRef::Array(_, _)) => serde_json::Value::String("<ARRAY>".to_string()),
                    Err(_) => serde_json::Value::Null,
                };
                row_values.push(value);
            }
            row_data.push(row_values);
        }

        // 残りの行データを収集
        loop {
            let row_result = match rows.next() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[SQL ERROR] Failed to fetch row: {}", e);
                    return Err(format!("行データ取得エラー: {}", e));
                }
            };

            let row = match row_result {
                Some(r) => r,
                None => break,
            };
            let mut row_values = Vec::new();
            for i in 0..column_count {
                let value = match row.get_ref(i) {
                    Ok(ValueRef::Null) => serde_json::Value::Null,
                    Ok(ValueRef::Boolean(b)) => serde_json::Value::Bool(b),
                    Ok(ValueRef::TinyInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::SmallInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::Int(i)) => serde_json::json!(i),
                    Ok(ValueRef::BigInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::HugeInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UTinyInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::USmallInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::UBigInt(i)) => serde_json::json!(i),
                    Ok(ValueRef::Float(f)) => serde_json::json!(f),
                    Ok(ValueRef::Double(f)) => serde_json::json!(f),
                    Ok(ValueRef::Decimal(d)) => serde_json::json!(d.to_string()),
                    Ok(ValueRef::Timestamp(unit, value)) => {
                        // Timestampを文字列に変換
                        let datetime = match unit {
                            TimeUnit::Second => Local.timestamp_opt(value, 0).single(),
                            TimeUnit::Millisecond => Local.timestamp_millis_opt(value).single(),
                            TimeUnit::Microsecond => Local.timestamp_micros(value).single(),
                            TimeUnit::Nanosecond => {
                                let secs = value / 1_000_000_000;
                                let nsecs = (value % 1_000_000_000) as u32;
                                Local.timestamp_opt(secs, nsecs).single()
                            }
                        };
                        match datetime {
                            Some(dt) => serde_json::Value::String(
                                dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
                            ),
                            None => {
                                serde_json::Value::String(format!("<Invalid Timestamp: {}>", value))
                            }
                        }
                    }
                    Ok(ValueRef::Text(s)) => {
                        serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    Ok(ValueRef::Blob(b)) => {
                        serde_json::Value::String(format!("<BLOB {} bytes>", b.len()))
                    }
                    Ok(ValueRef::Date32(_)) => {
                        // Dateを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
                        }
                    }
                    Ok(ValueRef::Time64(_, _)) => {
                        // Timeを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
                        }
                    }
                    Ok(ValueRef::Interval { .. }) => {
                        serde_json::Value::String("<INTERVAL>".to_string())
                    }
                    Ok(ValueRef::List(_, _)) => {
                        // 専用の関数でListを処理
                        extract_list_from_row(row, i)
                    }
                    Ok(ValueRef::Enum(_, _)) => {
                        // Enumを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<ENUM>".to_string()),
                        }
                    }
                    Ok(ValueRef::Struct(..)) => {
                        // Structを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<STRUCT>".to_string()),
                        }
                    }
                    Ok(ValueRef::Union(_, _)) => serde_json::Value::String("<UNION>".to_string()),
                    Ok(ValueRef::Map(_, _)) => serde_json::Value::String("<MAP>".to_string()),
                    Ok(ValueRef::Array(_, _)) => serde_json::Value::String("<ARRAY>".to_string()),
                    Err(_) => serde_json::Value::Null,
                };
                row_values.push(value);
            }
            row_data.push(row_values);
        }

        let execution_time = start_time.elapsed().as_millis();

        eprintln!(
            "[SQL] Query completed: {} columns, {} rows, {}ms",
            columns.len(),
            row_data.len(),
            execution_time
        );

        SqlQueryResult {
            columns,
            rows: row_data,
            affected_rows: 0,
            execution_time_ms: execution_time,
        }
    } else {
        // INSERT/UPDATE/DELETE/CREATE/DROP等の処理
        let affected = match conn.execute(query_to_execute, &[] as &[&dyn duckdb::ToSql]) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[SQL ERROR] Failed to execute: {}", e);
                return Err(format!("SQL実行エラー: {}", e));
            }
        };

        let execution_time = start_time.elapsed().as_millis();

        SqlQueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: affected,
            execution_time_ms: execution_time,
        }
    };

    Ok(result)
}

/// 全てのSQLテンプレートを取得
#[tauri::command]
pub async fn list_sql_templates(
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<SqlTemplate>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, query, 
             CAST(created_at AS VARCHAR) as created_at, 
             CAST(updated_at AS VARCHAR) as updated_at 
             FROM sql_templates 
             ORDER BY updated_at DESC",
        )
        .db_context("prepare query")
        .map_err(|e| e.to_string())?;

    let templates = stmt
        .query_map([], |row| {
            Ok(SqlTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).unwrap_or_else(|_| String::new()),
                query: row.get(3)?,
                created_at: row.get(4).unwrap_or_else(|_| String::new()),
                updated_at: row.get(5).unwrap_or_else(|_| String::new()),
            })
        })
        .db_context("query templates")
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .db_context("fetch templates")
        .map_err(|e| e.to_string())?;

    Ok(templates)
}

/// SQLテンプレートを保存（新規作成または更新）
#[tauri::command]
pub async fn save_sql_template(
    db_manager: State<'_, DatabaseManager>,
    request: SaveTemplateRequest,
) -> Result<SqlTemplate, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let id = if request.id > 0 {
        // 更新
        conn.execute(
            "UPDATE sql_templates 
             SET name = ?, description = ?, query = ?, updated_at = CURRENT_TIMESTAMP 
             WHERE id = ?",
            params![
                &request.name,
                &request.description,
                &request.query,
                request.id
            ],
        )
        .db_context("update template")
        .map_err(|e| e.to_string())?;

        request.id
    } else {
        // 新規作成
        conn.execute(
            "INSERT INTO sql_templates (name, description, query) 
             VALUES (?, ?, ?)",
            params![&request.name, &request.description, &request.query],
        )
        .db_context("insert template")
        .map_err(|e| e.to_string())?;

        // 最後に挿入されたIDを取得
        let mut stmt = conn
            .prepare("SELECT currval('sql_templates_id_seq')")
            .db_context("get last insert id")
            .map_err(|e| e.to_string())?;

        stmt.query_row([], |row| row.get(0))
            .db_context("fetch last insert id")
            .map_err(|e| e.to_string())?
    };

    // 保存したテンプレートを取得して返す
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, query, 
             CAST(created_at AS VARCHAR) as created_at, 
             CAST(updated_at AS VARCHAR) as updated_at 
             FROM sql_templates 
             WHERE id = ?",
        )
        .db_context("prepare query")
        .map_err(|e| e.to_string())?;

    let template = stmt
        .query_row(params![id], |row| {
            Ok(SqlTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2).unwrap_or_else(|_| String::new()),
                query: row.get(3)?,
                created_at: row.get(4).unwrap_or_else(|_| String::new()),
                updated_at: row.get(5).unwrap_or_else(|_| String::new()),
            })
        })
        .map_err(|e| format!("Failed to retrieve saved template: {}", e))?;

    Ok(template)
}

/// SQLテンプレートを削除
#[tauri::command]
pub async fn delete_sql_template(
    db_manager: State<'_, DatabaseManager>,
    id: i64,
) -> Result<(), String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let affected = conn
        .execute("DELETE FROM sql_templates WHERE id = ?", params![id])
        .db_context("delete template")
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("Template not found".to_string());
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub column_count: usize,
}

/// データベース内のテーブル一覧を取得
#[tauri::command]
pub async fn list_database_tables(
    db_manager: State<'_, DatabaseManager>,
) -> Result<Vec<TableInfo>, String> {
    let conn = db_manager
        .get_connection()
        .await
        .db_context("get database connection")
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT table_name 
             FROM information_schema.tables 
             WHERE table_schema = 'main' 
             ORDER BY table_name",
        )
        .db_context("prepare query")
        .map_err(|e| e.to_string())?;

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .db_context("query tables")
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .db_context("fetch tables")
        .map_err(|e| e.to_string())?;

    eprintln!("[SQL] Found {} tables in database", table_names.len());

    // 各テーブルのカラム数と行数を取得
    let mut tables = Vec::new();
    for table_name in table_names {
        // カラム数を取得
        let column_count_query =
            format!("SELECT COUNT(*) FROM pragma_table_info('{}')", table_name);
        let mut stmt = conn
            .prepare(&column_count_query)
            .db_context("prepare column count query")
            .map_err(|e| e.to_string())?;

        let column_count: i64 = stmt
            .query_row([], |row| row.get(0))
            .db_context("get column count")
            .map_err(|e| e.to_string())?;

        // 行数を取得
        let row_count_query = format!("SELECT COUNT(*) FROM {}", table_name);
        let mut stmt = conn
            .prepare(&row_count_query)
            .db_context("prepare row count query")
            .map_err(|e| e.to_string())?;

        let row_count: i64 = stmt
            .query_row([], |row| row.get(0))
            .db_context("get row count")
            .map_err(|e| e.to_string())?;

        eprintln!(
            "[SQL] Table '{}': {} columns, {} rows",
            table_name, column_count, row_count
        );

        tables.push(TableInfo {
            table_name,
            column_count: column_count as usize,
        });
    }

    Ok(tables)
}
