use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

use crate::content::models::{ContentType, FullContent};
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::content::{ContentRepository, FullContentParts};
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::import::ImportService;
use crate::content::types::{DiscoverItem, DiscoverQuery};
use crate::content::utils::show_adult;
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionSearchResult;
use crate::state::AppState;

const PAGE_SIZE: usize = 20;
/// How long a page survives past its soft TTL; within it, it is served stale while refreshing.
const STALE_GRACE: i64 = 7 * 24 * 3600;
pub const TRENDING_TTL: i64 = 6 * 3600;

static KEY_LOCKS: LazyLock<DashMap<String, Arc<Mutex<()>>>> = LazyLock::new(DashMap::new);

#[derive(Serialize, Deserialize)]
struct CachedPage {
    stale_at: i64,
    refs: Vec<CachedRef>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "k", rename_all = "camelCase")]
enum CachedRef {
    Cid { cid: String },
    Ext { item: ExtensionSearchResult },
}

pub struct DiscoverService;

impl DiscoverService {
    #[instrument(skip(state, query))]
    pub async fn fetch(
        state: &Arc<AppState>,
        user_id: i32,
        query: &DiscoverQuery,
        page: u32,
        ttl: i64,
    ) -> CoreResult<Vec<DiscoverItem>> {
        let page = page.max(1);
        let key = cache_key(query, page);

        let cached = match Self::read_cache(state, &key).await {
            Some(c) => {
                if Utc::now().timestamp() > c.stale_at {
                    Self::spawn_refresh(state.clone(), key, query.clone(), page, ttl);
                }
                c
            }
            None => {
                let lock = key_lock(&key);
                let _guard = lock.lock().await;
                match Self::read_cache(state, &key).await {
                    Some(c) => c,
                    None => Self::refresh(state, &key, query, page, ttl).await?,
                }
            }
        };

        let adult = show_adult(state, user_id).await;
        Self::resolve(state, query, cached.refs, adult).await
    }

    /// The tracker trending list for a content type, as plain content.
    pub async fn trending(
        state: &Arc<AppState>,
        user_id: i32,
        content_type: ContentType,
    ) -> CoreResult<Vec<FullContent>> {
        let query = DiscoverQuery::Tracker {
            tracker: "anilist".into(),
            content_type,
            query: None,
            sort: Some("TRENDING_DESC".into()),
            genre: None,
            format: None,
            status: None,
            tags: vec![],
        };

        Ok(Self::fetch(state, user_id, &query, 1, TRENDING_TTL).await?
            .into_iter()
            .filter_map(|item| match item {
                DiscoverItem::Content(c) => Some(c),
                DiscoverItem::Extension { .. } => None,
            })
            .collect())
    }

    /// How fast a query's results change.
    pub fn ttl_for(query: &DiscoverQuery) -> i64 {
        const HOUR: i64 = 3600;
        match query {
            DiscoverQuery::Extension { .. } => 2 * HOUR,
            DiscoverQuery::Tracker { query: Some(q), .. } if !q.trim().is_empty() => 24 * HOUR,
            DiscoverQuery::Tracker { sort, status, .. } => {
                let sort = sort.as_deref().unwrap_or_default().to_ascii_uppercase();
                if sort.contains("TRENDING") {
                    TRENDING_TTL
                } else if status.is_some() || sort.contains("DATE") {
                    12 * HOUR
                } else {
                    24 * HOUR
                }
            }
        }
    }

    async fn read_cache(state: &AppState, key: &str) -> Option<CachedPage> {
        let value = CacheRepository::get(&state.pool, key).await.ok()??;
        serde_json::from_value(value).ok()
    }

    fn spawn_refresh(state: Arc<AppState>, key: String, query: DiscoverQuery, page: u32, ttl: i64) {
        let lock = key_lock(&key);
        tokio::spawn(async move {
            let Ok(_guard) = lock.try_lock() else { return };
            if let Some(c) = Self::read_cache(&state, &key).await {
                if Utc::now().timestamp() <= c.stale_at { return; }
            }
            if let Err(e) = Self::refresh(&state, &key, &query, page, ttl).await {
                error!(key = %key, error = ?e, "Background discover refresh failed");
            }
        });
    }

