use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::content::models::{
    Content, ContentType,
    EpisodeData, FullContent, Metadata,
};
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::repositories::relations::RelationRepository;
use crate::content::repositories::unit::UnitRepository;
use crate::content::types::SearchResult;
use crate::content::utils::{normalize_title, similarity};
use crate::error::{CoreError, CoreResult};
use crate::tracker::repository::TrackerRepository;

pub struct ContentRepository;

impl ContentRepository {
    pub async fn create_with_type(
        pool: &SqlitePool,
        content_type: &ContentType,
        nsfw: bool,
        meta: Metadata,
    ) -> CoreResult<String> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO content (cid, type, nsfw, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(cid) DO NOTHING
            "#,
        )
            .bind(&meta.cid)
            .bind(content_type.as_str())
            .bind(nsfw)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

        Self::upsert_metadata(pool, &meta).await?;
        Ok(meta.cid)
    }

    pub async fn upsert_metadata(pool: &SqlitePool, meta: &Metadata) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO metadata (
                cid, source_name, source_id, subtype, title, alt_titles, title_i18n, synopsis,
                cover_image, banner_image, eps_or_chapters, status, genres,
                release_date, end_date, rating, trailer_url, characters, studio,
                staff, external_ids, episode_duration, created_at, updated_at
            ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
            ON CONFLICT(cid, source_name) DO UPDATE SET
                source_name     = excluded.source_name,
                source_id       = excluded.source_id,
                subtype         = excluded.subtype,
                title           = excluded.title,
                alt_titles      = excluded.alt_titles,
                title_i18n      = CASE
                                    WHEN excluded.title_i18n != '{}' THEN excluded.title_i18n
                                    ELSE metadata.title_i18n
                                  END,
                synopsis        = COALESCE(excluded.synopsis, metadata.synopsis),
                cover_image     = COALESCE(excluded.cover_image, metadata.cover_image),
                banner_image    = COALESCE(excluded.banner_image, metadata.banner_image),
                eps_or_chapters = excluded.eps_or_chapters,
                status          = excluded.status,
                genres          = excluded.genres,
                release_date    = excluded.release_date,
                end_date        = excluded.end_date,
                rating          = COALESCE(excluded.rating, metadata.rating),
                trailer_url     = COALESCE(excluded.trailer_url, metadata.trailer_url),
                characters      = excluded.characters,
                studio          = excluded.studio,
                staff           = excluded.staff,
                external_ids    = excluded.external_ids,
                episode_duration = COALESCE(excluded.episode_duration, metadata.episode_duration),
                updated_at      = excluded.updated_at
            "#,
        )
            .bind(&meta.cid)
            .bind(&meta.source_name)
            .bind(&meta.source_id)
            .bind(&meta.subtype)
            .bind(&meta.title)
            .bind(serde_json::to_string(&meta.alt_titles)?)
            .bind(serde_json::to_string(&meta.title_i18n)?)
            .bind(&meta.synopsis)
            .bind(&meta.cover_image)
            .bind(&meta.banner_image)
            .bind(serde_json::to_string(&meta.eps_or_chapters)?)
            .bind(meta.status.as_ref().map(|s| serde_json::to_string(s).unwrap()))
            .bind(serde_json::to_string(&meta.genres)?)
            .bind(&meta.release_date)
            .bind(&meta.end_date)
            .bind(meta.rating)
            .bind(&meta.trailer_url)
            .bind(serde_json::to_string(&meta.characters)?)
            .bind(&meta.studio)
            .bind(serde_json::to_string(&meta.staff)?)
            .bind(meta.external_ids.to_string())
            .bind(meta.episode_duration)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_content_by_cid(
        pool: &SqlitePool,
        cid: &str,
    ) -> CoreResult<Option<Content>> {
        let row: Option<(String, String, i32, i64, i64)> = sqlx::query_as(
            "SELECT cid, type, nsfw, created_at, updated_at FROM content WHERE cid = ?",
        )
            .bind(cid)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(cid, type_str, nsfw, created_at, updated_at)| Content {
            cid,
            content_type: serde_json::from_str(&format!("\"{}\"", type_str))
                .unwrap_or(ContentType::Anime),
            nsfw: nsfw == 1,
            created_at,
            updated_at,
        }))
    }

    pub async fn get_all_metadata(pool: &SqlitePool, cid: &str) -> CoreResult<Vec<Metadata>> {
        let rows = sqlx::query(
            "SELECT * FROM metadata WHERE cid = ? \
             ORDER BY CASE source_name WHEN 'anilist' THEN 0 ELSE 1 END",
        )
            .bind(cid)
            .fetch_all(pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(Self::map_metadata_row(&row)?);
        }

        Ok(results)
    }

    pub async fn get_by_cid(pool: &SqlitePool, cid: &str) -> CoreResult<Option<Metadata>> {
        let all = Self::get_all_metadata(pool, cid).await?;
        Ok(all.into_iter().next())
    }

    pub async fn find_closest_match(
        pool: &SqlitePool,
        title: &str,
        content_type: Option<ContentType>,
        release_year: Option<i64>,
    ) -> CoreResult<Option<Metadata>> {
        let content_type = match content_type {
            Some(t) => t,
            None => return Ok(None),
        };
        
        type MatchRow = (String, String, String, String);

        let rows: Vec<MatchRow> = if let Some(year) = release_year {
            sqlx::query_as(
                "SELECT m.cid, m.title, m.alt_titles, m.title_i18n \
                 FROM metadata m \
                 JOIN content c ON c.cid = m.cid \
                 WHERE c.type = ? AND (\
                     m.release_date IS NULL \
                     OR CAST(substr(m.release_date, 1, 4) AS INTEGER) BETWEEN ? AND ?\
                 ) \
                 GROUP BY m.cid",
            )
                .bind(content_type.as_str())
                .bind(year - 1)
                .bind(year + 1)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as(
                "SELECT m.cid, m.title, m.alt_titles, m.title_i18n \
                 FROM metadata m \
                 JOIN content c ON c.cid = m.cid \
                 WHERE c.type = ? \
                 GROUP BY m.cid",
            )
                .bind(content_type.as_str())
                .fetch_all(pool)
                .await?
        };

        let target_normalized = normalize_title(title);
        let mut best_match: Option<String> = None;
        let mut highest_score = 0.0_f64;
        const THRESHOLD: f64 = 0.85;

        for (cid, db_title, db_alt_titles_json, db_title_i18n_json) in rows {
            let mut max_local_score = similarity(&target_normalized, &normalize_title(&db_title));

            if let Ok(alt_titles) = serde_json::from_str::<Vec<String>>(&db_alt_titles_json) {
                for alt in alt_titles {
                    if alt.trim().is_empty() { continue; }
                    let score = similarity(&target_normalized, &normalize_title(&alt));
                    if score > max_local_score { max_local_score = score; }
                }
            }

            if let Ok(i18n) = serde_json::from_str::<std::collections::HashMap<String, String>>(&db_title_i18n_json) {
                for (_, localized) in i18n {
                    if localized.trim().is_empty() { continue; }
                    let score = similarity(&target_normalized, &normalize_title(&localized));
                    if score > max_local_score { max_local_score = score; }
                }
            }

            if max_local_score >= THRESHOLD && max_local_score > highest_score {
                highest_score = max_local_score;
                best_match = Some(cid);
            }
        }

        if let Some(cid) = best_match {
            return Self::get_by_cid(pool, &cid).await;
        }

        Ok(None)
    }

    pub async fn get_full_content(pool: &SqlitePool, cid: &str) -> CoreResult<Option<FullContent>> {

        let content = match Self::get_content_by_cid(pool, cid).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let (metadata, tracker_mappings, extension_sources, relations, content_units) = tokio::try_join!(
            Self::get_all_metadata(pool, cid),
            TrackerRepository::get_mappings_by_cid(pool, cid),
            ExtensionRepository::get_by_cid(pool, cid),
            RelationRepository::get_by_source(pool, cid),
            UnitRepository::get_by_cid(pool, cid),
        )?;

        Ok(Some(FullContent {
            content,
            metadata,
            tracker_mappings,
            extension_sources,
            relations,
            content_units,
        }))
    }

    fn map_metadata_row(row: &sqlx::sqlite::SqliteRow) -> CoreResult<Metadata> {
        Ok(Metadata {
            id:              row.try_get("id")?,
            cid:             row.try_get("cid")?,
            source_name:     row.try_get("source_name")?,
            source_id:       row.try_get("source_id")?,
            subtype:         row.try_get("subtype")?,
            title:           row.try_get("title")?,
            alt_titles:      serde_json::from_str(row.try_get::<String, _>("alt_titles")?.as_str()).unwrap_or_default(),
            title_i18n:      serde_json::from_str(row.try_get::<String, _>("title_i18n")?.as_str()).unwrap_or_default(),
            synopsis:        row.try_get("synopsis")?,
            cover_image:     row.try_get("cover_image")?,
            banner_image:    row.try_get("banner_image")?,
            eps_or_chapters: serde_json::from_str(row.try_get::<String, _>("eps_or_chapters")?.as_str())
                .unwrap_or(EpisodeData::Count(0)),
            status:          row.try_get::<Option<String>, _>("status")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            genres:          serde_json::from_str(row.try_get::<String, _>("genres")?.as_str()).unwrap_or_default(),
            release_date:    row.try_get("release_date")?,
            end_date:        row.try_get("end_date")?,
            rating:          row.try_get("rating")?,
            trailer_url:     row.try_get("trailer_url")?,
            characters:      serde_json::from_str(row.try_get::<String, _>("characters")?.as_str()).unwrap_or_default(),
            studio:          row.try_get("studio")?,
            staff:           serde_json::from_str(row.try_get::<String, _>("staff")?.as_str()).unwrap_or_default(),
            external_ids:    serde_json::from_str(row.try_get::<String, _>("external_ids")?.as_str())
                .unwrap_or(serde_json::json!({})),
            episode_duration: row.try_get("episode_duration")?,
            created_at:      row.try_get("created_at")?,
            updated_at:      row.try_get("updated_at")?,
        })
    }

    pub async fn delete(pool: &SqlitePool, cid: &str) -> CoreResult<()> {
        sqlx::query("DELETE FROM content WHERE cid = ?")
            .bind(cid)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn merge(
        pool: &SqlitePool,
        survivor_cid: &str,
        loser_cid: &str,
    ) -> CoreResult<()> {
        if survivor_cid == loser_cid {
            return Ok(());
        }

        let survivor = Self::get_content_by_cid(pool, survivor_cid).await?;
        let loser = Self::get_content_by_cid(pool, loser_cid).await?;

        let (survivor, loser) = match (survivor, loser) {
            (Some(s), Some(l)) => (s, l),
            _ => {
                return Err(CoreError::NotFound("survivor or loser cid not found".into()));
            }
        };

        if survivor.content_type != loser.content_type {
            return Err(CoreError::NotFound(
                "cannot merge entries of different content types".into(),
            ));
        }

        let mut tx = pool.begin().await?;

        sqlx::query(
            r#"
        UPDATE metadata SET cid = ?
        WHERE cid = ?
          AND source_name NOT IN (
              SELECT source_name FROM metadata WHERE cid = ?
          )
        "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
        UPDATE tracker_mappings SET cid = ?
        WHERE cid = ?
          AND tracker_name NOT IN (
              SELECT tracker_name FROM tracker_mappings WHERE cid = ?
          )
        "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE extension_sources SET cid = ? WHERE cid = ?")
            .bind(survivor_cid)
            .bind(loser_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
        UPDATE content_units SET cid = ?
        WHERE cid = ?
          AND (type, unit_number) NOT IN (
              SELECT type, unit_number FROM content_units WHERE cid = ?
          )
        "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
        UPDATE content_relations SET source_cid = ?
        WHERE source_cid = ?
          AND (target_tracker_name, target_tracker_id, relation_type) NOT IN (
              SELECT target_tracker_name, target_tracker_id, relation_type
              FROM content_relations WHERE source_cid = ?
          )
        "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE content_relations SET target_cid = ? WHERE target_cid = ?")
            .bind(survivor_cid)
            .bind(loser_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM content WHERE cid = ?")
            .bind(loser_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE content SET updated_at = ? WHERE cid = ?")
            .bind(Utc::now().timestamp())
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
    UPDATE ListEntry SET cid = ?
    WHERE cid = ?
      AND user_id NOT IN (
          SELECT user_id FROM ListEntry WHERE cid = ?
      )
    "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
    UPDATE AnimeProgress SET cid = ?
    WHERE cid = ?
      AND (user_id, episode) NOT IN (
          SELECT user_id, episode FROM AnimeProgress WHERE cid = ?
      )
    "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
    UPDATE ChapterProgress SET cid = ?
    WHERE cid = ?
      AND (user_id, chapter) NOT IN (
          SELECT user_id, chapter FROM ChapterProgress WHERE cid = ?
      )
    "#,
        )
            .bind(survivor_cid)
            .bind(loser_cid)
            .bind(survivor_cid)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn search_local(
        pool: &SqlitePool,
        query: &str,
        content_type: &ContentType,
    ) -> CoreResult<Vec<SearchResult>> {
        type Row = (String, String, Option<String>, String, String);

        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT m.cid, m.title, m.cover_image, m.alt_titles, m.title_i18n
            FROM metadata m
            JOIN content c ON c.cid = m.cid
            WHERE c.type = ?
            GROUP BY m.cid
            "#,
        )
            .bind(content_type.as_str())
            .fetch_all(pool)
            .await?;

        let normalized_query = normalize_title(query);
        let mut scored: Vec<(f64, SearchResult)> = Vec::new();

        for (cid, title, cover_image, alt_titles_json, title_i18n_json) in rows {
            let mut best = similarity(&normalized_query, &normalize_title(&title));
            let mut best_title = title.clone();

            if let Ok(alts) = serde_json::from_str::<Vec<String>>(&alt_titles_json) {
                for alt in alts {
                    if alt.trim().is_empty() { continue; }
                    let score = similarity(&normalized_query, &normalize_title(&alt));
                    if score > best {
                        best = score;
                        best_title = alt;
                    }
                }
            }

            if let Ok(i18n) = serde_json::from_str::<std::collections::HashMap<String, String>>(&title_i18n_json) {
                for (_, localized) in i18n {
                    if localized.trim().is_empty() { continue; }
                    let score = similarity(&normalized_query, &normalize_title(&localized));
                    if score > best {
                        best = score;
                        best_title = localized;
                    }
                }
            }
            
            const MIN_SCORE: f64 = 0.3;
            if best >= MIN_SCORE || title.to_lowercase().contains(&query.to_lowercase()) {
                scored.push((best, SearchResult {
                    cid,
                    title: best_title,
                    cover_image,
                }));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        Ok(scored.into_iter().map(|(_, r)| r).take(25).collect())
    }
}