use crate::database::DatabaseManager;
use duckdb::{params, types::ValueRef};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: Option<usize>,
    pub execution_time_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveTemplateRequest {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    eprintln!(
        "[SQL] Database path: {}",
        db_manager.get_db_path().display()
    );

    // クエリをトリムして、空の場合はエラーを返す
    let query = query.trim();
    if query.is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    eprintln!("[SQL] Executing query: {}", query);

    // クエリの種類を判定（SELECT系かそれ以外か）
    // コメント（-- や /* */）をスキップして最初のSQLキーワードを取得
    let query_type = query
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
        let mut stmt = conn.prepare(query).map_err(|e| {
            eprintln!("[SQL ERROR] Failed to prepare: {}", e);
            format!("Failed to prepare query: {}", e)
        })?;

        eprintln!("[SQL] Statement prepared successfully");

        // クエリを実行してRowsを取得
        let mut rows = stmt.query([]).map_err(|e| {
            eprintln!("[SQL ERROR] Failed to execute query: {}", e);
            format!("Failed to execute query: {}", e)
        })?;

        eprintln!("[SQL] Query executed, collecting rows...");

        // カラム情報と行データを収集
        let mut columns: Vec<String> = Vec::new();
        let mut row_data = Vec::new();
        let mut column_count = 0;

        // 最初の行からカラム情報を取得
        if let Some(first_row) = rows.next().map_err(|e| {
            eprintln!("[SQL ERROR] Failed to fetch first row: {}", e);
            format!("Failed to fetch first row: {}", e)
        })? {
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
                    Ok(ValueRef::Timestamp(_, _)) => {
                        // Timestampを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
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
                        // Listを文字列に変換
                        match first_row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<LIST>".to_string()),
                        }
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
        while let Some(row) = rows.next().map_err(|e| {
            eprintln!("[SQL ERROR] Failed to fetch row: {}", e);
            format!("Failed to fetch row: {}", e)
        })? {
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
                    Ok(ValueRef::Timestamp(_, _)) => {
                        // Timestampを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::Null,
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
                        // Listを文字列に変換
                        match row.get::<_, String>(i) {
                            Ok(s) => serde_json::Value::String(s),
                            Err(_) => serde_json::Value::String("<LIST>".to_string()),
                        }
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
            affected_rows: None,
            execution_time_ms: execution_time,
        }
    } else {
        // INSERT/UPDATE/DELETE/CREATE/DROP等の処理
        let affected = conn
            .execute(query, &[] as &[&dyn duckdb::ToSql])
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        let execution_time = start_time.elapsed().as_millis();

        SqlQueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: Some(affected),
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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, query, 
             CAST(created_at AS VARCHAR) as created_at, 
             CAST(updated_at AS VARCHAR) as updated_at 
             FROM sql_templates 
             ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(SqlTemplate {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                query: row.get(3)?,
                created_at: Some(row.get(4)?),
                updated_at: Some(row.get(5)?),
            })
        })
        .map_err(|e| format!("Failed to query templates: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to fetch templates: {}", e))?;

    Ok(templates)
}

/// 特定のSQLテンプレートを取得
#[tauri::command]
pub async fn get_sql_template(
    db_manager: State<'_, DatabaseManager>,
    id: i64,
) -> Result<SqlTemplate, String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, query, 
             CAST(created_at AS VARCHAR) as created_at, 
             CAST(updated_at AS VARCHAR) as updated_at 
             FROM sql_templates 
             WHERE id = ?",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let template = stmt
        .query_row(params![id], |row| {
            Ok(SqlTemplate {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                query: row.get(3)?,
                created_at: Some(row.get(4)?),
                updated_at: Some(row.get(5)?),
            })
        })
        .map_err(|e| format!("Template not found: {}", e))?;

    Ok(template)
}

/// SQLテンプレートを保存（新規作成または更新）
#[tauri::command]
pub async fn save_sql_template(
    db_manager: State<'_, DatabaseManager>,
    request: SaveTemplateRequest,
) -> Result<SqlTemplate, String> {
    let id = {
        let conn = db_manager
            .get_connection()
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        if let Some(id) = request.id {
            // 更新
            conn.execute(
                "UPDATE sql_templates 
                 SET name = ?, description = ?, query = ?, updated_at = CURRENT_TIMESTAMP 
                 WHERE id = ?",
                params![&request.name, &request.description, &request.query, id],
            )
            .map_err(|e| format!("Failed to update template: {}", e))?;

            id
        } else {
            // 新規作成
            conn.execute(
                "INSERT INTO sql_templates (name, description, query) 
                 VALUES (?, ?, ?)",
                params![&request.name, &request.description, &request.query],
            )
            .map_err(|e| format!("Failed to insert template: {}", e))?;

            // 最後に挿入されたIDを取得
            let mut stmt = conn
                .prepare("SELECT currval('sql_templates_id_seq')")
                .map_err(|e| format!("Failed to get last insert id: {}", e))?;

            stmt.query_row([], |row| row.get(0))
                .map_err(|e| format!("Failed to fetch last insert id: {}", e))?
        }
    };

    get_sql_template(db_manager, id).await
}

/// SQLテンプレートを削除
#[tauri::command]
pub async fn delete_sql_template(
    db_manager: State<'_, DatabaseManager>,
    id: i64,
) -> Result<(), String> {
    let conn = db_manager
        .get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let affected = conn
        .execute("DELETE FROM sql_templates WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete template: {}", e))?;

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
        .map_err(|e| format!("Failed to get database connection: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT table_name 
             FROM information_schema.tables 
             WHERE table_schema = 'main' 
             ORDER BY table_name",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query tables: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to fetch tables: {}", e))?;

    eprintln!("[SQL] Found {} tables in database", table_names.len());

    // 各テーブルのカラム数と行数を取得
    let mut tables = Vec::new();
    for table_name in table_names {
        // カラム数を取得
        let column_count_query =
            format!("SELECT COUNT(*) FROM pragma_table_info('{}')", table_name);
        let mut stmt = conn
            .prepare(&column_count_query)
            .map_err(|e| format!("Failed to prepare column count query: {}", e))?;

        let column_count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| format!("Failed to get column count: {}", e))?;

        // 行数を取得
        let row_count_query = format!("SELECT COUNT(*) FROM {}", table_name);
        let mut stmt = conn
            .prepare(&row_count_query)
            .map_err(|e| format!("Failed to prepare row count query: {}", e))?;

        let row_count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| format!("Failed to get row count: {}", e))?;

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
