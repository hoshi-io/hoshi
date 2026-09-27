use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use crate::content::models::{ExtensionSource, FullContent};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::services::extensions::ExtensionService;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::TrackerMapping;
use sqlx::SqlitePool;

pub struct MappingService;

impl MappingService {
    #[instrument(skip(pool, mapping))]
    pub async fn add_tracker_mapping(pool: &SqlitePool, mut mapping: TrackerMapping) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        mapping.created_at = now;
        mapping.updated_at = now;

        TrackerRepository::add_mapping(pool, &mapping).await?;
        debug!(cid = %mapping.cid, tracker = %mapping.tracker_name, "Tracker mapping added successfully");
        Ok(())
    }

    #[instrument(skip(state))]
    pub async fn update_tracker_mapping(
        state: &Arc<AppState>,
        cid: &str,
        tracker_name: &str,
        tracker_id: &str,
    ) -> CoreResult<()> {
        let now = chrono::Utc::now().timestamp();
        TrackerRepository::add_mapping(&state.pool, &TrackerMapping {
            cid:          cid.to_string(),
            tracker_name: tracker_name.to_string(),
            tracker_id:   tracker_id.to_string(),
            tracker_url:  None,
            created_at:   now,
            updated_at:   now,
        }).await?;

        info!(cid = %cid, tracker = %tracker_name, "Tracker mapping updated");
        Ok(())
    }

    #[instrument(skip(state))]
    pub async fn delete_tracker_mapping(state: &Arc<AppState>, cid: &str, tracker_name: &str) -> CoreResult<()> {
        let rows = TrackerRepository::delete_mapping(&state.pool, cid, tracker_name).await?;
        if rows == 0 {
            warn!(cid = %cid, tracker = %tracker_name, "Delete failed: Mapping not found");
            return Err(CoreError::NotFound("error.content.mapping_not_found".into()));
        }

        info!(cid = %cid, tracker = %tracker_name, "Tracker mapping deleted");
        Ok(())
    }

    #[instrument(skip(state, source))]
    pub async fn add_extension_mapping(state: &Arc<AppState>, mut source: ExtensionSource) -> CoreResult<i64> {
        let now = chrono::Utc::now().timestamp();

        source.created_at = now;
        source.updated_at = now;

        let id = ExtensionRepository::add_source(&state.pool, &source).await?;
        info!(cid = %source.cid, ext = %source.extension_name, "Extension source added");
        Ok(id)
    }

    #[instrument(skip(state))]
    pub async fn update_extension_mapping(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
    ) -> CoreResult<FullContent> {
        let ext_nsfw = state.extension_manager.read().await
            .list_extensions().iter()
            .find(|e| e.id == ext_name)
            .map(|e| e.nsfw)
            .unwrap_or(false);

        let now = chrono::Utc::now().timestamp();

        if let Some(id) = ExtensionRepository::find_mapping_id(&state.pool, cid, ext_name).await? {
            ExtensionRepository::update_source(&state.pool, id, ext_id).await?;
        } else {
            ExtensionRepository::add_source(&state.pool, &ExtensionSource {
                id: None, cid: cid.to_string(), extension_name: ext_name.to_string(),
                extension_id: ext_id.to_string(), nsfw: ext_nsfw,
                language: None, created_at: now, updated_at: now,
            }).await?;
        }

        ExtensionService::save_extension_metadata(state, cid, ext_name, ext_id).await;

        ContentRepository::get_full_content(&state.pool, cid).await?
            .ok_or_else(|| CoreError::NotFound("error.content.not_found".into()))
    }
}