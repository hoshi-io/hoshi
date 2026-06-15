use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use crate::content::models::ContentType;
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::repositories::content::ContentRepository;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::tracker::repository::TrackerRepository;
use crate::content::types::AniSkipResponse;
use crate::extensions::types::{ContentItems, EpisodeChapter, PlayContentResult};
use crate::state::AppState;

pub struct ExtensionService;

impl ExtensionService {

    #[instrument(skip(state))]
    pub async fn save_extension_metadata(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
    ) {
        debug!(ext = %ext_name, id = %ext_id, "Fetching metadata from extension");

        let ext_meta = match ContentResolverService::fetch_ext_metadata(state, ext_name, ext_id).await {
            Ok(v) => v,
            Err(e) => {
                warn!(ext = %ext_name, id = %ext_id, error = ?e, "Failed to fetch extension metadata");
                return;
            }
        };

        let now = chrono::Utc::now().timestamp();
        let meta = ContentResolverService::ext_meta_to_metadata(cid, ext_name, ext_id, &ext_meta, now);

        match ContentRepository::upsert_metadata(&state.pool, &meta).await {
            Ok(_) => info!(cid = %cid, source = %ext_name, "Extension metadata saved"),
            Err(e) => error!(cid = %cid, source = %ext_name, error = ?e, "Failed to upsert extension metadata"),
        }
    }

