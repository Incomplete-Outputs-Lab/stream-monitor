/// チャンネルレポジトリ
///
/// チャンネルテーブルへのアクセスを抽象化
use crate::database::models::Channel;
use duckdb::Connection;

pub struct ChannelRepository;

/// チャンネル作成リクエスト
pub struct CreateChannelParams {
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
    pub poll_interval: i32,
    pub twitch_user_id: Option<i64>,
}

impl ChannelRepository {
    /// IDでチャンネルを取得
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare(
            "SELECT 
                id, 
                platform, 
                channel_id, 
                channel_name, 
                COALESCE(display_name, channel_name) as display_name, 
                COALESCE(profile_image_url, '') as profile_image_url, 
                enabled, 
                poll_interval, 
                COALESCE(follower_count, 0) as follower_count, 
                COALESCE(broadcaster_type, '') as broadcaster_type, 
                COALESCE(view_count, 0) as view_count, 
                COALESCE(is_auto_discovered, false) as is_auto_discovered, 
                COALESCE(discovered_at, '') as discovered_at, 
                twitch_user_id, 
                CAST(created_at AS VARCHAR) as created_at, 
                CAST(updated_at AS VARCHAR) as updated_at 
            FROM channels 
            WHERE id = ?",
        )?;

        let id_str = id.to_string();
        let mut rows = stmt.query_map([id_str.as_str()], |row| {
            Ok(Channel {
                id: Some(row.get(0)?),
                platform: row.get(1)?,
                channel_id: row.get(2)?,
                channel_name: row.get(3)?,
                display_name: row.get(4)?,
                profile_image_url: row.get(5)?,
                enabled: row.get(6)?,
                poll_interval: row.get(7)?,
                follower_count: row.get(8)?,
                broadcaster_type: row.get(9)?,
                view_count: row.get(10)?,
                is_auto_discovered: row.get(11)?,
                discovered_at: row.get(12)?,
                twitch_user_id: row.get(13)?,
                created_at: Some(row.get(14)?),
                updated_at: Some(row.get(15)?),
            })
        })?;

