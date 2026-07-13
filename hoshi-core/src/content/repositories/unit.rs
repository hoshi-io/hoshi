use sqlx::SqlitePool;

use crate::content::models::ContentUnit;
use crate::error::CoreResult;

pub struct UnitRepository;

impl UnitRepository {
    pub async fn upsert(pool: &SqlitePool, unit: &ContentUnit) -> CoreResult<()> {
        sqlx::query(
            r#"
        INSERT INTO content_units (
            cid, unit_number, type, title, description,
            thumbnail_url, released_at, duration, absolute_number, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(cid, type, unit_number) DO UPDATE SET
            title         = excluded.title,
            description   = excluded.description,
            thumbnail_url = excluded.thumbnail_url,
            released_at   = excluded.released_at,
            updated_at    = excluded.updated_at
        "#,
        )
            .bind(&unit.cid)
            .bind(unit.unit_number)
            .bind(&unit.content_type)
            .bind(&unit.title)
            .bind(&unit.description)
            .bind(&unit.thumbnail_url)
            .bind(&unit.released_at)
            .bind(unit.duration)
            .bind(unit.absolute_number)
            .bind(unit.created_at)
            .bind(unit.created_at)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn last_synced_at(pool: &SqlitePool, cid: &str) -> CoreResult<Option<i64>> {
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT MAX(updated_at) FROM content_units WHERE cid = ?"
        )
            .bind(cid)
            .fetch_optional(pool)
            .await?;

        Ok(row.and_then(|(v,)| v))
    }

    pub async fn get_by_cid(pool: &SqlitePool, cid: &str) -> CoreResult<Vec<ContentUnit>> {
        let rows: Vec<(Option<i64>, String, f64, String, Option<String>, Option<String>,
                       Option<String>, Option<String>, Option<i32>, Option<i32>, i64)> =
            sqlx::query_as(
                "SELECT id, cid, unit_number, type, title, description, thumbnail_url, \
                 released_at, duration, absolute_number, created_at \
                 FROM content_units WHERE cid = ? \
                 ORDER BY CASE WHEN type = 'episode' THEN 1 ELSE 2 END, unit_number ASC",
            )
                .bind(cid)
                .fetch_all(pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(id, cid, unit_number, content_type, title, description,
                      thumbnail_url, released_at, duration, absolute_number, created_at)| {
                ContentUnit {
                    id,
                    cid,
                    unit_number,
                    content_type,
                    title,
                    description,
                    thumbnail_url,
                    released_at,
                    duration,
                    absolute_number,
                    created_at,
                }
            })
            .collect())
    }

    pub async fn get_unit(
        pool: &SqlitePool,
        cid: &str,
        unit_number: f64,
        content_type: &str,
    ) -> CoreResult<Option<ContentUnit>> {
        let row: Option<(Option<i64>, String, f64, String, Option<String>, Option<String>,
                         Option<String>, Option<String>, Option<i32>, Option<i32>, i64)> =
            sqlx::query_as(
                "SELECT id, cid, unit_number, type, title, description, thumbnail_url, \
             released_at, duration, absolute_number, created_at \
             FROM content_units WHERE cid = ? AND unit_number = ? AND type = ? \
             LIMIT 1",
            )
                .bind(cid)
                .bind(unit_number)
                .bind(content_type)
                .fetch_optional(pool)
                .await?;

        Ok(row.map(|(id, cid, unit_number, content_type, title, description,
                        thumbnail_url, released_at, duration, absolute_number, created_at)| {
            ContentUnit {
                id, cid, unit_number, content_type, title, description,
                thumbnail_url, released_at, duration, absolute_number, created_at,
            }
        }))
    }
}