use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{error, instrument};
use crate::content::types::{parse_content_type, SearchParams};
use crate::content::utils::show_adult;
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::ExtensionSearchResult;
use crate::state::AppState;
use crate::tracker::provider::TrackerMedia;

pub struct SearchService;

impl SearchService {
    #[instrument(skip(state, params))]
    pub async fn search(
        state: &Arc<AppState>,
        params: SearchParams,
        user_id: i32,
    ) -> CoreResult<Vec<TrackerMedia>> {
        let show_adult = show_adult(state, user_id).await;

        let content_type = parse_content_type(
            params.r#type.as_deref().unwrap_or("anime")
        );

        let tracker_name = match params.tracker.as_deref().unwrap_or("anilist") {
            "mal"   => "mal",
            "kitsu" => "kitsu",
            _       => "anilist",
        };

        let provider = state.tracker_registry.get(tracker_name)
            .or_else(|| state.tracker_registry.get("anilist"))
            .ok_or_else(|| {
                error!(tracker = %tracker_name, "No suitable search provider found");
                CoreError::Internal("error.tracker.no_provider_available".into())
            })?;

        let limit = params.limit.unwrap_or(20) as usize;
        let page = {
            let offset = params.offset.unwrap_or(0) as usize;
            (offset / limit) + 1
        };

        let mut results = provider.search(
            params.query.as_deref(),
            content_type,
            limit,
            page,
            params.sort.as_deref(),
            params.genre.as_deref(),
            params.format.as_deref(),
            params.nsfw,
            params.status.as_deref(),
        ).await?;

        if !show_adult {
            results.retain(|media| !media.nsfw);
        }

        Ok(results)
    }

    pub async fn search_extension(
        state: &Arc<AppState>,
        ext_id: &str,
        query: Option<String>,
        filters_json: Option<String>,
        page: Option<u32>,
    ) -> CoreResult<Vec<ExtensionSearchResult>> {
        let filters: Value = filters_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(json!({}));

        let query_str = query.unwrap_or_default();
        let page = page.unwrap_or(1);

        let manager = state.extension_manager.read().await;
        let extension_is_nsfw = manager.is_nsfw(ext_id);

        let mut results = manager
            .search(ext_id, &query_str, filters, page)
            .await?;

        for item in &mut results {
            item.nsfw = Some(extension_is_nsfw || item.nsfw.unwrap_or(false));
        }

        Ok(results)
    }
}