use std::collections::HashMap;

use sqlx::{Sqlite, SqlitePool};

use crate::content::models::ContentUnit;
use crate::content::repositories::{placeholders, IN_CHUNK};
use crate::error::CoreResult;

type UnitRow = (Option<i64>, String, f64, String, Option<String>, Option<String>,
                Option<String>, Option<String>, Option<i32>, Option<i32>, i64);

pub struct UnitRepository;

impl UnitRepository {

    pub async fn upsert<'e, E>(executor: E, unit: &ContentUnit) -> CoreResult<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
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
            .execute(executor)
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
        Ok(Self::get_by_cids(pool, &[cid]).await?.remove(cid).unwrap_or_default())
    }

    /// Per cid: episodes first, then by number (same order as `get_by_cid`).
    pub async fn get_by_cids(pool: &SqlitePool, cids: &[&str]) -> CoreResult<HashMap<String, Vec<ContentUnit>>> {
        let mut out: HashMap<String, Vec<ContentUnit>> = HashMap::new();
        for chunk in cids.chunks(IN_CHUNK) {
            let sql = format!(
                "SELECT id, cid, unit_number, type, title, description, thumbnail_url, \
                 released_at, duration, absolute_number, created_at \
                 FROM content_units WHERE cid IN ({}) \
                 ORDER BY cid, CASE WHEN type = 'episode' THEN 1 ELSE 2 END, unit_number ASC",
                placeholders(chunk.len()),
            );
            let mut q = sqlx::query_as::<_, UnitRow>(&sql);
            for cid in chunk {
                q = q.bind(*cid);
            }
            for (id, cid, unit_number, content_type, title, description,
                 thumbnail_url, released_at, duration, absolute_number, created_at) in q.fetch_all(pool).await? {
                out.entry(cid.clone()).or_default().push(ContentUnit {
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
                });
            }
        }
        Ok(out)
    }
}