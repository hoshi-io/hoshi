use sqlx::SqlitePool;

use crate::content::models::{Relation, RelationType};
use crate::error::CoreResult;

pub struct RelationRepository;

impl RelationRepository {
    pub async fn get_by_source(pool: &SqlitePool, source_cid: &str) -> CoreResult<Vec<Relation>> {
        let rows: Vec<(i64, String, Option<String>, String, String, String, String, Option<String>, String, i64)> = sqlx::query_as(
            "SELECT id, source_cid, target_cid, target_tracker_name, target_tracker_id, \
                    relation_type, target_title, target_cover_image, source_name, created_at \
             FROM content_relations WHERE source_cid = ?",
        )
            .bind(source_cid)
            .fetch_all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|(id, source_cid, target_cid, target_tracker_name, target_tracker_id,
                      type_raw, target_title, target_cover_image, source_name, created_at)| {
                let relation_type = serde_json::from_str(&format!("\"{}\"", type_raw))
                    .unwrap_or(RelationType::Alternative);
                Relation {
                    id: Some(id),
                    source_cid,
                    target_cid,
                    target_tracker_name,
                    target_tracker_id,
                    relation_type,
                    target_title,
                    target_cover_image,
                    source_name,
                    created_at,
                }
            })
            .collect())
    }

    pub async fn upsert(pool: &SqlitePool, relation: &Relation) -> CoreResult<()> {
        sqlx::query(
            r#"
                INSERT INTO content_relations (
                    source_cid, target_cid, target_tracker_name, target_tracker_id,
                    relation_type, target_title, target_cover_image, source_name, created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(source_cid, target_tracker_name, target_tracker_id, relation_type)
                DO UPDATE SET
                    target_cid = excluded.target_cid,
                    target_title = excluded.target_title,
                    target_cover_image = excluded.target_cover_image
            "#,
        )
            .bind(&relation.source_cid)
            .bind(&relation.target_cid)
            .bind(&relation.target_tracker_name)
            .bind(&relation.target_tracker_id)
            .bind(relation.relation_type.as_str())
            .bind(&relation.target_title)
            .bind(&relation.target_cover_image)
            .bind(&relation.source_name)
            .bind(relation.created_at)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn backfill_target_cid(
        pool: &SqlitePool, tracker_name: &str, tracker_id: &str, cid: &str,
    ) -> CoreResult<()> {
        sqlx::query(
            "UPDATE content_relations SET target_cid = ? \
             WHERE target_tracker_name = ? AND target_tracker_id = ? AND target_cid IS NULL",
        )
            .bind(cid).bind(tracker_name).bind(tracker_id)
            .execute(pool).await?;
        Ok(())
    }
}