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
            stream_id BIGINT NOT NULL,
            collected_at TIMESTAMP NOT NULL,
            viewer_count INTEGER,
            chat_rate_1min INTEGER DEFAULT 0,
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

    // channelsテーブルにdisplay_nameフィールドを追加
    let mut channels_has_display_name = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'display_name'",
    )?;
    let channels_has_display_name_count: i64 =
        channels_has_display_name.query_row([], |row| row.get(0))?;

    if channels_has_display_name_count == 0 {
        // display_nameフィールドがない場合、ALTER TABLEで追加
        eprintln!("[Migration] Adding display_name column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN display_name TEXT", [])?;
    }

    // channelsテーブルにprofile_image_urlフィールドを追加
    let mut channels_has_profile_image = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'profile_image_url'",
    )?;
    let channels_has_profile_image_count: i64 =
        channels_has_profile_image.query_row([], |row| row.get(0))?;

    if channels_has_profile_image_count == 0 {
        // profile_image_urlフィールドがない場合、ALTER TABLEで追加
        eprintln!("[Migration] Adding profile_image_url column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN profile_image_url TEXT", [])?;
    }

    // channelsテーブルにfollower_countフィールドを追加
    let mut channels_has_follower_count = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'follower_count'",
    )?;
    let channels_has_follower_count_count: i64 =
        channels_has_follower_count.query_row([], |row| row.get(0))?;

    if channels_has_follower_count_count == 0 {
        eprintln!("[Migration] Adding follower_count column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN follower_count INTEGER", [])?;
    }

    // channelsテーブルにbroadcaster_typeフィールドを追加
    let mut channels_has_broadcaster_type = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'broadcaster_type'",
    )?;
    let channels_has_broadcaster_type_count: i64 =
        channels_has_broadcaster_type.query_row([], |row| row.get(0))?;

    if channels_has_broadcaster_type_count == 0 {
        eprintln!("[Migration] Adding broadcaster_type column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN broadcaster_type TEXT", [])?;
    }

    // channelsテーブルにview_countフィールドを追加
    let mut channels_has_view_count = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'view_count'",
    )?;
    let channels_has_view_count_count: i64 =
        channels_has_view_count.query_row([], |row| row.get(0))?;

    if channels_has_view_count_count == 0 {
        eprintln!("[Migration] Adding view_count column to channels table");
        conn.execute("ALTER TABLE channels ADD COLUMN view_count INTEGER", [])?;
    }

    Ok(())
}
