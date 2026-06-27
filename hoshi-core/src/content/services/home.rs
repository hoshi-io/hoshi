use std::sync::Arc;
use chrono::Utc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use crate::content::models::FullContent;
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::content::ContentRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::types::{HomeView, AnimeSection, MangaSection, NovelSection};
use crate::content::utils::show_adult;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;
use crate::tracker::repository::TrackerRepository;

const HOME_CACHE_KEY: &str = "home_view_v1";
const HOME_CACHE_TTL: i64  = 6 * 3600;
const IMPORT_CONCURRENCY: usize = 8;

pub struct HomeService;

impl HomeService {
    pub fn warmup(state: Arc<AppState>) {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            match CacheRepository::get(&state.pool, HOME_CACHE_KEY).await {
                Ok(Some(_)) => {
                    info!("Home cache already warm, skipping warmup");
                    return;
                }
                _ => {}
            }

            info!("Starting home cache warmup...");
            if let Err(e) = Self::refresh_home_cache(state).await {
                error!(error = ?e, "Home cache warmup failed");
            }
        });
    }

    async fn refresh_home_cache(state: Arc<AppState>) -> CoreResult<()> {
        let provider = state.tracker_registry.get("anilist")
            .ok_or_else(|| CoreError::Internal("error.tracker.anilist_not_registered".into()))?;

        let sections = provider.get_home().await?;

        let section_keys = [
            "trending_anime", "popular_anime", "top_rated_anime",
            "seasonal_anime", "upcoming_anime", "recently_finished_anime",
            "top_action_anime",
            "trending_manga", "popular_manga", "top_rated_manga",
            "seasonal_manga", "recently_finished_manga",
            "trending_novel", "popular_novel", "top_rated_novel",
            "recently_finished_novel",
        ];

        let mut seen: std::collections::HashMap<String, TrackerMedia> =
            std::collections::HashMap::new();

        for key in &section_keys {
            for media in sections.get(*key).cloned().unwrap_or_default() {
                seen.entry(media.tracker_id.clone()).or_insert(media);
            }
        }

        info!(unique_items = seen.len(), "Importing home entries");

        let semaphore = Arc::new(Semaphore::new(IMPORT_CONCURRENCY));

        let handles: Vec<_> = seen.into_iter().map(|(tracker_id, media)| {
            let sem   = semaphore.clone();
            let state = state.clone();
            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result  = Self::import_and_load(&state, &media).await;
                (tracker_id, result)
            })
        }).collect();

        let mut cache: std::collections::HashMap<String, FullContent> =
            std::collections::HashMap::new();

        for handle in handles {
            match handle.await {
                Ok((id, Ok(full))) => { cache.insert(id, full); }
                Ok((id, Err(e)))   => warn!(error = ?e, %id, "Failed to import home entry"),
                Err(e)             => warn!(error = ?e, "Import task panicked"),
            }
        }

        let cache: std::collections::HashMap<String, FullContent> = cache
            .into_iter()
            .map(|(id, fc)| (id, fc.slim_for_home()))
            .collect();

        let lookup = |key: &str| -> Vec<FullContent> {
            sections.get(key).cloned().unwrap_or_default()
                .into_iter()
                .filter_map(|m| cache.get(&m.tracker_id).cloned())
                .collect()
        };

        let view = HomeView {
            anime: AnimeSection {
                trending:          lookup("trending_anime"),
                popular:           lookup("popular_anime"),
                top_rated:         lookup("top_rated_anime"),
                seasonal:          lookup("seasonal_anime"),
                upcoming:          lookup("upcoming_anime"),
                recently_finished: lookup("recently_finished_anime"),
                top_action:        lookup("top_action_anime"),
            },
            manga: MangaSection {
                trending:          lookup("trending_manga"),
                popular:           lookup("popular_manga"),
                top_rated:         lookup("top_rated_manga"),
                seasonal:          lookup("seasonal_manga"),
                recently_finished: lookup("recently_finished_manga"),
            },
            novel: NovelSection {
                trending:          lookup("trending_novel"),
                popular:           lookup("popular_novel"),
                top_rated:         lookup("top_rated_novel"),
                recently_finished: lookup("recently_finished_novel"),
            },
            cached_at: Utc::now().timestamp(),
        };

        let value = serde_json::to_value(&view)
            .map_err(|_| CoreError::Internal("error.content.serialization".into()))?;

        CacheRepository::set(
            &state.pool, HOME_CACHE_KEY, "anilist", "home",
            &value, 30 * 24 * 3600,
        ).await?;

        info!("Home cache refreshed ({} unique items imported)", cache.len());
        Ok(())
    }

    async fn import_and_load(state: &Arc<AppState>, media: &TrackerMedia) -> CoreResult<FullContent> {
        let existing = TrackerRepository::find_cid_by_tracker(
            &state.pool, "anilist", &media.tracker_id,
        ).await?;

        if let Some(cid) = existing {
            return ContentRepository::get_full_content(&state.pool, &cid).await?
                .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()));
        }

        EnrichmentService::create_enriched_content(
            state,
            &media.content_type,
            media,
            &media.tracker_id,
            "anilist",
            None,
        ).await
    }

    pub async fn get_home_view(state: &Arc<AppState>, user_id: i32) -> CoreResult<HomeView> {
        let adult_enabled = show_adult(state, user_id).await;

        if let Some(cached_value) = CacheRepository::get(&state.pool, HOME_CACHE_KEY).await? {
            let mut view: HomeView = serde_json::from_value(cached_value)
                .map_err(|e| CoreError::Internal(format!("Cache corrupto: {}", e)))?;

            if Utc::now().timestamp() - view.cached_at > HOME_CACHE_TTL {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::refresh_home_cache(state_clone).await {
                        error!(error = ?e, "Background home refresh failed");
                    }
                });
            }

            if !adult_enabled { view.filter_nsfw(); }
            return Ok(view);
        }

        Self::refresh_home_cache(state.clone()).await?;

        let value = CacheRepository::get(&state.pool, HOME_CACHE_KEY)
            .await?
            .ok_or_else(|| CoreError::Internal("error.content.home_cache_missing".into()))?;

        let mut view: HomeView = serde_json::from_value(value)?;
        if !adult_enabled { view.filter_nsfw(); }
        Ok(view)
    }

    pub async fn get_trending(
        state: &Arc<AppState>,
        media_type: &str,
        user_id: i32,
    ) -> CoreResult<Vec<FullContent>> {
        let view = Self::get_home_view(state, user_id).await?;
        Ok(match media_type {
            "anime" => view.anime.trending,
            "manga" => view.manga.trending,
            "novel" => view.novel.trending,
            _ => return Err(CoreError::NotFound("error.content.invalid_media_type".into())),
        })
    }
}