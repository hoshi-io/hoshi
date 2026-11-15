use chrono::Utc;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::CoreResult;
use crate::torrent::handle::magnet_key;

/// A torrent that played an episode, enough to start it again without a search.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TorrentChoice {
    pub torrent_id: String,
    pub title: String,
    pub magnet: String,
    pub is_pack: bool,
    /// Saved for this exact episode (otherwise it's a pack saved for another one).
    #[sqlx(default)]
    pub exact: bool,
}

pub struct TorrentChoiceRepository;

impl TorrentChoiceRepository {
    /// This episode's own choice, else the most recent pack used for the show on this source.
    pub async fn find(pool: &SqlitePool, user_id: i32, cid: &str, ext: &str, episode: u32) -> CoreResult<Option<TorrentChoice>> {
        let row = sqlx::query_as::<_, TorrentChoice>(
            "SELECT torrent_id, title, magnet, is_pack, episode = ? AS exact
             FROM torrent_choices
             WHERE user_id = ? AND cid = ? AND extension_id = ? AND (episode = ? OR is_pack = 1)
             ORDER BY episode = ? DESC, updated_at DESC
             LIMIT 1",
        )
            .bind(episode)
            .bind(user_id)
            .bind(cid)
            .bind(ext)
            .bind(episode)
            .bind(episode)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn save(
        pool: &SqlitePool,
        user_id: i32,
        cid: &str,
        ext: &str,
        episode: u32,
        torrent_id: &str,
        title: &str,
        magnet: &str,
        is_pack: bool,
    ) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO torrent_choices (user_id, cid, extension_id, episode, torrent_id, title, magnet, info_hash, is_pack, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, cid, extension_id, episode) DO UPDATE SET
                torrent_id = excluded.torrent_id, title = excluded.title, magnet = excluded.magnet,
                info_hash = excluded.info_hash, is_pack = excluded.is_pack, updated_at = excluded.updated_at",
        )
            .bind(user_id)
            .bind(cid)
            .bind(ext)
            .bind(episode)
            .bind(torrent_id)
            .bind(title)
            .bind(magnet)
            .bind(magnet_key(magnet).unwrap_or_default())
            .bind(is_pack)
            .bind(Utc::now().timestamp())
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl TorrentChoiceRepository {
    /// `(info_hash, cid, episode)` for every saved choice of this user.
    pub async fn usages(pool: &SqlitePool, user_id: i32) -> CoreResult<Vec<(String, String, u32)>> {
        let rows = sqlx::query_as::<_, (String, String, u32)>(
            "SELECT info_hash, cid, episode FROM torrent_choices WHERE user_id = ? ORDER BY episode",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }
}
