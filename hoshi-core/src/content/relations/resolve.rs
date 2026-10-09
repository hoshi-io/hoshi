use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tracing::{debug, warn};

use crate::content::repositories::relations::RelationRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::content::services::resolver::ContentResolverService;
use crate::state::AppState;

/// Caps how many network resolves one operation may do.
pub struct ResolveBudget(AtomicUsize);

impl ResolveBudget {
    pub fn new(max: usize) -> Self {
        Self(AtomicUsize::new(max))
    }

    /// Returns true and consumes one unit if any are left.
    pub fn try_take(&self) -> bool {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1))
            .is_ok()
    }
}

/// Fetch the tracker entry, create the enriched content, backfill `target_cid`
/// on every relation pointing at it, and return the new cid.
pub async fn resolve_target(
    state: &Arc<AppState>,
    tracker_name: &str,
    tracker_id: &str,
) -> Option<String> {
    let media = match ContentResolverService::fetch_tracker_media(state, tracker_name, tracker_id).await {
        Ok(m) => m,
        Err(e) => {
            debug!("relation resolve: fetch {tracker_name}:{tracker_id} failed: {e:?}");
            return None;
        }
    };

    let full = match EnrichmentService::create_enriched_content(
        state,
        &media.content_type,
        &media,
        tracker_id,
        tracker_name,
        None,
    )
        .await
    {
        Ok(f) => f,
        Err(e) => {
            debug!("relation resolve: enrich {tracker_name}:{tracker_id} failed: {e:?}");
            return None;
        }
    };

    if let Err(e) = RelationRepository::backfill_target_cid(
        &state.pool,
        tracker_name,
        tracker_id,
        &full.content.cid,
    )
        .await
    {
        warn!("relation resolve: backfill {tracker_name}:{tracker_id} failed: {e:?}");
    }

    Some(full.content.cid)
}