    /// Unguarded: only call while holding the key's lock.
    async fn refresh(
        state: &Arc<AppState>,
        key: &str,
        query: &DiscoverQuery,
        page: u32,
        ttl: i64,
    ) -> CoreResult<CachedPage> {
        debug!(key = %key, "Fetching discover page");

        let (source, refs) = match query {
            DiscoverQuery::Tracker { tracker, content_type, query, sort, genre, format, status, tags } => {
                let provider = state.tracker_registry.get(tracker)
                    .ok_or_else(|| CoreError::NotFound("error.tracker.no_provider_available".into()))?;

                let medias = provider.search(
                    query.as_deref(),
                    content_type.clone(),
                    PAGE_SIZE,
                    page as usize,
                    sort.as_deref(),
                    genre.as_deref(),
                    format.as_deref(),
                    None,
                    status.as_deref(),
                    Some(tags.as_slice()),
                ).await?;

                let media_refs: Vec<_> = medias.iter().collect();
                let cids = ImportService::ensure_imported(&state.pool, tracker, &media_refs).await?;

                let refs = medias.iter()
                    .filter_map(|m| cids.get(&m.tracker_id))
                    .map(|cid| CachedRef::Cid { cid: cid.clone() })
                    .collect();
                (tracker.as_str(), refs)
            }
            DiscoverQuery::Extension { extension, query, filters } => {
                let manager = &state.extension_manager;
                let ext_nsfw = manager.is_nsfw(extension);
                let filters = if filters.is_null() { serde_json::json!({}) } else { filters.clone() };

                let refs = manager
                    .search(extension, query.as_deref().unwrap_or_default(), filters, page)
                    .await?
                    .into_iter()
                    .map(|mut item| {
                        item.nsfw = Some(ext_nsfw || item.nsfw.unwrap_or(false));
                        CachedRef::Ext { item }
                    })
                    .collect();
                (extension.as_str(), refs)
            }
        };

        let now = Utc::now().timestamp();
        let soft_ttl = jittered(ttl, key, now);
        let cached = CachedPage { stale_at: now + soft_ttl, refs };

        let value = serde_json::to_value(&cached)
            .map_err(|_| CoreError::Internal("error.content.serialization".into()))?;
        CacheRepository::set(&state.pool, key, source, "discover", &value, soft_ttl + STALE_GRACE).await?;

        Ok(cached)
    }

    async fn resolve(
        state: &AppState,
        query: &DiscoverQuery,
        refs: Vec<CachedRef>,
        adult: bool,
    ) -> CoreResult<Vec<DiscoverItem>> {
        let pool = &state.pool;

        let ext_cids: HashMap<String, String> = match query {
            DiscoverQuery::Extension { extension, .. } => {
                let ids: Vec<&str> = refs.iter().filter_map(|r| match r {
                    CachedRef::Ext { item } => Some(item.id.as_str()),
                    CachedRef::Cid { .. } => None,
                }).collect();
                ExtensionRepository::find_cids_by_extension_ids(pool, extension, &ids).await?
            }
            DiscoverQuery::Tracker { .. } => HashMap::new(),
        };

        let cids: Vec<&str> = refs.iter().filter_map(|r| match r {
            CachedRef::Cid { cid } => Some(cid.as_str()),
            CachedRef::Ext { item } => ext_cids.get(&item.id).map(String::as_str),
        }).collect();
        let mut contents = ContentRepository::get_full_contents(pool, &cids, FullContentParts::SUMMARY).await?;

        let extension = match query {
            DiscoverQuery::Extension { extension, .. } => extension.as_str(),
            DiscoverQuery::Tracker { .. } => "",
        };

        let items = refs.into_iter().filter_map(|r| {
            let cid = match &r {
                CachedRef::Cid { cid } => Some(cid.as_str()),
                CachedRef::Ext { item } => ext_cids.get(&item.id).map(String::as_str),
            };
            if let Some(full) = cid.and_then(|cid| contents.remove(cid)) {
                return (adult || !full.content.nsfw).then(|| DiscoverItem::Content(full.slim_for_home()));
            }
            match r {
                CachedRef::Ext { item } if adult || !item.nsfw.unwrap_or(false) => {
                    Some(DiscoverItem::Extension { extension: extension.to_string(), item })
                }
                _ => None,
            }
        }).collect();

        Ok(items)
    }
}

fn cache_key(query: &DiscoverQuery, page: u32) -> String {
    let json = serde_json::to_string(query).unwrap_or_default();
    let hash = hex::encode(Sha256::digest(json.as_bytes()));
    format!("discover:{}:{page}", &hash[..24])
}

fn key_lock(key: &str) -> Arc<Mutex<()>> {
    KEY_LOCKS.entry(key.to_string()).or_default().clone()
}

/// ±15% so pages cached together don't all expire together.
fn jittered(ttl: i64, key: &str, now: i64) -> i64 {
    let seed = Sha256::digest(format!("{key}:{now}").as_bytes())[0] as f64 / 255.0;
    (ttl as f64 * (0.85 + seed * 0.3)) as i64
}
