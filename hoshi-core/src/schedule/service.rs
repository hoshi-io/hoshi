use std::sync::Arc;
use chrono::Utc;
use tracing::{debug, info, instrument, warn};
use crate::content::repositories::content::{ContentRepository, FullContentParts};
use crate::content::repositories::cache::CacheRepository;
use crate::content::services::import::ImportService;
use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::schedule::types::{AiringEntryEnriched, ScheduleWindow};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const SCHEDULE_CACHE_TTL: i64 = 3 * 3600;

fn cache_key(user_id: i32, window: &ScheduleWindow) -> String {
    format!("schedule:anilist:{user_id}:{}:{}", window.days_back, window.days_ahead)
}

pub struct ScheduleService;

impl ScheduleService {
    #[instrument(skip(state))]
    pub async fn get_schedule(
        state: Arc<AppState>,
        user_id: i32,
        window: ScheduleWindow,
    ) -> CoreResult<Vec<AiringEntryEnriched>> {
        let now = Utc::now().timestamp();
        Self::fetch_or_build_schedule(&state, user_id, &window, now).await
    }

    async fn fetch_or_build_schedule(
        state: &Arc<AppState>,
        user_id: i32,
        window: &ScheduleWindow,
        now: i64,
    ) -> CoreResult<Vec<AiringEntryEnriched>> {
        let pool = state.pool();
        let key = cache_key(user_id, window);

        if let Some(cached) = CacheRepository::get(pool, &key).await? {
            debug!("Schedule cache hit");
            return serde_json::from_value(cached)
                .map_err(|e| core_err!(Internal, "error.schedule.cache_deserialize_failed", e));
        }

        debug!("Schedule cache miss, fetching from AniList");

        let from_ts = now - window.days_back  * 86_400;
        let to_ts   = now + window.days_ahead * 86_400;

        let provider = state
            .tracker_registry
            .get("anilist")
            .ok_or_else(|| CoreError::Internal("error.tracker.anilist_not_registered".into()))?;

        let episodes = provider.fetch_airing_schedule(from_ts, to_ts).await?;

        let tracker_ids: Vec<&str> = episodes.iter()
            .filter_map(|e| e.media.as_ref().map(|m| m.tracker_id.as_str()))
            .collect();
        let mut cid_by_tracker_id = TrackerRepository::find_cids_by_tracker_ids(pool, "anilist", &tracker_ids).await?;

        // Only titles never seen before need an import (one write each).
        for episode in &episodes {
            let Some(media) = &episode.media else { continue };
            if cid_by_tracker_id.contains_key(&media.tracker_id) {
                continue;
            }
            match ImportService::import_media(pool, "anilist", media).await {
                Ok(cid) => { cid_by_tracker_id.insert(media.tracker_id.clone(), cid); }
                Err(e) => warn!(tracker_id = %media.tracker_id, error = ?e, "Failed to import media for airing entry"),
            }
        }

        let cids: Vec<&str> = cid_by_tracker_id.values().map(String::as_str).collect();
        let contents = ContentRepository::get_full_contents(pool, &cids, FullContentParts::ALL).await?;

        let mut entries: Vec<AiringEntryEnriched> = Vec::with_capacity(episodes.len());
        for episode in episodes {
            let Some(media) = episode.media else { continue };
            let Some(full_content) = cid_by_tracker_id.get(&media.tracker_id).and_then(|cid| contents.get(cid)) else {
                continue;
            };
            entries.push(AiringEntryEnriched {
                tracker_id: media.tracker_id,
                episode: episode.episode,
                airing_at: episode.airing_at,
                full_content: full_content.clone(),
            });
        }

        info!(count = entries.len(), "Fetched global airing schedule from AniList");

        CacheRepository::set(
            pool, &key, "anilist", "airing_schedule",
            &serde_json::to_value(&entries)
                .map_err(|e| core_err!(Internal, "error.schedule.cache_serialize_failed", e))?,
            SCHEDULE_CACHE_TTL,
        ).await?;

        Ok(entries)
    }
}