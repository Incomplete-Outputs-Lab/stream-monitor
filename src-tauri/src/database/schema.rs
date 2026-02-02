use duckdb::Connection;

pub fn init_database(conn: &Connection) -> Result<(), duckdb::Error> {
    eprintln!("[Schema] Starting database schema initialization...");

    eprintln!("[Schema] Step 1: Creating channels table...");
    // Create sequence for channels table
    conn.execute("CREATE SEQUENCE IF NOT EXISTS channels_id_seq START 1", [])?;
    eprintln!("[Schema] channels sequence created");

    // channels テーブル: 監視対象チャンネル設定
    match conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id BIGINT PRIMARY KEY DEFAULT nextval('channels_id_seq'),
            platform TEXT NOT NULL CHECK(platform IN ('twitch', 'youtube')),
            channel_id TEXT NOT NULL,
            channel_name TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            poll_interval INTEGER NOT NULL DEFAULT 60,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(platform, channel_id)
        )
        "#,
        [],
    ) {
        Ok(_) => {
            eprintln!("[Schema] Step 1: channels table created successfully");
        }
        Err(e) => {
            eprintln!("[Schema] Step 1: FAILED to create channels table: {}", e);
            return Err(e);
        }
    }

    eprintln!("[Schema] Step 2: Creating streams table...");
    // Create sequence for streams table
    conn.execute("CREATE SEQUENCE IF NOT EXISTS streams_id_seq START 1", [])?;
    eprintln!("[Schema] streams sequence created");

    // streams テーブル: 配信基本情報
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS streams (
            id BIGINT PRIMARY KEY DEFAULT nextval('streams_id_seq'),
            channel_id BIGINT NOT NULL,
            stream_id TEXT NOT NULL,
            title TEXT,
            category TEXT,
            started_at TIMESTAMP NOT NULL,
            ended_at TIMESTAMP,
            FOREIGN KEY (channel_id) REFERENCES channels(id),
            UNIQUE(channel_id, stream_id)
        )
        "#,
        [],
    )?;
    eprintln!("[Schema] Step 2: streams table created");

    eprintln!("[Schema] Step 3: Creating stream_stats table...");
    // Create sequence for stream_stats table
    conn.execute(
        "CREATE SEQUENCE IF NOT EXISTS stream_stats_id_seq START 1",
        [],
    )?;
    eprintln!("[Schema] stream_stats sequence created");

    // stream_stats テーブル: 定期収集統計データ
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stream_stats (
            id BIGINT PRIMARY KEY DEFAULT nextval('stream_stats_id_seq'),
            stream_id BIGINT,
            collected_at TIMESTAMP NOT NULL,
            viewer_count INTEGER,
            chat_rate_1min INTEGER DEFAULT 0,
            twitch_user_id TEXT,
            channel_name TEXT,
            FOREIGN KEY (stream_id) REFERENCES streams(id)
        )
        "#,
        [],
    )?;
    eprintln!("[Schema] Step 3: stream_stats table created");

    eprintln!("[Schema] Step 4: Creating chat_messages table...");
    // Create sequence for chat_messages table
    conn.execute(
        "CREATE SEQUENCE IF NOT EXISTS chat_messages_id_seq START 1",
        [],
    )?;
    eprintln!("[Schema] chat_messages sequence created");

    // chat_messages テーブル: チャット全ログ
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS chat_messages (
            id BIGINT PRIMARY KEY DEFAULT nextval('chat_messages_id_seq'),
            stream_id BIGINT NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            platform TEXT NOT NULL,
            user_id TEXT,
            user_name TEXT NOT NULL,
            message TEXT NOT NULL,
            message_type TEXT DEFAULT 'normal',
            FOREIGN KEY (stream_id) REFERENCES streams(id)
        )
        "#,
        [],
    )?;
    eprintln!("[Schema] Step 4: chat_messages table created");

    eprintln!("[Schema] Step 4.1: Creating sql_templates table...");
    // Create sequence for sql_templates table
    conn.execute(
        "CREATE SEQUENCE IF NOT EXISTS sql_templates_id_seq START 1",
        [],
    )?;
    eprintln!("[Schema] sql_templates sequence created");

    // sql_templates テーブル: SQLクエリテンプレート
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS sql_templates (
            id BIGINT PRIMARY KEY DEFAULT nextval('sql_templates_id_seq'),
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            query TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    eprintln!("[Schema] Step 4.1: sql_templates table created");

    eprintln!("[Schema] Step 4.5: Running database migrations...");
    // 既存テーブルにフィールドを追加（マイグレーション）
    migrate_database_schema(conn)?;
    eprintln!("[Schema] Step 4.5: Migrations completed");

    eprintln!("[Schema] Step 5: Creating indexes...");
    // インデックス作成
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_streams_channel_id ON streams(channel_id)",
        [],
    )?;
    eprintln!("[Schema] Index 1 created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_streams_started_at ON streams(started_at)",
        [],
    )?;
    eprintln!("[Schema] Index 2 created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_stream_id ON stream_stats(stream_id)",
        [],
    )?;
    eprintln!("[Schema] Index 3 created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_collected_at ON stream_stats(collected_at)",
        [],
    )?;
    eprintln!("[Schema] Index 4 created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_stream_id ON chat_messages(stream_id)",
        [],
    )?;
    eprintln!("[Schema] Index 5 created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_timestamp ON chat_messages(timestamp)",
        [],
    )?;
    eprintln!("[Schema] Index 6 created");

    eprintln!("[Schema] All steps completed successfully");
    Ok(())
}

