use std::sync::Arc;
use tracing::{instrument, warn};
use crate::content::models::{FullContent, Metadata};
use crate::content::repositories::content::ContentRepository;
use crate::content::services::chinese_title::ChineseTitleService;
pub(crate) use crate::content::services::content_units::SimklUnitsService;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

pub struct ContentService;

impl ContentService {
    #[instrument(skip(state))]
    pub async fn get_content(
        state: &Arc<AppState>,
        source: &str,
        source_id: &str,
    ) -> CoreResult<FullContent> {
        let mut full = ContentResolverService::resolve_source(state, source, source_id).await?;
        Self::inject_chinese_titles(state, &mut full).await;
        Ok(full)
    }

    pub async fn get_content_by_cid(
        state: &Arc<AppState>,
        cid: &str,
    ) -> CoreResult<FullContent> {
        let mut full = ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))?;

        if let Err(e) = SimklUnitsService::sync_units_if_needed(state, cid).await {
            warn!(cid = %cid, error = ?e, "Failed to sync units synchronously");
        } else if let Some(refreshed) = ContentRepository::get_full_content(&state.pool, cid).await? {
            full = refreshed;
        }

        tokio::spawn(ContentResolverService::backfill_via_preferred_mapping(state.clone(), cid.to_string()));

        Self::inject_chinese_titles(state, &mut full).await;
        Ok(full)
    }

    async fn inject_chinese_titles(state: &Arc<AppState>, full: &mut FullContent) {
        let Some(mapping) = full.tracker_mappings.iter().find(|m| m.tracker_name == "anilist") else {
            return;
        };
        let tracker_id = mapping.tracker_id.clone();

        for meta in &mut full.metadata {
            ChineseTitleService::maybe_inject_chinese_title(&state.pool, &tracker_id, &mut meta.title_i18n).await;
        }
    }

    #[instrument(skip(state, meta))]
    pub async fn update_content(
        state: &Arc<AppState>,
        cid: &str,
        meta: Metadata,
    ) -> CoreResult<FullContent> {
        ContentRepository::upsert_metadata(&state.pool, &meta).await?;

        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| {
                warn!(cid = %cid, "Content not found after metadata update");
                CoreError::NotFound("error.content.not_found".into())
            })
    }

    pub async fn merge_content(
        state: &AppState,
        survivor_cid: &str,
        loser_cid: &str,
    ) -> CoreResult<FullContent> {
        ContentRepository::merge(&state.pool, survivor_cid, loser_cid).await?;

        ContentRepository::get_full_content(&state.pool, survivor_cid)
            .await?
            .ok_or_else(|| CoreError::NotFound("survivor cid not found after merge".into()))
    }
}