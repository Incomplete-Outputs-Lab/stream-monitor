/// レポジトリパターンの共通基盤
///
/// クエリフィルターとヘルパートレイトを定義
use serde::{Deserialize, Serialize};

/// 時間範囲フィルター
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeRangeFilter {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// チャンネル/配信フィルター
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelFilter {
    pub channel_id: Option<i64>,
    pub stream_id: Option<i64>,
}

/// クエリフィルタートレイト
#[allow(dead_code)]
pub trait QueryFilter {
    /// WHERE句とパラメータを生成
    ///
    /// # Returns
    /// (WHERE句の文字列, パラメータのVec)
    fn to_where_clause(&self, table_alias: &str) -> (String, Vec<String>);
}

impl QueryFilter for TimeRangeFilter {
    fn to_where_clause(&self, table_alias: &str) -> (String, Vec<String>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref start) = self.start_time {
            clauses.push(format!("{}.timestamp >= ?", table_alias));
            params.push(start.clone());
        }

        if let Some(ref end) = self.end_time {
            clauses.push(format!("{}.timestamp <= ?", table_alias));
            params.push(end.clone());
        }

        if clauses.is_empty() {
            (String::new(), params)
        } else {
            (format!(" AND {}", clauses.join(" AND ")), params)
        }
    }
}

impl QueryFilter for ChannelFilter {
    fn to_where_clause(&self, table_alias: &str) -> (String, Vec<String>) {
        let mut clauses = Vec::new();
        let params = Vec::new();

        if let Some(ch_id) = self.channel_id {
            clauses.push(format!("{}.channel_id = {}", table_alias, ch_id));
        }

        if let Some(st_id) = self.stream_id {
            clauses.push(format!("{}.stream_id = {}", table_alias, st_id));
        }

        if clauses.is_empty() {
            (String::new(), params)
        } else {
            (format!(" AND {}", clauses.join(" AND ")), params)
        }
    }
}

/// stream_stats用の時間範囲フィルター（collected_atを使用）
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct StreamStatsTimeFilter {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl StreamStatsTimeFilter {
    #[allow(dead_code)]
    pub fn to_where_clause(&self, table_alias: &str) -> (String, Vec<String>) {
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref start) = self.start_time {
            clauses.push(format!("{}.collected_at >= ?", table_alias));
            params.push(start.clone());
        }

        if let Some(ref end) = self.end_time {
            clauses.push(format!("{}.collected_at <= ?", table_alias));
            params.push(end.clone());
        }

        if clauses.is_empty() {
            (String::new(), params)
        } else {
            (format!(" AND {}", clauses.join(" AND ")), params)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_filter() {
        let filter = TimeRangeFilter {
            start_time: Some("2024-01-01".to_string()),
            end_time: Some("2024-12-31".to_string()),
        };
        let (clause, params) = filter.to_where_clause("cm");
        assert!(clause.contains("cm.timestamp >= ?"));
        assert!(clause.contains("cm.timestamp <= ?"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_channel_filter() {
        let filter = ChannelFilter {
            channel_id: Some(1),
            stream_id: Some(10),
        };
        let (clause, params) = filter.to_where_clause("cm");
        assert!(clause.contains("cm.channel_id = ?"));
        assert!(clause.contains("cm.stream_id = ?"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_empty_filters() {
        let filter = TimeRangeFilter::default();
        let (clause, params) = filter.to_where_clause("cm");
        assert!(clause.is_empty());
        assert!(params.is_empty());
    }
}
