use std::sync::Arc;
use sqlx::SqlitePool;
use chrono::Utc;
use tracing::{error, info, instrument, warn};
use serde_json::json;

use crate::content::models::{ContentType, EpisodeData, ExtensionSource, FullContent, Metadata};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::extensions::ExtensionService;
use crate::content::utils::{generate_cid, normalize_title, similarity};
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionMetadata;
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

pub struct ContentResolverService;

impl ContentResolverService {

    #[instrument(skip(state))]
    pub async fn ensure_extension_link(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<(ContentType, String)> {
        let existing = ExtensionRepository::get_extension_id_and_type(&state.pool, cid, ext_name).await?;

        if let Some((type_str, id)) = existing {
            let ct = Self::parse_content_type(&type_str);

            let has_meta = ExtensionRepository::has_metadata(&state.pool, cid, ext_name).await?;
            if !has_meta {
                ExtensionService::save_extension_metadata(state, cid, ext_name, &id).await;
            }
            return Ok((ct, id));
        }

        let content = ContentRepository::get_content_by_cid(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))?;
        let meta = ContentRepository::get_by_cid(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.metadata_not_found".into()))?;

        let title = meta.title;
        let ct = content.content_type;

        let search_results = state
            .extension_manager
            .read()
            .await
            .search(ext_name, &title, json!({}), 1)
            .await
            .map_err(|e| {
                error!(ext = %ext_name, error = ?e, "Extension search failed");
                CoreError::Internal("error.content.extension_search_failed".into())
            })?;

        const MIN_SIMILARITY: f64 = 0.8;

        let mut all_titles: Vec<String> = Vec::with_capacity(meta.title_i18n.len() + 1);
        all_titles.push(title.clone());
        all_titles.extend(meta.title_i18n.values().cloned());
        let normalized_queries: Vec<String> = all_titles
            .iter()
            .map(|t| normalize_title(t))
            .collect();

        let best_candidate = search_results
            .iter()
            .filter_map(|item| {
                let normalized_item = normalize_title(&item.title);
                let best_score = normalized_queries
                    .iter()
                    .map(|q| similarity(q, &normalized_item))
                    .fold(0.0_f64, f64::max);
                if best_score >= MIN_SIMILARITY {
                    Some((best_score, item))
                } else {
                    None
                }
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, item)| item)
            .ok_or_else(|| {
                warn!(
            title = %title,
            titles = ?all_titles,
            ext = %ext_name,
            "No search result met similarity threshold across any title variant"
        );
                CoreError::NotFound("error.content.no_match_found".into())
            })?;

        let candidate_id = best_candidate.id.clone();
        let ext_meta = Self::fetch_ext_metadata(state, ext_name, &candidate_id).await?;
        let ext_nsfw = state.extension_manager.read().await.is_nsfw(ext_name);

        Self::resolve_and_link(&state.pool, cid, ext_name, &candidate_id, &ext_meta, &ct, ext_nsfw).await?;
        ExtensionService::save_extension_metadata(state, cid, ext_name, &candidate_id).await;
        Ok((ct, candidate_id))
    }

    #[instrument(skip(pool, ext_meta))]
    pub async fn resolve_and_link(
        pool: &SqlitePool,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<()> {
        if Self::matches_by_tracker_ids(pool, cid, ext_meta, content_type).await? {
            info!(cid = %cid, ext = %ext_name, "Resolved via tracker ID");
            return Self::link(pool, cid, ext_name, ext_id, ext_nsfw).await;
        }

        if let Some(matched) = ContentRepository::find_closest_match(
            pool, &ext_meta.title, Some(content_type.clone()), ext_meta.year,
        ).await? {
            if matched.cid == cid {
                info!(cid = %cid, ext = %ext_name, "Resolved via fuzzy title match");
                return Self::link(pool, cid, ext_name, ext_id, ext_nsfw).await;
            }
            warn!(expected = %cid, matched = %matched.cid, "Fuzzy matched a different CID — rejecting");
        }

        Err(CoreError::NotFound("error.content.extension_no_match".into()))
    }

    pub async fn link_or_enrich_tracker(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_nsfw: bool,
        tracker: &str,
        tracker_id: &str,
        content_type: &ContentType,
    ) -> CoreResult<Option<FullContent>> {
        let maybe_cid = TrackerRepository::find_cid_by_tracker(&state.pool, tracker, tracker_id).await?;

        let cid = if let Some(cid) = maybe_cid {
            cid
        } else {
            info!(tracker = %tracker, id = %tracker_id, "No CID found, enriching from tracker");
            let media = Self::fetch_tracker_media(state, tracker, tracker_id).await?;
            let full = EnrichmentService::create_enriched_content(
                state, content_type, &media, tracker_id, tracker, None
            ).await?;
            full.content.cid
        };

        Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;

        Ok(Some(Self::load_full_content(state, &cid).await?))
    }

    pub async fn create_derived(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
        ext_nsfw: bool,
    ) -> CoreResult<String> {
        if let Some(id_str) = ext_meta.anilist_id.as_ref().map(|v| v.to_string()) {
            if let Ok(Some(cid)) = TrackerRepository::find_cid_by_tracker(
                &state.pool, "anilist", &id_str
            ).await {
                Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;
                return Ok(cid);
            }
        }

        if let Some(id_str) = ext_meta.mal_id.as_ref().map(|v| v.to_string()) {
            let prefix = match content_type {
                ContentType::Anime => "anime",
                ContentType::Manga | ContentType::Novel => "manga",
            };
            let prefixed = if id_str.starts_with(&format!("{}:", prefix)) {
                id_str.clone()
            } else {
                format!("{}:{}", prefix, id_str)
            };
            if let Ok(Some(cid)) = TrackerRepository::find_cid_by_tracker(
                &state.pool, "mal", &prefixed
            ).await {
                Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;
                return Ok(cid);
            }
        }

        let cid = generate_cid();
        let now = Utc::now().timestamp();
        let meta = Self::ext_meta_to_metadata(&cid, ext_name, ext_id, ext_meta, now);
        ContentRepository::create_with_type(&state.pool, content_type, ext_nsfw, meta).await?;
        Self::link(&state.pool, &cid, ext_name, ext_id, ext_nsfw).await?;
        Ok(cid)
    }

    pub fn ext_meta_to_metadata(
        cid: &str,
        source_name: &str,
        source_id: &str,
        ext_meta: &ExtensionMetadata,
        now: i64,
    ) -> Metadata {
        Metadata {
            id: None,
            cid: cid.to_string(),
            source_name: source_name.to_string(),
            source_id: Some(source_id.to_string()),
            subtype: None,
            title: ext_meta.title.clone(),
            alt_titles: vec![],
            title_i18n: Default::default(),
            synopsis: ext_meta.synopsis.clone(),
            cover_image: ext_meta.image.clone(),
            banner_image: ext_meta.image.clone(),
            eps_or_chapters: ext_meta
                .eps_or_chapters
                .map(|n| EpisodeData::Count(n as i32))
                .unwrap_or(EpisodeData::Count(0)),
            status: None,
            genres: ext_meta.genres.clone().unwrap_or_default(),
            release_date: ext_meta.year.map(|y| format!("{}-01-01", y)),
            end_date: None,
            rating: ext_meta.rating.map(|v| v as f32),
            trailer_url: None,
            characters: vec![],
            studio: None,
            staff: vec![],
            external_ids: json!({}),
            episode_duration: None,
            created_at: now,
            updated_at: now,
        }
    }

    async fn matches_by_tracker_ids(
        pool: &SqlitePool,
        expected_cid: &str,
        ext_meta: &ExtensionMetadata,
        content_type: &ContentType,
    ) -> CoreResult<bool> {
        if let Some(id_str) = ext_meta.anilist_id.as_ref().map(|v| v.to_string()) {
            if let Some(cid) = TrackerRepository::find_cid_by_tracker(pool, "anilist", &id_str).await? {
                if cid == expected_cid { return Ok(true); }
                warn!(found = %cid, expected = %expected_cid, "AniList ID matched wrong CID");
            }
        }

        if let Some(id_str) = ext_meta.mal_id.as_ref().map(|v| v.to_string()) {
            let prefix = match content_type {
                ContentType::Anime => "anime",
                ContentType::Manga | ContentType::Novel => "manga",
            };
            let prefixed = if id_str.starts_with(&format!("{}:", prefix)) {
                id_str.clone()
            } else {
                format!("{}:{}", prefix, id_str)
            };
            if let Some(cid) = TrackerRepository::find_cid_by_tracker(pool, "mal", &prefixed).await? {
                if cid == expected_cid { return Ok(true); }
                warn!(found = %cid, expected = %expected_cid, "MAL ID matched wrong CID");
            }
        }

        Ok(false)
    }

    pub async fn link(pool: &SqlitePool, cid: &str, ext_name: &str, ext_id: &str, nsfw: bool) -> CoreResult<()> {
        let now = Utc::now().timestamp();
        ExtensionRepository::add_source(pool, &ExtensionSource {
            id: None,
            cid: cid.to_string(),
            extension_name: ext_name.to_string(),
            extension_id: ext_id.to_string(),
            nsfw,
            language: None,
            created_at: now,
            updated_at: now,
        }).await?;
        Ok(())
    }

    pub async fn load_full_content(state: &Arc<AppState>, cid: &str) -> CoreResult<FullContent> {
        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| {
                warn!(cid = %cid, "Content not found after resolution");
                CoreError::NotFound("error.content.not_found".into())
            })
    }

    pub async fn fetch_tracker_media(
        state: &Arc<AppState>,
        tracker: &str,
        tracker_id: &str,
    ) -> CoreResult<crate::tracker::provider::TrackerMedia> {
        let provider = state.tracker_registry.get(tracker)
            .ok_or_else(|| CoreError::Internal(
                format!("error.tracker.{}_not_registered", tracker).into()
            ))?;
        provider.get_by_id(tracker_id)
            .await
            .map_err(|e| {
                error!(tracker = %tracker, id = %tracker_id, error = ?e, "Failed to fetch from tracker");
                e
            })?
            .ok_or_else(|| {
                warn!(tracker = %tracker, id = %tracker_id, "Tracker returned no media for ID");
                CoreError::NotFound("error.content.tracker_media_not_found".into())
            })
    }

    pub async fn fetch_ext_metadata(
        state: &Arc<AppState>,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<ExtensionMetadata> {
        state
            .extension_manager
            .read()
            .await
            .get_metadata(ext_name, ext_id)
            .await
            .map_err(|e| {
                error!(ext = %ext_name, id = %ext_id, error = ?e, "getMetadata failed");
                CoreError::Internal("error.content.extension_metadata_failed".into())
            })
    }

    fn parse_content_type(s: &str) -> ContentType {
        serde_json::from_str::<ContentType>(&format!("\"{}\"", s))
            .unwrap_or(ContentType::Anime)
    }
}