        rows.next().transpose()
    }

    /// 全チャンネルを取得（作成日時降順）
    pub fn list_all(conn: &Connection) -> Result<Vec<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare(
            "SELECT 
                id, 
                platform, 
                channel_id, 
                channel_name, 
                COALESCE(display_name, channel_name) as display_name, 
                COALESCE(profile_image_url, '') as profile_image_url, 
                enabled, 
                poll_interval, 
                COALESCE(follower_count, 0) as follower_count, 
                COALESCE(broadcaster_type, '') as broadcaster_type, 
                COALESCE(view_count, 0) as view_count, 
                COALESCE(is_auto_discovered, false) as is_auto_discovered, 
                COALESCE(discovered_at, '') as discovered_at, 
                twitch_user_id, 
                CAST(created_at AS VARCHAR) as created_at, 
                CAST(updated_at AS VARCHAR) as updated_at 
            FROM channels 
            ORDER BY created_at DESC",
        )?;

        let channels = stmt
            .query_map([], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: row.get(4)?,
                    profile_image_url: row.get(5)?,
                    enabled: row.get(6)?,
                    poll_interval: row.get(7)?,
                    follower_count: row.get(8)?,
                    broadcaster_type: row.get(9)?,
                    view_count: row.get(10)?,
                    is_auto_discovered: row.get(11)?,
                    discovered_at: row.get(12)?,
                    twitch_user_id: row.get(13)?,
                    created_at: Some(row.get(14)?),
                    updated_at: Some(row.get(15)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// プラットフォームでチャンネルをフィルタ
    #[allow(dead_code)]
    pub fn list_by_platform(
        conn: &Connection,
        platform: &str,
    ) -> Result<Vec<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare(
            "SELECT 
                id, 
                platform, 
                channel_id, 
                channel_name, 
                COALESCE(display_name, channel_name) as display_name, 
                COALESCE(profile_image_url, '') as profile_image_url, 
                enabled, 
                poll_interval, 
                COALESCE(follower_count, 0) as follower_count, 
                COALESCE(broadcaster_type, '') as broadcaster_type, 
                COALESCE(view_count, 0) as view_count, 
                COALESCE(is_auto_discovered, false) as is_auto_discovered, 
                COALESCE(discovered_at, '') as discovered_at, 
                twitch_user_id, 
                CAST(created_at AS VARCHAR) as created_at, 
                CAST(updated_at AS VARCHAR) as updated_at 
            FROM channels 
            WHERE platform = ?
            ORDER BY created_at DESC",
        )?;

        let channels = stmt
            .query_map([platform], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: row.get(4)?,
                    profile_image_url: row.get(5)?,
                    enabled: row.get(6)?,
                    poll_interval: row.get(7)?,
                    follower_count: row.get(8)?,
                    broadcaster_type: row.get(9)?,
                    view_count: row.get(10)?,
                    is_auto_discovered: row.get(11)?,
                    discovered_at: row.get(12)?,
                    twitch_user_id: row.get(13)?,
                    created_at: Some(row.get(14)?),
                    updated_at: Some(row.get(15)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// 有効なチャンネルのみを取得
    #[allow(dead_code)]
    pub fn list_enabled(conn: &Connection) -> Result<Vec<Channel>, duckdb::Error> {
        let mut stmt = conn.prepare(
            "SELECT 
                id, 
                platform, 
                channel_id, 
                channel_name, 
                COALESCE(display_name, channel_name) as display_name, 
                COALESCE(profile_image_url, '') as profile_image_url, 
                enabled, 
                poll_interval, 
                COALESCE(follower_count, 0) as follower_count, 
                COALESCE(broadcaster_type, '') as broadcaster_type, 
                COALESCE(view_count, 0) as view_count, 
                COALESCE(is_auto_discovered, false) as is_auto_discovered, 
                COALESCE(discovered_at, '') as discovered_at, 
                twitch_user_id, 
                CAST(created_at AS VARCHAR) as created_at, 
                CAST(updated_at AS VARCHAR) as updated_at 
            FROM channels 
            WHERE enabled = true
            ORDER BY created_at DESC",
        )?;

        let channels = stmt
            .query_map([], |row| {
                Ok(Channel {
                    id: Some(row.get(0)?),
                    platform: row.get(1)?,
                    channel_id: row.get(2)?,
                    channel_name: row.get(3)?,
                    display_name: row.get(4)?,
                    profile_image_url: row.get(5)?,
                    enabled: row.get(6)?,
                    poll_interval: row.get(7)?,
                    follower_count: row.get(8)?,
                    broadcaster_type: row.get(9)?,
                    view_count: row.get(10)?,
                    is_auto_discovered: row.get(11)?,
                    discovered_at: row.get(12)?,
                    twitch_user_id: row.get(13)?,
                    created_at: Some(row.get(14)?),
                    updated_at: Some(row.get(15)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// チャンネルを作成（IDを返す）
    pub fn create(conn: &Connection, params: CreateChannelParams) -> Result<i64, duckdb::Error> {
        let channel_id: i64 = conn.query_row(
            "INSERT INTO channels (platform, channel_id, channel_name, poll_interval, twitch_user_id) 
             VALUES (?, ?, ?, ?, ?) RETURNING id",
            duckdb::params![
                &params.platform,
                &params.channel_id,
                &params.channel_name,
                params.poll_interval,
                params.twitch_user_id,
            ],
            |row| row.get(0),
        )?;

        Ok(channel_id)
    }

    /// チャンネルを削除
    pub fn delete(conn: &Connection, id: i64) -> Result<(), duckdb::Error> {
        let id_str = id.to_string();
        conn.execute("DELETE FROM channels WHERE id = ?", [id_str.as_str()])?;
        Ok(())
    }

    /// チャンネルの有効状態を更新
    pub fn update_enabled(conn: &Connection, id: i64, enabled: bool) -> Result<(), duckdb::Error> {
        let enabled_str = enabled.to_string();
        let id_str = id.to_string();
        conn.execute(
            "UPDATE channels SET enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            [enabled_str.as_str(), id_str.as_str()],
        )?;
        Ok(())
    }

    /// Twitchの全ユーザーIDを取得（自動発見用）
    pub fn get_all_twitch_user_ids(conn: &Connection) -> Result<Vec<i64>, duckdb::Error> {
        let mut stmt = conn.prepare(
            "SELECT twitch_user_id FROM channels WHERE platform = 'twitch' AND twitch_user_id IS NOT NULL"
        )?;

        let user_ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(user_ids)
    }

    /// チャンネルIDとプラットフォームで存在確認
    pub fn exists(
        conn: &Connection,
        platform: &str,
        channel_id: &str,
    ) -> Result<bool, duckdb::Error> {
        let mut stmt =
            conn.prepare("SELECT COUNT(*) FROM channels WHERE platform = ? AND channel_id = ?")?;
        let count: i64 = stmt.query_row([platform, channel_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// 自動発見フラグとtwitch_user_idを更新
    pub fn update_auto_discovered(
        conn: &Connection,
        platform: &str,
        channel_id: &str,
        is_auto_discovered: bool,
        twitch_user_id: Option<i64>,
    ) -> Result<(), duckdb::Error> {
        if is_auto_discovered {
            // 自動発見時は discovered_at を現在時刻に設定
            conn.execute(
                "UPDATE channels SET is_auto_discovered = ?, discovered_at = CURRENT_TIMESTAMP, twitch_user_id = ? WHERE platform = ? AND channel_id = ?",
                duckdb::params![is_auto_discovered, twitch_user_id, platform, channel_id],
            )?;
        } else {
            // 昇格時は is_auto_discovered を false、discovered_at を NULL に設定
            conn.execute(
                "UPDATE channels SET is_auto_discovered = false, discovered_at = NULL, twitch_user_id = ? WHERE platform = ? AND channel_id = ?",
                duckdb::params![twitch_user_id, platform, channel_id],
            )?;
        }
        Ok(())
    }

    /// チャンネル情報を取得（エクスポート用）
    pub fn get_channel_info(conn: &Connection, id: i64) -> Result<(String, i64), duckdb::Error> {
        conn.query_row(
            "SELECT channel_id, (SELECT COUNT(*) FROM streams WHERE channel_id = c.id) as stream_count FROM channels c WHERE id = ?",
            [id.to_string()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        )
    }
}
