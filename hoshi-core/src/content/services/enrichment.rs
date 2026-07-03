use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use sqlx::SqlitePool;
use crate::content::models::{ContentType, FullContent};
use crate::content::repositories::content::ContentRepository;
use crate::content::services::import::ImportService;
use crate::content::services::mapping::MappingService;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::TrackerMapping;

pub struct EnrichmentService;

impl EnrichmentService {

    pub async fn create_enriched_content(
        state: &Arc<AppState>,
        c_type: &ContentType,
        media: &TrackerMedia,
        id: &str,
        tracker: &str,
        provided_cross_ids: Option<&HashMap<String, String>>,
    ) -> CoreResult<FullContent> {
        if let Some(full) = Self::find_existing_cid_via_cross_ids(state, media, tracker, id).await? {
            return Ok(full);
        }

        match c_type {
            ContentType::Anime => {
                Self::create_enriched_anime(state, media, tracker, id, provided_cross_ids).await
            }
            ContentType::Manga | ContentType::Novel => {
                Self::create_enriched_manga_or_novel(state, media, tracker, id, provided_cross_ids).await
            }
        }
    }

    async fn find_existing_cid_via_cross_ids(
        state: &Arc<AppState>,
        media: &TrackerMedia,
        tracker: &str,
        id: &str,
    ) -> CoreResult<Option<FullContent>> {
        if let Ok(Some(cid)) = TrackerRepository::find_cid_by_tracker(
            &state.pool, tracker, id
        ).await {
            info!(cid = %cid, tracker = %tracker, id = %id, "Found existing CID via direct mapping");
            let now = chrono::Utc::now().timestamp();
            MappingService::add_tracker_mapping(&state.pool, TrackerMapping {
                cid: cid.clone(),
                tracker_name: tracker.into(),
                tracker_id: id.into(),
                tracker_url: None,
                created_at: now,
                updated_at: now,
            }).await.ok();
            return Ok(ContentRepository::get_full_content(&state.pool, &cid).await?);
        }

        for (cross_tracker, cross_id) in &media.cross_ids {
            if let Ok(Some(cid)) = TrackerRepository::find_cid_by_tracker(
                &state.pool, cross_tracker, cross_id
            ).await {
                info!(cid = %cid, via = %cross_tracker, "Found existing CID via cross-ID, linking");
                let now = chrono::Utc::now().timestamp();
                MappingService::add_tracker_mapping(&state.pool, TrackerMapping {
                    cid: cid.clone(),
                    tracker_name: tracker.into(),
                    tracker_id: id.into(),
                    tracker_url: None,
                    created_at: now,
                    updated_at: now,
                }).await.ok();
                return Ok(ContentRepository::get_full_content(&state.pool, &cid).await?);
            }
        }

        Ok(None)
    }

    async fn create_enriched_anime(
        state: &Arc<AppState>,
        media: &TrackerMedia,
        tracker: &str,
        id: &str,
        provided_cross_ids: Option<&HashMap<String, String>>,
    ) -> CoreResult<FullContent> {
        let cid = ImportService::import_media(&state.pool, tracker, media).await?;
        let now = chrono::Utc::now().timestamp();

        info!(cid = %cid, tracker = %tracker, id = %id, "Anime imported, resolving cross IDs");

        let cross_ids: HashMap<String, String> = match provided_cross_ids {
            Some(ids) => {
                ids.clone()
            }
            None => {
                let endpoint = match tracker.to_lowercase().as_str() {
                    "anilist"                              => format!("anilist/{}", id),
                    "mal" | "myanimelist"                  => {
                        let raw_id = id.split_once(':').map_or(id, |(_, id)| id);
                        format!("myanimelist/{}", raw_id)
                    }
                    "kitsu"                                => format!("kitsu/{}", id),
                    "simkl"                                => format!("simkl/{}", id),
                    "trakt"                                => format!("trakt/show/{}", id),
                    "annict"                               => format!("annict/{}", id),
                    "hikka"                                => format!("hikka/{}", id),
                    "notify"                               => format!("notify/{}", id),
                    "shikimori"                            => format!("shikimori/{}", id),
                    _ => return Err(CoreError::Internal("error.enrichment.unsupported_tracker".into())),
                };
                let url = format!("https://animeapi.my.id/{}", endpoint);

                let resp = state
                    .http_client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "Failed to fetch anime mappings");
                        CoreError::Network("error.system.network".into())
                    })?;

                let data: serde_json::Value = resp.json().await.map_err(|e| {
                    error!(error = ?e, "Failed to parse anime mappings");
                    CoreError::Parse("error.system.parse".into())
                })?;
                
