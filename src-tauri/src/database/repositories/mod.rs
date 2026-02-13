pub mod aggregation_repository;
/// レポジトリパターンモジュール
///
/// データベースアクセスを抽象化し、型変換ロジックを統一します。
pub mod base;
pub mod channel_repository;
pub mod chat_message_repository;
pub mod game_category_repository;
pub mod multiview_repository;
pub mod sql_template_repository;
pub mod stream_repository;
pub mod stream_stats_repository;

// Re-exports
pub use aggregation_repository::AggregationRepository;
pub use channel_repository::ChannelRepository;
pub use chat_message_repository::ChatMessageRepository;
pub use game_category_repository::GameCategoryRepository;
pub use sql_template_repository::{SqlTemplate, SqlTemplateRepository};
pub use stream_repository::{StreamInfo, StreamRepository, TimelinePoint};
pub use stream_stats_repository::StreamStatsRepository;