/// データベーススキーマのマイグレーションを行う関数
/// 既存のテーブルにフィールドを追加する
fn migrate_database_schema(conn: &Connection) -> Result<(), duckdb::Error> {
    // streamsテーブルにthumbnail_urlフィールドを追加
    let mut streams_has_thumbnail = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('streams') WHERE name = 'thumbnail_url'",
    )?;
    let streams_has_thumbnail_count: i64 = streams_has_thumbnail.query_row([], |row| row.get(0))?;

    if streams_has_thumbnail_count == 0 {
        // thumbnail_urlフィールドがない場合、ALTER TABLEで追加
        eprintln!("[Migration] Adding thumbnail_url column to streams table");
        conn.execute("ALTER TABLE streams ADD COLUMN thumbnail_url TEXT", [])?;
    }

    // stream_statsテーブルにcategoryフィールドを追加
    let mut stream_stats_has_category = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('stream_stats') WHERE name = 'category'",
    )?;
    let stream_stats_has_category_count: i64 =
        stream_stats_has_category.query_row([], |row| row.get(0))?;

    if stream_stats_has_category_count == 0 {
        eprintln!("[Migration] Adding category column to stream_stats table");
        conn.execute("ALTER TABLE stream_stats ADD COLUMN category TEXT", [])?;
    }

    // channelsテーブルにis_auto_discoveredフィールドを追加
    let mut channels_has_is_auto_discovered = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'is_auto_discovered'",
    )?;
    let channels_has_is_auto_discovered_count: i64 =
        channels_has_is_auto_discovered.query_row([], |row| row.get(0))?;

    if channels_has_is_auto_discovered_count == 0 {
        eprintln!("[Migration] Adding is_auto_discovered column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN is_auto_discovered BOOLEAN DEFAULT FALSE",
            [],
        )?;
    }

    // channelsテーブルにdiscovered_atフィールドを追加
    let mut channels_has_discovered_at = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'discovered_at'",
    )?;
    let channels_has_discovered_at_count: i64 =
        channels_has_discovered_at.query_row([], |row| row.get(0))?;

    if channels_has_discovered_at_count == 0 {
        eprintln!("[Migration] Adding discovered_at column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN discovered_at TIMESTAMP",
            [],
        )?;
    }

    // channelsテーブルにcurrent_viewer_countフィールドを追加
    let mut channels_has_current_viewer_count = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'current_viewer_count'",
    )?;
    let channels_has_current_viewer_count_count: i64 =
        channels_has_current_viewer_count.query_row([], |row| row.get(0))?;

    if channels_has_current_viewer_count_count == 0 {
        eprintln!("[Migration] Adding current_viewer_count column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN current_viewer_count INTEGER",
            [],
        )?;
    }

    // channelsテーブルにcurrent_categoryフィールドを追加
    let mut channels_has_current_category = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'current_category'",
    )?;
    let channels_has_current_category_count: i64 =
        channels_has_current_category.query_row([], |row| row.get(0))?;

    if channels_has_current_category_count == 0 {
        eprintln!("[Migration] Adding current_category column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN current_category TEXT", [])?;
    }

    // channelsテーブルにtwitch_user_idフィールドを追加（不変なTwitch user ID）
    let mut channels_has_twitch_user_id = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'twitch_user_id'",
    )?;
    let channels_has_twitch_user_id_count: i64 =
        channels_has_twitch_user_id.query_row([], |row| row.get(0))?;

    if channels_has_twitch_user_id_count == 0 {
        eprintln!("[Migration] Adding twitch_user_id column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN twitch_user_id BIGINT", [])?;
        // インデックスを作成（検索パフォーマンス向上のため）
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_channels_twitch_user_id ON channels(twitch_user_id)",
            [],
        )?;
        eprintln!("[Migration] Created index on twitch_user_id");
    }

    // stream_statsテーブルにtitleフィールドを追加
    let mut stream_stats_has_title = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('stream_stats') WHERE name = 'title'")?;
    let stream_stats_has_title_count: i64 =
        stream_stats_has_title.query_row([], |row| row.get(0))?;

    if stream_stats_has_title_count == 0 {
        eprintln!("[Migration] Adding title column to stream_stats table");
        conn.execute("ALTER TABLE stream_stats ADD COLUMN title TEXT", [])?;
    }

    // stream_statsテーブルにfollower_countフィールドを追加
    let mut stream_stats_has_follower_count = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('stream_stats') WHERE name = 'follower_count'",
    )?;
    let stream_stats_has_follower_count_count: i64 =
        stream_stats_has_follower_count.query_row([], |row| row.get(0))?;

    if stream_stats_has_follower_count_count == 0 {
        eprintln!("[Migration] Adding follower_count column to stream_stats table");
        conn.execute(
            "ALTER TABLE stream_stats ADD COLUMN follower_count INTEGER",
            [],
        )?;
    }

    Ok(())
}
