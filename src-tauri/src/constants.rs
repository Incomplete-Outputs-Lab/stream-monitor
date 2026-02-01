/// アプリケーション全体で使用される定数
#[allow(dead_code)]
pub mod twitch {
    /// トークンの有効期限チェック閾値（分）
    pub const TOKEN_EXPIRY_THRESHOLD_MINUTES: i64 = 30;

    /// 1リクエストあたりの最大ストリーム数
    pub const MAX_STREAMS_PER_REQUEST: usize = 100;

    /// 取得する最大ストリーム総数
    pub const MAX_TOTAL_STREAMS: usize = 500;

    /// レート制限バケットの容量（リクエスト数/分）
    pub const RATE_LIMIT_BUCKET_CAPACITY: usize = 800;

    /// レート制限ウィンドウ（秒）
    pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;

    /// 401エラーステータスコード
    pub const ERROR_UNAUTHORIZED: &str = "401";

    /// Unauthorizedエラーテキスト
    pub const ERROR_UNAUTHORIZED_TEXT: &str = "Unauthorized";
}

pub mod youtube {
    /// OAuth認証URL
    pub const OAUTH_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

    /// OAuthトークンURL
    pub const OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

    /// YouTube読み取り専用スコープ
    #[allow(dead_code)]
    pub const SCOPE_YOUTUBE_READONLY: &str = "https://www.googleapis.com/auth/youtube.readonly";

    /// APIレスポンス部分: ID
    pub const PART_ID: &str = "id";

    /// APIレスポンス部分: スニペット
    pub const PART_SNIPPET: &str = "snippet";

    /// APIレスポンス部分: コンテンツ詳細
    pub const PART_CONTENT_DETAILS: &str = "contentDetails";

    /// APIレスポンス部分: 著者詳細
    pub const PART_AUTHOR_DETAILS: &str = "authorDetails";

    /// イベントタイプ: ライブ
    pub const EVENT_TYPE_LIVE: &str = "live";

    /// タイプ: ビデオ
    pub const TYPE_VIDEO: &str = "video";

    /// デフォルトの最大結果数
    pub const MAX_RESULTS_DEFAULT: u32 = 1;

    /// メッセージタイプ: スーパーチャット
    pub const MESSAGE_TYPE_SUPERCHAT: &str = "superchat";

    /// メッセージタイプ: ファン資金
    pub const MESSAGE_TYPE_FAN_FUNDING: &str = "fanfunding";

    /// メッセージタイプ: スポンサー
    pub const MESSAGE_TYPE_SPONSOR: &str = "sponsor";

    /// メッセージタイプ: 通常
    pub const MESSAGE_TYPE_NORMAL: &str = "normal";

    /// プラットフォーム名
    pub const PLATFORM_NAME: &str = "youtube";
}

#[allow(dead_code)]
pub mod database {
    /// チャットメッセージのバッチサイズ
    pub const CHAT_BATCH_SIZE: usize = 100;

    /// バッチフラッシュ間隔（秒）
    pub const BATCH_FLUSH_INTERVAL_SECS: u64 = 5;

    /// Twitchプラットフォーム名
    pub const PLATFORM_TWITCH: &str = "twitch";

    /// YouTubeプラットフォーム名
    pub const PLATFORM_YOUTUBE: &str = "youtube";
}
