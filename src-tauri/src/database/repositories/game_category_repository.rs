/// GameCategoryRepository - game_categoriesテーブル専用レポジトリ
///
/// Twitchゲームカテゴリの管理を行います。
use crate::database::models::GameCategory;
use chrono::Local;
use duckdb::Connection;

pub struct GameCategoryRepository;

impl GameCategoryRepository {
    /// カテゴリを挿入または更新（UPSERT）
    ///
    /// game_idが既存の場合は更新、存在しない場合は挿入します。
    pub fn upsert_category(
        conn: &Connection,
        game_id: &str,
        game_name: &str,
        box_art_url: Option<&str>,
    ) -> Result<(), duckdb::Error> {
        let now = Local::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO game_categories (game_id, game_name, box_art_url, last_updated)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(game_id) DO UPDATE SET
                game_name = excluded.game_name,
                box_art_url = excluded.box_art_url,
                last_updated = excluded.last_updated
            "#,
            duckdb::params![game_id, game_name, box_art_url, now],
        )?;
        Ok(())
    }

    /// 全カテゴリを取得
    ///
    /// 更新日時の降順でソートして返します。
    pub fn get_all_categories(conn: &Connection) -> Result<Vec<GameCategory>, duckdb::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                game_id,
                game_name,
                box_art_url,
                CAST(last_updated AS VARCHAR) as last_updated
            FROM game_categories
            ORDER BY last_updated DESC
            "#,
        )?;

        let results = stmt.query_map([], |row| {
            Ok(GameCategory {
                game_id: row.get(0)?,
                game_name: row.get(1)?,
                box_art_url: row.get(2)?,
                last_updated: row.get(3)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// IDでカテゴリを取得
    pub fn get_category_by_id(
        conn: &Connection,
        game_id: &str,
    ) -> Result<Option<GameCategory>, duckdb::Error> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                game_id,
                game_name,
                box_art_url,
                CAST(last_updated AS VARCHAR) as last_updated
            FROM game_categories
            WHERE game_id = ?
            "#,
        )?;

        let mut results = stmt.query_map([game_id], |row| {
            Ok(GameCategory {
                game_id: row.get(0)?,
                game_name: row.get(1)?,
                box_art_url: row.get(2)?,
                last_updated: row.get(3)?,
            })
        })?;

        results.next().transpose()
    }

    /// カテゴリを削除
    pub fn delete_category(conn: &Connection, game_id: &str) -> Result<(), duckdb::Error> {
        conn.execute("DELETE FROM game_categories WHERE game_id = ?", [game_id])?;
        Ok(())
    }

    /// カテゴリ名で検索
    ///
    /// 部分一致検索を行います（LIKE '%query%'）
    pub fn search_categories(
        conn: &Connection,
        query: &str,
    ) -> Result<Vec<GameCategory>, duckdb::Error> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            r#"
            SELECT
                game_id,
                game_name,
                box_art_url,
                CAST(last_updated AS VARCHAR) as last_updated
            FROM game_categories
            WHERE game_name LIKE ?
            ORDER BY game_name
            LIMIT 100
            "#,
        )?;

        let results = stmt.query_map([&search_pattern], |row| {
            Ok(GameCategory {
                game_id: row.get(0)?,
                game_name: row.get(1)?,
                box_art_url: row.get(2)?,
                last_updated: row.get(3)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// カテゴリが存在するか確認
    #[allow(dead_code)]
    pub fn exists(conn: &Connection, game_id: &str) -> Result<bool, duckdb::Error> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM game_categories WHERE game_id = ?",
            [game_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
