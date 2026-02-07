-- デバッグ用クエリ: データ確認
-- stream_statsのchannel_nameの一覧
SELECT DISTINCT ss.channel_name, COUNT(*) as count
FROM stream_stats ss
GROUP BY ss.channel_name
ORDER BY count DESC
LIMIT 10;

-- channelsテーブルの内容
SELECT id, channel_id, channel_name, platform
FROM channels
ORDER BY id
LIMIT 10;

-- マッチング確認
SELECT
    c.id,
    c.channel_id,
    c.channel_name as c_channel_name,
    (SELECT COUNT(DISTINCT ss.channel_name)
     FROM stream_stats ss
     WHERE ss.channel_name = c.channel_name) as match_count
FROM channels c;
