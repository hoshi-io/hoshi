use sqlx::SqlitePool;
use crate::error::CoreResult;
use crate::progress::types::{
    AnimeProgress, ChapterProgress,
    UpdateAnimeProgressBody, UpdateChapterProgressBody,
};

pub struct ProgressRepository;

impl ProgressRepository {
    pub async fn upsert_anime(
        pool: &SqlitePool,
        user_id: i32,
        body: &UpdateAnimeProgressBody,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO AnimeProgress
                (user_id, cid, episode, timestamp_seconds, episode_duration_seconds, completed, last_accessed)
            VALUES
                (?, ?, ?, ?, ?, ?, unixepoch())
            ON CONFLICT(user_id, cid, episode) DO UPDATE SET
                timestamp_seconds        = excluded.timestamp_seconds,
                episode_duration_seconds = excluded.episode_duration_seconds,
                completed                = excluded.completed,
                last_accessed            = unixepoch()
            "#,
        )
            .bind(user_id)
            .bind(&body.cid)
            .bind(body.episode)
            .bind(body.timestamp_seconds)
            .bind(body.episode_duration_seconds)
            .bind(body.completed.unwrap_or(false))
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn upsert_chapter(
        pool: &SqlitePool,
        user_id: i32,
        body: &UpdateChapterProgressBody,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ChapterProgress
                (user_id, cid, chapter, completed, last_accessed)
            VALUES
                (?, ?, ?, ?, unixepoch())
            ON CONFLICT(user_id, cid, chapter) DO UPDATE SET
                completed     = excluded.completed,
                last_accessed = unixepoch()
            "#,
        )
            .bind(user_id)
            .bind(&body.cid)
            .bind(body.chapter)
            .bind(body.completed.unwrap_or(false))
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_latest_anime_per_cid(
        pool: &SqlitePool,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<Vec<AnimeProgress>> {
        let rows = sqlx::query_as(
            r#"
            SELECT ap.*
            FROM AnimeProgress ap
            INNER JOIN (
                SELECT cid, MAX(last_accessed) AS max_accessed
                FROM AnimeProgress
                WHERE user_id = ?
                GROUP BY cid
            ) latest ON ap.cid = latest.cid AND ap.last_accessed = latest.max_accessed
            WHERE ap.user_id = ?
            ORDER BY ap.last_accessed DESC
            LIMIT ?
            "#,
        )
            .bind(user_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    pub async fn get_latest_chapter_per_cid(
        pool: &SqlitePool,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<Vec<ChapterProgress>> {
        let rows = sqlx::query_as(
            r#"
            SELECT cp.*
            FROM ChapterProgress cp
            INNER JOIN (
                SELECT cid, MAX(last_accessed) AS max_accessed
                FROM ChapterProgress
                WHERE user_id = ? AND completed = 0
                GROUP BY cid
            ) latest ON cp.cid = latest.cid AND cp.last_accessed = latest.max_accessed
            WHERE cp.user_id = ?
            ORDER BY cp.last_accessed DESC
            LIMIT ?
            "#,
        )
            .bind(user_id)
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }

    pub async fn get_progress_for_cid(
        pool: &SqlitePool,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<(Vec<AnimeProgress>, Vec<ChapterProgress>)> {
        let anime = sqlx::query_as(
            "SELECT * FROM AnimeProgress WHERE user_id = ? AND cid = ? ORDER BY episode ASC",
        )
            .bind(user_id)
            .bind(cid)
            .fetch_all(pool)
            .await?;

        let chapters = sqlx::query_as(
            "SELECT * FROM ChapterProgress WHERE user_id = ? AND cid = ? ORDER BY chapter ASC",
        )
            .bind(user_id)
            .bind(cid)
            .fetch_all(pool)
            .await?;

        Ok((anime, chapters))
    }
}