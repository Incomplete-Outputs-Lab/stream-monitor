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
            channel_id BIGINT,
            stream_id BIGINT,
            timestamp TIMESTAMP NOT NULL,
            platform TEXT NOT NULL,
            user_id TEXT,
            user_name TEXT NOT NULL,
            message TEXT NOT NULL,
            message_type TEXT DEFAULT 'normal'
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_channel_id ON chat_messages(channel_id)",
        [],
    )?;
    eprintln!("[Schema] Index 7 created");

    // 追加のパフォーマンス最適化インデックス
    eprintln!("[Schema] Creating additional performance optimization indexes...");

    // stream_stats テーブルの最適化インデックス
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_category ON stream_stats(category)",
        [],
    )?;
    eprintln!("[Schema] Index 8: stream_stats.category created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_channel_name ON stream_stats(channel_name)",
        [],
    )?;
    eprintln!("[Schema] Index 9: stream_stats.channel_name created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_channel_collected ON stream_stats(channel_name, collected_at)",
        [],
    )?;
    eprintln!("[Schema] Index 10: stream_stats(channel_name, collected_at) created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_stream_stats_category_collected ON stream_stats(category, collected_at)",
        [],
    )?;
    eprintln!("[Schema] Index 11: stream_stats(category, collected_at) created");

    // chat_messages テーブルの最適化インデックス
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_user_name ON chat_messages(user_name)",
        [],
    )?;
    eprintln!("[Schema] Index 12: chat_messages.user_name created");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_user_timestamp ON chat_messages(user_name, timestamp)",
        [],
    )?;
    eprintln!("[Schema] Index 13: chat_messages(user_name, timestamp) created");

    // streams テーブルの複合インデックス
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_streams_channel_started ON streams(channel_id, started_at)",
        [],
    )?;
    eprintln!("[Schema] Index 14: streams(channel_id, started_at) created");

    // channels テーブルの最適化インデックス
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_platform ON channels(platform)",
        [],
    )?;
    eprintln!("[Schema] Index 15: channels.platform created");

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

    // channelsテーブルにdisplay_nameフィールドを追加
    let mut channels_has_display_name = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'display_name'",
    )?;
    let channels_has_display_name_count: i64 =
        channels_has_display_name.query_row([], |row| row.get(0))?;

    if channels_has_display_name_count == 0 {
        eprintln!("[Migration] Adding display_name column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN display_name TEXT DEFAULT ''",
            [],
        )?;
        // 既存データのdisplay_nameをchannel_nameで初期化
        conn.execute(
            "UPDATE channels SET display_name = channel_name WHERE display_name = '' OR display_name IS NULL",
            [],
        )?;
    }

    // channelsテーブルにprofile_image_urlフィールドを追加
    let mut channels_has_profile_image_url = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'profile_image_url'",
    )?;
    let channels_has_profile_image_url_count: i64 =
        channels_has_profile_image_url.query_row([], |row| row.get(0))?;

    if channels_has_profile_image_url_count == 0 {
        eprintln!("[Migration] Adding profile_image_url column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN profile_image_url TEXT DEFAULT ''",
            [],
        )?;
    }

    // channelsテーブルにfollower_countフィールドを追加
    let mut channels_has_follower_count = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'follower_count'",
    )?;
    let channels_has_follower_count_count: i64 =
        channels_has_follower_count.query_row([], |row| row.get(0))?;

    if channels_has_follower_count_count == 0 {
        eprintln!("[Migration] Adding follower_count column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN follower_count INTEGER DEFAULT 0",
            [],
        )?;
    }

    // channelsテーブルにbroadcaster_typeフィールドを追加
    let mut channels_has_broadcaster_type = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'broadcaster_type'",
    )?;
    let channels_has_broadcaster_type_count: i64 =
        channels_has_broadcaster_type.query_row([], |row| row.get(0))?;

    if channels_has_broadcaster_type_count == 0 {
        eprintln!("[Migration] Adding broadcaster_type column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN broadcaster_type TEXT DEFAULT ''",
            [],
        )?;
    }

    // channelsテーブルにview_countフィールドを追加
    let mut channels_has_view_count = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('channels') WHERE name = 'view_count'")?;
    let channels_has_view_count_count: i64 =
        channels_has_view_count.query_row([], |row| row.get(0))?;

    if channels_has_view_count_count == 0 {
        eprintln!("[Migration] Adding view_count column to channels table");
        conn.execute(
            "ALTER TABLE channels ADD COLUMN view_count INTEGER DEFAULT 0",
            [],
        )?;
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
            "ALTER TABLE channels ADD COLUMN discovered_at TEXT DEFAULT ''",
            [],
        )?;
    }

    // 既存のNULL値を空文字列に更新
    eprintln!("[Migration] Updating NULL values in channels table to default values");
    conn.execute(
        "UPDATE channels SET 
            display_name = COALESCE(display_name, channel_name),
            profile_image_url = COALESCE(profile_image_url, ''),
            follower_count = COALESCE(follower_count, 0),
            broadcaster_type = COALESCE(broadcaster_type, ''),
            view_count = COALESCE(view_count, 0),
            is_auto_discovered = COALESCE(is_auto_discovered, false),
            discovered_at = COALESCE(discovered_at, '')
        WHERE display_name IS NULL 
            OR profile_image_url IS NULL 
            OR follower_count IS NULL 
            OR broadcaster_type IS NULL 
            OR view_count IS NULL 
            OR is_auto_discovered IS NULL 
            OR discovered_at IS NULL",
        [],
    )?;
    eprintln!("[Migration] NULL values updated successfully");

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

    // chat_messagesテーブルにchannel_idフィールドを追加
    let mut chat_messages_has_channel_id = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'channel_id'",
    )?;
    let chat_messages_has_channel_id_count: i64 =
        chat_messages_has_channel_id.query_row([], |row| row.get(0))?;

    if chat_messages_has_channel_id_count == 0 {
        eprintln!("[Migration] Adding channel_id column to chat_messages table");
        conn.execute("ALTER TABLE chat_messages ADD COLUMN channel_id BIGINT", [])?;
        // インデックスを作成
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_messages_channel_id ON chat_messages(channel_id)",
            [],
        )?;
        eprintln!("[Migration] Created index on chat_messages.channel_id");
    }

    // chat_messagesテーブルのstream_idをNULL可能にする
    // DuckDBはALTER COLUMNをサポートしていないため、テーブル構造を確認して再作成が必要か判断
    let mut chat_messages_stream_id_notnull = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'stream_id' AND \"notnull\" = 1",
    )?;
    let chat_messages_stream_id_notnull_count: i64 =
        chat_messages_stream_id_notnull.query_row([], |row| row.get(0))?;

    if chat_messages_stream_id_notnull_count > 0 {
        eprintln!("[Migration] Migrating chat_messages table to make stream_id nullable");

        // 既存データがあるかチェック
        let mut count_stmt = conn.prepare("SELECT COUNT(*) FROM chat_messages")?;
        let existing_count: i64 = count_stmt.query_row([], |row| row.get(0))?;

        if existing_count > 0 {
            eprintln!(
                "[Migration] Found {} existing chat messages, migrating data...",
                existing_count
            );
        }

        // 一時テーブルを作成（stream_idをNULL可能に）
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS chat_messages_new (
                id BIGINT PRIMARY KEY,
                channel_id BIGINT,
                stream_id BIGINT,
                timestamp TIMESTAMP NOT NULL,
                platform TEXT NOT NULL,
                user_id TEXT,
                user_name TEXT NOT NULL,
                message TEXT NOT NULL,
                message_type TEXT DEFAULT 'normal'
            )
            "#,
            [],
        )?;

        // 既存データをコピー
        if existing_count > 0 {
            conn.execute(
                r#"
                INSERT INTO chat_messages_new 
                SELECT id, channel_id, stream_id, timestamp, platform, user_id, user_name, message, message_type 
                FROM chat_messages
                "#,
                [],
            )?;
        }

        // 古いテーブルを削除
        conn.execute("DROP TABLE chat_messages", [])?;

        // 新しいテーブルをリネーム
        conn.execute("ALTER TABLE chat_messages_new RENAME TO chat_messages", [])?;

        // インデックスを再作成
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_messages_stream_id ON chat_messages(stream_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_messages_timestamp ON chat_messages(timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chat_messages_channel_id ON chat_messages(channel_id)",
            [],
        )?;

        eprintln!("[Migration] chat_messages table migration completed");
    }

    // chat_messagesテーブルにbadgesフィールドを追加（TEXT[]型）
    let mut chat_messages_badges_info =
        conn.prepare("SELECT type FROM pragma_table_info('chat_messages') WHERE name = 'badges'")?;
    let badges_type: Option<String> = chat_messages_badges_info
        .query_map([], |row| row.get(0))
        .ok()
        .and_then(|mut rows| rows.next())
        .and_then(|r| r.ok());

    match badges_type.as_deref() {
        None => {
            // カラムが存在しない場合は TEXT[] 型で追加
            eprintln!("[Migration] Adding badges column (TEXT[]) to chat_messages table");
            conn.execute("ALTER TABLE chat_messages ADD COLUMN badges TEXT[]", [])?;
        }
        Some("TEXT") => {
            // TEXT 型の場合は TEXT[] に変更（既存データを変換）
            eprintln!("[Migration] Converting badges column from TEXT to TEXT[]");

            // 一時カラムを追加
            conn.execute("ALTER TABLE chat_messages ADD COLUMN badges_new TEXT[]", [])?;

            // 既存のJSON文字列データを配列に変換（存在する場合）
            // 空文字列やNULLはそのまま
            conn.execute(
                r#"
                UPDATE chat_messages 
                SET badges_new = CASE 
                    WHEN badges IS NULL OR badges = '' THEN NULL
                    ELSE TRY_CAST(badges AS TEXT[])
                END
                "#,
                [],
            )?;

            // 古いカラムを削除
            conn.execute("ALTER TABLE chat_messages DROP COLUMN badges", [])?;

            // 新しいカラムをリネーム
            conn.execute(
                "ALTER TABLE chat_messages RENAME COLUMN badges_new TO badges",
                [],
            )?;

            eprintln!("[Migration] badges column conversion completed");
        }
        Some("TEXT[]") | Some("VARCHAR[]") => {
            // 既に TEXT[] または VARCHAR[] 型の場合は何もしない（DuckDBでは同等）
            eprintln!("[Migration] badges column already TEXT[]/VARCHAR[], skipping");
        }
        Some(other) => {
            eprintln!(
                "[Migration] Warning: badges column has unexpected type: {}",
                other
            );
        }
    }

    // chat_messagesテーブルにbadge_infoフィールドを追加（サブスク月数等の詳細情報）
    let mut chat_messages_has_badge_info = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'badge_info'",
    )?;
    let chat_messages_has_badge_info_count: i64 =
        chat_messages_has_badge_info.query_row([], |row| row.get(0))?;

    if chat_messages_has_badge_info_count == 0 {
        eprintln!("[Migration] Adding badge_info column to chat_messages table");
        conn.execute("ALTER TABLE chat_messages ADD COLUMN badge_info TEXT", [])?;
        eprintln!("[Migration] badge_info column added successfully");
    }

    // chat_messagesテーブルにdisplay_nameフィールドを追加
    let mut chat_messages_has_display_name = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'display_name'",
    )?;
    let chat_messages_has_display_name_count: i64 =
        chat_messages_has_display_name.query_row([], |row| row.get(0))?;

    if chat_messages_has_display_name_count == 0 {
        eprintln!("[Migration] Adding display_name column to chat_messages table");
        conn.execute("ALTER TABLE chat_messages ADD COLUMN display_name TEXT", [])?;
        eprintln!("[Migration] display_name column added successfully");
    }

    // 既存のchat_messagesのchannel_idをstreams経由で更新
    eprintln!("[Migration] Updating chat_messages.channel_id from streams table");
    let update_result = conn.execute(
        r#"
        UPDATE chat_messages cm
        SET channel_id = (
            SELECT s.channel_id 
            FROM streams s 
            WHERE s.id = cm.stream_id
        )
        WHERE cm.channel_id IS NULL 
            AND cm.stream_id IS NOT NULL
        "#,
        [],
    );

    match update_result {
        Ok(updated_rows) => {
            eprintln!(
                "[Migration] Updated {} chat_messages with channel_id from streams",
                updated_rows
            );
        }
        Err(e) => {
            eprintln!(
                "[Migration] Warning: Failed to update chat_messages.channel_id: {}",
                e
            );
        }
    }

    // chat_rate_1min列を削除（存在する場合）
    let mut stream_stats_has_chat_rate = conn.prepare(
        "SELECT COUNT(*) FROM pragma_table_info('stream_stats') WHERE name = 'chat_rate_1min'",
    )?;
    let stream_stats_has_chat_rate_count: i64 =
        stream_stats_has_chat_rate.query_row([], |row| row.get(0))?;

    if stream_stats_has_chat_rate_count > 0 {
        eprintln!("[Migration] Dropping chat_rate_1min column from stream_stats table");
        conn.execute("ALTER TABLE stream_stats DROP COLUMN chat_rate_1min", [])?;
        eprintln!("[Migration] chat_rate_1min column dropped successfully");
    }

    // chat_messagesテーブルに複合インデックス追加（パフォーマンス最適化）
    eprintln!("[Migration] Creating composite index on chat_messages(stream_id, timestamp)");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_stream_timestamp ON chat_messages(stream_id, timestamp)",
        [],
    )?;
    eprintln!("[Migration] Composite index created successfully");

    // chat_messagesテーブルのuser_idにインデックス追加（ユーザー識別子ベースの集計用）
    eprintln!("[Migration] Creating index on chat_messages.user_id");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chat_messages_user_id ON chat_messages(user_id)",
        [],
    )?;
    eprintln!("[Migration] chat_messages.user_id index created successfully");

    // game_categoriesテーブルを作成（カテゴリIDキャッシュ用）
    eprintln!("[Migration] Creating game_categories table if not exists");
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS game_categories (
            game_id TEXT PRIMARY KEY,
            game_name TEXT NOT NULL,
            box_art_url TEXT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;
    eprintln!("[Migration] game_categories table created");

    // stream_statsテーブルにgame_idフィールドを追加
    let mut stream_stats_has_game_id = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('stream_stats') WHERE name = 'game_id'")?;
    let stream_stats_has_game_id_count: i64 =
        stream_stats_has_game_id.query_row([], |row| row.get(0))?;

    if stream_stats_has_game_id_count == 0 {
        eprintln!("[Migration] Adding game_id column to stream_stats table");
        conn.execute("ALTER TABLE stream_stats ADD COLUMN game_id TEXT", [])?;
        // インデックスを作成（検索パフォーマンス向上のため）
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stream_stats_game_id ON stream_stats(game_id)",
            [],
        )?;
        eprintln!("[Migration] Created index on stream_stats.game_id");
    }

    eprintln!("[Migration] All migrations completed successfully");
    Ok(())
}
