use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::CoreResult;
use crate::home::types::SequelCard;

pub struct HomeRepository;

impl HomeRepository {
    pub async fn get_layout(pool: &SqlitePool, user_id: i32) -> CoreResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT layout FROM UserHomeLayout WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
        Ok(row.map(|(l,)| l))
    }

    pub async fn set_layout(pool: &SqlitePool, user_id: i32, layout: &str) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO UserHomeLayout (user_id, layout, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET layout = excluded.layout, updated_at = excluded.updated_at",
        )
            .bind(user_id)
            .bind(layout)
            .bind(Utc::now().timestamp())
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete_layout(pool: &SqlitePool, user_id: i32) -> CoreResult<()> {
        sqlx::query("DELETE FROM UserHomeLayout WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Sequels of finished entries whose target isn't in the user's list, newest finished first.
    pub async fn unstarted_sequels(
        pool: &SqlitePool,
        user_id: i32,
        content_type: &str,
        include_nsfw: bool,
        limit: i64,
    ) -> CoreResult<Vec<SequelCard>> {
        let rows = sqlx::query_as::<_, SequelCard>(
            r#"
            SELECT COALESCE(r.target_cid, tm.cid) AS target_cid,
                   r.target_tracker_name, r.target_tracker_id,
                   r.target_title, r.target_cover_image
              FROM ListEntry le
              JOIN content c ON c.cid = le.cid
              JOIN content_relations r ON r.source_cid = le.cid AND r.relation_type = 'sequel'
              LEFT JOIN tracker_mappings tm
                     ON tm.tracker_name = r.target_tracker_name AND tm.tracker_id = r.target_tracker_id
             WHERE le.user_id = ?
               AND le.status IN ('COMPLETED', 'REPEATING')
               AND c.type = ?
               AND (? OR c.nsfw = 0)
               AND NOT EXISTS (
                   SELECT 1 FROM ListEntry l2
                    WHERE l2.user_id = le.user_id AND l2.cid = COALESCE(r.target_cid, tm.cid)
               )
             GROUP BY r.target_tracker_name, r.target_tracker_id
             ORDER BY MAX(le.updated_at) DESC
             LIMIT ?
            "#,
        )
            .bind(user_id)
            .bind(content_type)
            .bind(include_nsfw)
            .bind(limit)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }
}
