use chrono::Utc;
use sqlx::SqlitePool;

use crate::content::models::ExtensionSource;
use crate::error::CoreResult;

pub struct ExtensionRepository;

impl ExtensionRepository {
    
    pub async fn add_source(pool: &SqlitePool, source: &ExtensionSource) -> CoreResult<i64> {
        let now = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO extension_sources
                (cid, extension_name, extension_id, nsfw, language, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(extension_name, extension_id) DO UPDATE SET
                nsfw       = excluded.nsfw,
                language   = excluded.language,
                updated_at = excluded.updated_at
            "#,
        )
            .bind(&source.cid)
            .bind(&source.extension_name)
            .bind(&source.extension_id)
            .bind(source.nsfw)
            .bind(&source.language)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn update_source(pool: &SqlitePool, id: i64, ext_id: &str) -> CoreResult<()> {
        let now = Utc::now().timestamp();

        sqlx::query(
            "UPDATE extension_sources SET extension_id = ?, updated_at = ? WHERE id = ?",
        )
            .bind(ext_id)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn find_cid_by_extension(
        pool: &SqlitePool,
        extension_name: &str,
        extension_id: &str,
    ) -> CoreResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT cid FROM extension_sources WHERE extension_name = ? AND extension_id = ?",
        )
            .bind(extension_name)
            .bind(extension_id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(cid,)| cid))
    }

    pub async fn get_by_cid(pool: &SqlitePool, cid: &str) -> CoreResult<Vec<ExtensionSource>> {
        let rows: Vec<(i64, String, String, String, i32, Option<String>, i64, i64)> =
            sqlx::query_as(
                "SELECT id, cid, extension_name, extension_id, nsfw, language, created_at, updated_at \
                 FROM extension_sources WHERE cid = ?",
            )
                .bind(cid)
                .fetch_all(pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(id, cid, extension_name, extension_id, nsfw, language, created_at, updated_at)| {
                ExtensionSource {
                    id: Some(id),
                    cid,
                    extension_name,
                    extension_id,
                    nsfw: nsfw == 1,
                    language,
                    created_at,
                    updated_at,
                }
            })
            .collect())
    }

    pub async fn find_mapping_id(
        pool: &SqlitePool,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Option<i64>> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM extension_sources WHERE cid = ? AND extension_name = ?",
        )
            .bind(cid)
            .bind(ext_name)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(id,)| id))
    }

    pub async fn get_extension_id_and_type(
        pool: &SqlitePool,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Option<(String, String)>> {
        let row: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT c.type, es.extension_id
            FROM content c
            JOIN extension_sources es ON c.cid = es.cid
            WHERE c.cid = ? AND es.extension_name = ?
            "#,
        )
            .bind(cid)
            .bind(ext_name)
            .fetch_optional(pool)
            .await?;

        Ok(row)
    }

    pub async fn has_metadata(
        pool: &SqlitePool,
        cid: &str,
        source_name: &str,
    ) -> CoreResult<bool> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM metadata WHERE cid = ? AND source_name = ?",
        )
            .bind(cid)
            .bind(source_name)
            .fetch_one(pool)
            .await?;

        Ok(row.0 > 0)
    }
}