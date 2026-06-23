use std::collections::HashMap;
use tracing::{debug, instrument};
use crate::content::models::{ContentType, ContentUnit};
use crate::content::repositories::content::ContentRepository;
use crate::error::CoreResult;
use crate::progress::repository::ProgressRepository;
use crate::progress::types::{
    ContentProgressResponse, ContinueItem, ContinueWatchingResponse,
    ProgressResponse, UpdateAnimeProgressBody, UpdateChapterProgressBody,
};
use crate::state::AppState;

pub struct ProgressService;

impl ProgressService {
    #[instrument(skip(state))]
    pub async fn update_anime_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateAnimeProgressBody,
    ) -> CoreResult<ProgressResponse> {
        ProgressRepository::upsert_anime(state.pool(), user_id, &body).await?;

        debug!(cid = %body.cid, episode = body.episode, "Anime progress saved successfully");

        Ok(ProgressResponse { success: true })
    }

    #[instrument(skip(state))]
    pub async fn update_chapter_progress(
        state: &AppState,
        user_id: i32,
        body: UpdateChapterProgressBody,
    ) -> CoreResult<ProgressResponse> {
        ProgressRepository::upsert_chapter(state.pool(), user_id, &body).await?;

        debug!(cid = %body.cid, chapter = body.chapter, "Chapter progress saved successfully");

        Ok(ProgressResponse { success: true })
    }

    #[instrument(skip(state))]
    pub async fn get_continue_watching(
        state: &AppState,
        user_id: i32,
        limit: Option<i64>,
    ) -> CoreResult<ContinueWatchingResponse> {
        let limit = limit.unwrap_or(20);
        let pool = state.pool();

        let anime_rows = ProgressRepository::get_latest_anime_per_cid(pool, user_id, limit).await?;
        let chapter_rows = ProgressRepository::get_latest_chapter_per_cid(pool, user_id, limit).await?;

        let mut items: Vec<ContinueItem> = Vec::new();

        for row in anime_rows {
            let (title, cover_image, title_i18n, nsfw, units) =
                Self::fetch_enriched_data(state, &row.cid).await;

            let (display_episode, display_timestamp, _) = if row.completed {
                (row.episode + 1, Some(0), false)
            } else {
                (row.episode, Some(row.timestamp_seconds), row.completed)
            };

            let unit = units.into_iter().find(|u| u.unit_number == display_episode as f64);

            // If the user finished the episode but there IS NO next episode,
            if row.completed && unit.is_none() {
                continue;
            }

            items.push(ContinueItem {
                cid: row.cid,
                content_type: "anime".into(),
                title,
                title_i18n,
                cover_image,
                nsfw,
                episode: Some(display_episode),
                timestamp_seconds: display_timestamp,
                episode_duration_seconds: if row.completed { None } else { row.episode_duration_seconds },
                chapter: None,
                unit,
                last_accessed: row.last_accessed,
            });
        }

        for row in chapter_rows {
            if items.iter().any(|i| i.cid == row.cid) {
                continue;
            }

            let (title, cover_image, title_i18n, nsfw, units) =
                Self::fetch_enriched_data(state, &row.cid).await;
            let content_type = Self::fetch_content_type(state, &row.cid).await;

            let unit = units.into_iter().find(|u| u.unit_number == row.chapter as f64);

            items.push(ContinueItem {
                cid: row.cid,
                content_type,
                title,
                title_i18n,
                cover_image,
                nsfw,
                episode: None,
                timestamp_seconds: None,
                episode_duration_seconds: None,
                chapter: Some(row.chapter),
                unit,
                last_accessed: row.last_accessed,
            });
        }

        items.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        items.truncate(limit as usize);
        
        Ok(ContinueWatchingResponse { items })
    }

    async fn fetch_enriched_data(
        state: &AppState,
        cid: &str,
    ) -> (String, Option<String>, HashMap<String, String>, bool, Vec<ContentUnit>) {
        ContentRepository::get_full_content(state.pool(), cid)
            .await
            .ok()
            .flatten()
            .map(|f| {
                let primary = f.primary_metadata();
                let title = primary.map(|m| m.title.clone()).unwrap_or_else(|| "Unknown".into());
                let cover_image = primary.and_then(|m| m.cover_image.clone());
                let title_i18n = primary.map(|m| m.title_i18n.clone()).unwrap_or_default();
                (title, cover_image, title_i18n, f.content.nsfw, f.content_units)
            })
            .unwrap_or_else(|| ("Unknown".into(), None, HashMap::new(), false, vec![]))
    }

    #[instrument(skip(state))]
    pub async fn get_content_progress(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<ContentProgressResponse> {
        let (anime_progress, chapter_progress) =
            ProgressRepository::get_progress_for_cid(state.pool(), user_id, &cid).await?;

        Ok(ContentProgressResponse {
            cid,
            anime_progress,
            chapter_progress,
        })
    }

    async fn fetch_content_type(state: &AppState, cid: &str) -> String {
        ContentRepository::get_content_by_cid(state.pool(), cid)
            .await
            .ok()
            .flatten()
            .map(|c| match c.content_type {
                ContentType::Anime => "anime",
                ContentType::Manga => "manga",
                ContentType::Novel => "novel",
            })
            .unwrap_or("unknown")
            .to_string()
    }
}