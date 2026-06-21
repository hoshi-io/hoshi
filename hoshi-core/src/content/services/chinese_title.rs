use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::{CoreError, CoreResult};

const CHINESE_TITLES_URL: &str =
    "https://raw.githubusercontent.com/soruly/anilist-chinese/refs/heads/master/anilist-chinese.json";

#[derive(Deserialize)]
struct RawEntry {
    id: u32,
    title: String,
}

type ChineseTitleMap = HashMap<u32, String>;

static CACHE: OnceLock<RwLock<Option<ChineseTitleMap>>> = OnceLock::new();

fn cache() -> &'static RwLock<Option<ChineseTitleMap>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

pub struct ChineseTitleService;

impl ChineseTitleService {
    /// Ensure the map is loaded. Cheap after the first call (read-lock, Some → return).
    pub async fn ensure_loaded() {
        if cache().read().await.is_some() {
            return;
        }

        let mut guard = cache().write().await;
        if guard.is_some() {
            return;
        }

        match Self::fetch().await {
            Ok(map) => {
                info!(entries = map.len(), "Chinese title map loaded into memory");
                *guard = Some(map);
            }
            Err(e) => {
                warn!(error = ?e, "Failed to load Chinese title map; Chinese titles will be skipped");
            }
        }
    }

    pub async fn lookup(anilist_id: u32) -> Option<String> {
        cache()
            .read()
            .await
            .as_ref()?
            .get(&anilist_id)
            .cloned()
    }

    pub async fn evict() {
        *cache().write().await = None;
    }

    async fn fetch() -> CoreResult<ChineseTitleMap> {
        info!("Fetching Chinese title map from upstream");

        let response = reqwest::get(CHINESE_TITLES_URL)
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?
            .error_for_status()
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let entries: Vec<RawEntry> = serde_json::from_slice(&bytes)
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let map = entries
            .into_iter()
            .filter(|e| !e.title.is_empty())
            .map(|e| (e.id, e.title))
            .collect();

        Ok(map)
    }
}