    #[instrument(skip(state))]
    pub async fn get_content_items(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<ContentItems> {
        let cache_key = format!("items:{}:{}", ext_name, cid);

        if let Ok(Some(cached_val)) = CacheRepository::get(&state.pool, &cache_key).await {
            if let Ok(items) = serde_json::from_value::<ContentItems>(cached_val) {
                return Ok(items);
            }
        }

        let (content_type, ext_id) =
            ContentResolverService::ensure_extension_link(state, cid, ext_name).await?;

        let manager = state.extension_manager.read().await;

        let items = match content_type {
            ContentType::Anime => {
                let eps = manager.find_episodes(ext_name, &ext_id).await?;
                ContentItems::Episodes(eps)
            }
            _ => {
                let ch = manager.find_chapters(ext_name, &ext_id).await?;
                ContentItems::Chapters(ch)
            }
        };

        let ttl = match content_type {
            ContentType::Anime => 10800,
            _ => 86400,
        };

        if let Ok(val) = serde_json::to_value(&items) {
            let _ = CacheRepository::set(&state.pool, &cache_key, ext_name, "content_items", &val, ttl).await;
        }

        Ok(items)
    }

    #[instrument(skip(state, server, category))]
    pub async fn play_content(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        number: f64,
        server: Option<String>,
        category: Option<String>,
    ) -> CoreResult<PlayContentResult> {
        let items_list = Self::get_content_items(state, cid, ext_name).await?;

        let (content_type, _ext_id) = {
            let (type_str, id) = ExtensionRepository::get_extension_id_and_type(&state.pool, cid, ext_name)
                .await?
                .ok_or_else(|| CoreError::Internal("error.content.link_failed".into()))?;

            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", type_str))
                .unwrap_or(ContentType::Anime);
            (ct, id)
        };

        let real_id = match &items_list {
            ContentItems::Episodes(eps) => eps
                .iter()
                .find(|ep| {
                    ep.number
                        .map(|n| (n - number).abs() < 0.01)
                        .unwrap_or(false)
                })
                .map(|ep| ep.id.clone()),

            ContentItems::Chapters(ch) => ch
                .iter()
                .find(|c| {
                    c.number
                        .map(|n| (n - number).abs() < 0.01)
                        .unwrap_or(false)
                })
                .map(|c| c.id.clone()),
        }
            .ok_or_else(|| {
                warn!(cid = %cid, ext = %ext_name, number = %number, "Item number not found");
                CoreError::NotFound("error.content.item_number_not_found".into())
            })?;

        let manager = state.extension_manager.read().await;

        match content_type {
            ContentType::Anime => {
                let srv = server.unwrap_or_else(|| "default".into());
                let cat = category.unwrap_or_else(|| "sub".into());
                let cache_key = format!("play:anime:{}:{}:{}:{}", ext_name, real_id, srv, cat);

                if let Ok(Some(cached_val)) = CacheRepository::get(&state.pool, &cache_key).await {
                    if let Ok(res) = serde_json::from_value::<PlayContentResult>(cached_val) {
                        return Ok(res);
                    }
                }

                debug!(ext = %ext_name, id = %real_id, server = %srv, "Fetching video servers");

                let mut data = manager.find_episode_server(ext_name, &real_id, &srv, &cat).await?;

                if data.source.chapters.is_empty() {
                    let mappings = TrackerRepository::get_mappings_by_cid(&state.pool, cid).await.unwrap_or_default();

                    let mal_id = mappings.iter()
                        .find(|m| m.tracker_name == "mal")
                        .and_then(|m| {
                            m.tracker_id.strip_prefix("anime:")?.parse::<i64>().ok()
                        });

                    if let Some(id) = mal_id {
                        debug!(mal_id = %id, ep = %number, "Chapters empty, fetching from AniSkip");

                        let url = format!("https://api.aniskip.com/v2/skip-times/{}/{}", id, number);

                        let res = state.http_client
                            .get(url)
                            .query(&[
                                ("types", "op"),
                                ("types", "ed"),
                                ("types", "recap"),
                                ("types", "mixed-op"),
                                ("types", "mixed-ed"),
                                ("episodeLength", "0"),
                            ])
                            .send()
                            .await;

                        if let Ok(response) = res {
                            if let Ok(skip_data) = response.json::<AniSkipResponse>().await {
                                let mut chapters: Vec<EpisodeChapter> = skip_data.results.into_iter().map(|r| {
                                    let title = match r.skip_type.as_str() {
                                        "op" => "Opening",
                                        "ed" => "Ending",
                                        "recap" => "Recap",
                                        "mixed-op" => "Mixed Opening",
                                        "mixed-ed" => "Mixed Ending",
                                        _ => "Skip",
                                    };

                                    EpisodeChapter {
                                        start: r.interval.start_time,
                                        end: r.interval.end_time,
                                        title: title.to_string(),
                                    }
                                }).collect();

                                chapters.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
                                data.source.chapters = chapters;
                            }
                        }
                    }
                }

                let result = PlayContentResult::Video(data);

                // Cache stream links with a low TTL (30 minutes)
                if let Ok(val) = serde_json::to_value(&result) {
                    let _ = CacheRepository::set(&state.pool, &cache_key, ext_name, "play_anime", &val, 1800).await;
                }

                Ok(result)
            }

            ContentType::Manga => {
                let cache_key = format!("play:manga:{}:{}", ext_name, real_id);

                if let Ok(Some(cached_val)) = CacheRepository::get(&state.pool, &cache_key).await {
                    if let Ok(res) = serde_json::from_value::<PlayContentResult>(cached_val) {
                        return Ok(res);
                    }
                }

                debug!(ext = %ext_name, id = %real_id, "Fetching chapter pages");

                let data = manager.find_manga_pages(ext_name, &real_id).await?;
                let result = PlayContentResult::Reader(data);

                if let Ok(val) = serde_json::to_value(&result) {
                    let _ = CacheRepository::set(&state.pool, &cache_key, ext_name, "play_manga", &val, 86400).await;
                }

                Ok(result)
            }

            ContentType::Novel => {
                let cache_key = format!("play:novel:{}:{}", ext_name, real_id);

                if let Ok(Some(cached_val)) = CacheRepository::get(&state.pool, &cache_key).await {
                    if let Ok(res) = serde_json::from_value::<PlayContentResult>(cached_val) {
                        return Ok(res);
                    }
                }

                debug!(ext = %ext_name, id = %real_id, "Fetching novel HTML");

                let html = manager.find_novel_html(ext_name, &real_id).await?;
                let result = PlayContentResult::Novel(html);

                if let Ok(val) = serde_json::to_value(&result) {
                    let _ = CacheRepository::set(&state.pool, &cache_key, ext_name, "play_novel", &val, 86400).await;
                }

                Ok(result)
            }
        }
    }
}