use std::future::Future;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::SqlitePool;
use tracing::{debug, error, info, instrument, warn};
use crate::content::models::ContentType;
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::repositories::content::ContentRepository;
use crate::content::services::aniskip::AniSkipService;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::{CachedContentItems, ContentItems, PlayContentResult};
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
    
    async fn cached_or<T, F, Fut>(
        pool: &SqlitePool,
        cache_key: &str,
        ext_name: &str,
        cache_type: &str,
        fetch: F,
    ) -> CoreResult<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = CoreResult<(T, i64)>>,
    {
        if let Ok(Some(cached)) = CacheRepository::get(pool, cache_key).await {
            if let Ok(value) = serde_json::from_value(cached) {
                return Ok(value);
            }
        }

        let (value, ttl) = fetch().await?;

        if let Ok(json) = serde_json::to_value(&value) {
            let _ = CacheRepository::set(pool, cache_key, ext_name, cache_type, &json, ttl).await;
        }

        Ok(value)
    }

    fn items_cache_key(cid: &str, ext_name: &str) -> String {
        format!("items:{}:{}", ext_name, cid)
    }

    /// Drops the cached episode/chapter list, e.g. after the extension mapping changed.
    pub async fn invalidate_items(pool: &SqlitePool, cid: &str, ext_name: &str) {
        if let Err(e) = CacheRepository::delete(pool, &Self::items_cache_key(cid, ext_name)).await {
            warn!(cid = %cid, ext = %ext_name, error = ?e, "Failed to invalidate items cache");
        }
    }

    #[instrument(skip(state))]
    pub async fn get_content_items(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<ContentItems> {
        let cache_key = Self::items_cache_key(cid, ext_name);

        let cached: CachedContentItems = Self::cached_or(&state.pool, &cache_key, ext_name, "content_items", || async {
            let (content_type, ext_id) = ContentResolverService::ensure_extension_link(state, cid, ext_name).await?;
            let manager = &state.extension_manager;

            let items = match content_type {
                ContentType::Anime => ContentItems::Episodes(manager.find_episodes(ext_name, &ext_id).await?),
                _ => ContentItems::Chapters(manager.find_chapters(ext_name, &ext_id).await?),
            };

            let ttl = match content_type {
                ContentType::Anime => 10800,
                _ => 86400,
            };
            Ok((items.into(), ttl))
        }).await?;
        Ok(cached.into())
    }

    #[instrument(skip(state, server, category))]
    pub async fn play_content(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        number: f64,
        item_id: Option<String>,
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

        let real_id = Self::find_item_id(&items_list, number, item_id.as_deref())
            .ok_or_else(|| {
                warn!(cid = %cid, ext = %ext_name, number = %number, "Item number not found");
                CoreError::NotFound("error.content.item_number_not_found".into())
            })?;

        let manager = &state.extension_manager;

        match content_type {
            ContentType::Anime => {
                let srv = server.unwrap_or_else(|| "default".into());
                let cat = category.unwrap_or_else(|| "sub".into());
                let cache_key = format!("play:anime:{}:{}:{}:{}", ext_name, real_id, srv, cat);

                Self::cached_or(&state.pool, &cache_key, ext_name, "play_anime", || async {
                    debug!(ext = %ext_name, id = %real_id, server = %srv, "Fetching video servers");
                    let mut data = manager.find_episode_server(ext_name, &real_id, &srv, &cat).await?;

                    if data.source.chapters.is_empty() {
                        data.source.chapters = AniSkipService::fetch_chapters(state, cid, number).await;
                    }

                    Ok((PlayContentResult::Video(data), 1800)) // stream links: short TTL
                }).await
            }

            ContentType::Manga => {
                let cache_key = format!("play:manga:{}:{}", ext_name, real_id);

                Self::cached_or(&state.pool, &cache_key, ext_name, "play_manga", || async {
                    debug!(ext = %ext_name, id = %real_id, "Fetching chapter pages");
                    let data = manager.find_manga_pages(ext_name, &real_id).await?;
                    Ok((PlayContentResult::Reader(data), 86400))
                }).await
            }

            ContentType::Novel => {
                let cache_key = format!("play:novel:{}:{}", ext_name, real_id);

                Self::cached_or(&state.pool, &cache_key, ext_name, "play_novel", || async {
                    debug!(ext = %ext_name, id = %real_id, "Fetching novel HTML");
                    let html = manager.find_novel_html(ext_name, &real_id).await?;
                    Ok((PlayContentResult::Novel(html), 86400))
                }).await
            }
        }
    }

    /// `item_id` pins one version of a chapter (language/scanlator); it must belong to `number`.
    /// Without it, or when it's no longer listed, the first item with that number wins.
    fn find_item_id(items: &ContentItems, number: f64, item_id: Option<&str>) -> Option<String> {
        let matches_number = |n: Option<f64>| n.map(|n| (n - number).abs() < 0.01).unwrap_or(false);

        match items {
            ContentItems::Episodes(eps) => eps.iter().find(|ep| matches_number(ep.number)).map(|ep| ep.id.clone()),
            ContentItems::Chapters(ch) => {
                let mut same_number = ch.iter().filter(|c| matches_number(c.number));
                let pinned = item_id.and_then(|id| same_number.clone().find(|c| c.id == id));
                pinned.or_else(|| same_number.next()).map(|c| c.id.clone())
            }
        }
    }
}