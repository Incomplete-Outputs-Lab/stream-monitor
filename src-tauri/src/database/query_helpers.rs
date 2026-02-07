/// DuckDB特殊型を扱うためのSQLフラグメント生成
///
/// DuckDBのLIST型やTIMESTAMP型を直接SELECTすると、Rustのduckdbクレートで
/// 型変換エラーが発生します。このモジュールは、それらの型を安全に扱うための
/// SQLフラグメントを生成します。
pub mod chat_query {
    /// badgesカラムのSELECT句（常にVARCHARにキャスト）
    ///
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}", chat_query::badges_select("cm"));
    /// // 生成されるSQL: "SELECT CAST(cm.badges AS VARCHAR) as badges"
    /// ```
    pub fn badges_select(table_alias: &str) -> String {
        format!("CAST({}.badges AS VARCHAR) as badges", table_alias)
    }

    /// timestampカラムのSELECT句（常にVARCHARにキャスト）
    ///
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}", chat_query::timestamp_select("cm"));
    /// // 生成されるSQL: "SELECT CAST(cm.timestamp AS VARCHAR) as timestamp"
    /// ```
    #[allow(dead_code)]
    pub fn timestamp_select(table_alias: &str) -> String {
        format!("CAST({}.timestamp AS VARCHAR) as timestamp", table_alias)
    }

    /// chat_messagesの基本SELECT句（よく使うカラムセット）
    ///
    /// DuckDB特殊型（badges, timestamp）を含む標準的なカラムセットを生成します。
    ///
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {} FROM chat_messages cm",
    ///                   chat_query::standard_columns("cm"));
    /// ```
    #[allow(dead_code)]
    pub fn standard_columns(table_alias: &str) -> String {
        format!(
            "{}.id, {}.channel_id, {}.stream_id, {}, {}.platform, \
             {}.user_id, {}.user_name, {}.message, {}.message_type, {}, {}.badge_info",
            table_alias,
            table_alias,
            table_alias,
            timestamp_select(table_alias),
            table_alias,
            table_alias,
            table_alias,
            table_alias,
            table_alias,
            badges_select(table_alias),
            table_alias
        )
    }
}

/// stream_stats テーブル用のクエリヘルパー
///
/// DuckDBのTIMESTAMP型を安全に扱うためのSQLフラグメントを生成します。
pub mod stream_stats_query {
    /// collected_atカラムのSELECT句（常にVARCHARにキャスト）
    ///
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}", stream_stats_query::collected_at_select("ss"));
    /// // 生成されるSQL: "SELECT CAST(ss.collected_at AS VARCHAR) as collected_at"
    /// ```
    #[allow(dead_code)]
    pub fn collected_at_select(table_alias: &str) -> String {
        format!(
            "CAST({}.collected_at AS VARCHAR) as collected_at",
            table_alias
        )
    }

    /// インターバル計算のSQLフラグメント（LEAD関数使用）
    ///
    /// 次のレコードとの時間差を分単位で計算します。
    ///
    /// # Arguments
    /// * `table_alias` - テーブルエイリアス（例: "ss"）
    /// * `partition_by` - PARTITION BY句の内容（例: "stream_id"）
    ///
    /// # Examples
    /// ```
    /// let sql = format!("SELECT {}",
    ///     stream_stats_query::interval_calculation("ss", "ss.stream_id"));
    /// ```
    #[allow(dead_code)]
    pub fn interval_calculation(table_alias: &str, partition_by: &str) -> String {
        format!(
            "EXTRACT(EPOCH FROM (\
                LEAD({}.collected_at) OVER (PARTITION BY {} ORDER BY {}.collected_at) \
                - {}.collected_at\
            )) / 60.0 AS interval_minutes",
            table_alias, partition_by, table_alias, table_alias
        )
    }

    /// 標準的なインターバル計算（stream_idでパーティション、NULL時の代替パーティション付き）
    ///
    /// stream_idがNULLの場合、channel_name + dateでパーティション分割します。
    pub fn interval_with_fallback(table_alias: &str) -> String {
        format!(
            "EXTRACT(EPOCH FROM (\
                LEAD({}.collected_at) OVER (\
                    PARTITION BY COALESCE(\
                        CAST({}.stream_id AS VARCHAR), \
                        {}.channel_name || '_' || CAST(DATE({}.collected_at) AS VARCHAR)\
                    ) \
                    ORDER BY {}.collected_at\
                ) - {}.collected_at\
            )) / 60.0 AS interval_minutes",
            table_alias, table_alias, table_alias, table_alias, table_alias, table_alias
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badges_select() {
        let sql = chat_query::badges_select("cm");
        assert_eq!(sql, "CAST(cm.badges AS VARCHAR) as badges");
    }

    #[test]
    fn test_timestamp_select() {
        let sql = chat_query::timestamp_select("cm");
        assert_eq!(sql, "CAST(cm.timestamp AS VARCHAR) as timestamp");
    }

    #[test]
    fn test_standard_columns() {
        let sql = chat_query::standard_columns("cm");
        assert!(sql.contains("cm.id"));
        assert!(sql.contains("CAST(cm.badges AS VARCHAR) as badges"));
        assert!(sql.contains("CAST(cm.timestamp AS VARCHAR) as timestamp"));
    }

    #[test]
    fn test_collected_at_select() {
        let sql = stream_stats_query::collected_at_select("ss");
        assert_eq!(sql, "CAST(ss.collected_at AS VARCHAR) as collected_at");
    }

    #[test]
    fn test_interval_calculation() {
        let sql = stream_stats_query::interval_calculation("ss", "ss.stream_id");
        assert!(sql.contains("LEAD(ss.collected_at)"));
        assert!(sql.contains("PARTITION BY ss.stream_id"));
        assert!(sql.contains("interval_minutes"));
    }
}