                let ids = Self::extract_anime_cross_ids(&data);
                ids
            }
        };

        if cross_ids.is_empty() {
            warn!(cid = %cid, tracker = %tracker, id = %id, "No cross IDs found for anime, skipping mapping persistence");
        } else {
            let normalized: HashMap<String, String> = cross_ids.into_iter().map(|(k, v)| {
                if k == "mal" { (k, format!("{}:{}", "anime", v)) } else { (k, v) }
            }).collect();

            info!(cid = %cid, count = normalized.len(), mappings = ?normalized, "Persisting anime mappings");
            Self::persist_anime_mappings(&state.pool, &cid, &normalized, now).await;
        }

        ContentRepository::get_full_content(&state.pool, &cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))
    }

    async fn create_enriched_manga_or_novel(
        state: &Arc<AppState>,
        media: &TrackerMedia,
        tracker: &str,
        id: &str,
        provided_cross_ids: Option<&HashMap<String, String>>,
    ) -> CoreResult<FullContent> {
        let cid = ImportService::import_media(&state.pool, tracker, media).await?;
        let now = chrono::Utc::now().timestamp();

        info!(cid = %cid, tracker = %tracker, id = %id, "Manga/novel imported, resolving cross IDs");

        let cross_ids: HashMap<String, String> = match provided_cross_ids {
            Some(ids) => {
                ids.clone()
            }
            None => {
                // strip manga: prefix from myanimelist ids
                let id = id.strip_prefix("manga:").unwrap_or(id);

                let endpoint = match tracker.to_lowercase().as_str() {
                    "anilist"                               => format!("/v1/source/anilist/{}", id),
                    "kitsu"                                 => format!("/v1/source/kitsu/{}", id),
                    "animeplanet" | "anime-planet"          => format!("/v1/source/anime-planet/{}", id),
                    "mangaupdates" | "manga-updates"        => format!("/v1/source/manga-updates/{}", id),
                    "mal" | "myanimelist" | "my-anime-list" => format!("/v1/source/my-anime-list/{}", id),
                    _ => return Err(CoreError::Internal("error.enrichment.unsupported_tracker".into())),
                };
                let url = format!("https://api.mangabaka.dev{}", endpoint);

                let resp = state
                    .http_client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "Failed to fetch manga mappings");
                        CoreError::Network("error.system.network".into())
                    })?;

                let data: serde_json::Value = resp.json().await.map_err(|e| {
                    error!(error = ?e, "Failed to parse manga mappings");
                    CoreError::Parse("error.system.parse".into())
                })?;


                let ids = Self::extract_manga_cross_ids(&data);
                ids
            }
        };

        if cross_ids.is_empty() {
            warn!(cid = %cid, tracker = %tracker, id = %id, "No cross IDs found for manga/novel, skipping mapping persistence");
        } else {
            let normalized: HashMap<String, String> = cross_ids.into_iter().map(|(k, v)| {
                if k == "mal" { (k, format!("{}:{}", "manga", v)) } else { (k, v) }
            }).collect();

            info!(cid = %cid, count = normalized.len(), mappings = ?normalized, "Persisting manga/novel mappings");
            Self::persist_manga_mappings(&state.pool, &cid, &normalized, now).await;
        }

        ContentRepository::get_full_content(&state.pool, &cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))
    }

    pub fn extract_anime_cross_ids(data: &serde_json::Value) -> HashMap<String, String> {
        let allowed = [
            "anidb",
            "anilist",
            "animeplanet",
            "hikka",
            "kitsu",
            "myanimelist",
            "notify",
            "shikimori",
            "simkl",
            "trakt",
            "annict",
        ];
        let mut out = HashMap::new();

        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                if !allowed.contains(&key.as_str()) || value.is_null() { continue; }
                let id_str = if let Some(s) = value.as_str() {
                    s.to_string()
                } else if let Some(n) = value.as_i64() {
                    n.to_string()
                } else {
                    continue;
                };
                let normalized = match key.as_str() {
                    "myanimelist" => "mal",
                    other => other,
                };
                out.insert(normalized.to_string(), id_str);
            }
        }
        out
    }

    pub fn extract_manga_cross_ids(data: &serde_json::Value) -> HashMap<String, String> {
        let allowed = [
            "anilist", "anime_planet", "anime_news_network",
            "kitsu", "manga_updates", "my_anime_list", "shikimori",
        ];
        let mut out = HashMap::new();

        if let Some(arr) = data.pointer("/data/series").and_then(|v| v.as_array()) {
            if let Some(source_obj) = arr.first()
                .and_then(|s| s.get("source"))
                .and_then(|s| s.as_object())
            {
                for (key, val) in source_obj {
                    if !allowed.contains(&key.as_str()) { continue; }
                    let id_str = match val.get("id") {
                        Some(serde_json::Value::Number(n)) => n.to_string(),
                        Some(serde_json::Value::String(s)) => s.clone(),
                        _ => continue,
                    };
                    let normalized = match key.as_str() {
                        "my_anime_list" | "myanimelist" => "mal",
                        "manga_updates"                 => "mangaupdates",
                        "anime_planet"                  => "animeplanet",
                        "anime_news_network"            => "animenewsnetwork",
                        other                           => other,
                    };
                    out.insert(normalized.to_string(), id_str);
                }
            }
        }
        out
    }

    async fn persist_anime_mappings(
        pool: &SqlitePool,
        cid: &str,
        cross_ids: &HashMap<String, String>,
        now: i64,
    ) {
        for (tracker_name, tracker_id) in cross_ids {
            match MappingService::add_tracker_mapping(pool, TrackerMapping {
                cid: cid.to_string(),
                tracker_name: tracker_name.clone(),
                tracker_id: tracker_id.clone(),
                tracker_url: None,
                created_at: now,
                updated_at: now,
            }).await {
                Ok(_) => debug!(cid = %cid, tracker_name = %tracker_name, "Mapping saved OK"),
                Err(e) => error!(cid = %cid, tracker_name = %tracker_name, tracker_id = %tracker_id, error = ?e, "Failed to save tracker mapping"),
            }
        }
    }

    async fn persist_manga_mappings(
        pool: &SqlitePool,
        cid: &str,
        cross_ids: &HashMap<String, String>,
        now: i64,
    ) {
        Self::persist_anime_mappings(pool, cid, cross_ids, now).await;
    }
}