use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use futures::future::join_all;
use tracing::{debug, info, instrument, warn};
use crate::content::{repositories::content::ContentRepository};
use crate::content::repositories::cache::CacheRepository;
use crate::content::services::import::ImportService;
use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::list::types::ListEntry;
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
        let pool = state.pool();
        let now  = Utc::now().timestamp();

        let list_map = Self::build_list_map(pool, user_id).await?;
        let raw = Self::fetch_or_build_schedule(&state, user_id, &window, now).await?;

        let enriched = raw
            .into_iter()
            .map(|mut e| {
                if let Some(le) = list_map.get(&e.tracker_id) {
                    e.user_status   = Some(le.status.clone());
                    e.user_progress = Some(le.progress);
                    e.user_score    = le.score;
                }
                e
            })
            .collect();

        Ok(enriched)
    }

    async fn build_list_map(pool: &sqlx::SqlitePool, user_id: i32) -> CoreResult<HashMap<String, ListEntry>> {
        let current  = ListRepository::get_entries(pool, user_id, Some("CURRENT")).await?;
        let planning = ListRepository::get_entries(pool, user_id, Some("PLANNING")).await?;
        let entries: Vec<ListEntry> = current.into_iter().chain(planning).collect();

        let lookups = entries.into_iter().map(|entry| async move {
            let mappings = TrackerRepository::get_mappings_by_cid(pool, &entry.cid).await;
            (entry, mappings)
        });

        let mut list_map: HashMap<String, ListEntry> = HashMap::new();
        for (entry, mappings) in join_all(lookups).await {
            let mappings = match mappings {
                Ok(m) => m,
                Err(e) => { warn!(cid = %entry.cid, error = ?e, "Failed to fetch tracker mappings, skipping entry"); continue; }
            };
            if let Some(m) = mappings.into_iter().find(|m| m.tracker_name == "anilist") {
                list_map.insert(m.tracker_id, entry);
            }
        }

        Ok(list_map)
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
        let mut entries: Vec<AiringEntryEnriched> = Vec::with_capacity(episodes.len());

        for episode in episodes {
            let Some(media) = &episode.media else { continue };
            let tracker_id = media.tracker_id.clone();

            let cid = match TrackerRepository::find_cid_by_tracker(
                pool, "anilist", &tracker_id,
            ).await? {
                Some(cid) => cid,
                None => match ImportService::import_media(pool, "anilist", media).await {
                    Ok(cid) => cid,
                    Err(e) => {
                        warn!(tracker_id = %tracker_id, error = ?e, "Failed to import media for airing entry");
                        continue;
                    }
                },
            };

            let full_content = match ContentRepository::get_full_content(pool, &cid).await? {
                Some(fc) => fc,
                None => {
                    warn!(cid = %cid, "Missing full content after import, skipping entry");
                    continue;
                }
            };

            entries.push(AiringEntryEnriched {
                tracker_id,
                episode: episode.episode,
                airing_at: episode.airing_at,
                full_content,
                user_status: None,
                user_progress: None,
                user_score: None,
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