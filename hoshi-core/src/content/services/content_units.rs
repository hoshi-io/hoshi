use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::content::models::{ContentUnit, Status};
use crate::content::repositories::content::ContentRepository;
use crate::content::repositories::unit::UnitRepository;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;
use crate::tracker::repository::TrackerRepository;

const SIMKL_CLIENT_ID: &str = "d8385263a0cd0e60acd779d9db61310f41c8f99e40571af596ef79c7de1d4b2e";
const AIRING_SYNC_INTERVAL_SECS: i64 = 6 * 60 * 60;

pub struct SimklUnitsService;

impl SimklUnitsService {

    #[instrument(skip(state))]
    pub async fn sync_units_if_needed(state: &Arc<AppState>, cid: &str) -> CoreResult<()> {
        let simkl_id = match TrackerRepository::find_tracker_id_by_cid(&state.pool, cid, "simkl").await? {
            Some(id) => id,
            None => return Ok(()),
        };

        let units = UnitRepository::get_by_cid(&state.pool, cid).await?;
        let has_units = !units.is_empty();

        if !has_units {
            info!(cid = %cid, simkl_id = %simkl_id, "No units cached, fetching from Simkl");
            return Self::fetch_and_persist(state, cid, &simkl_id).await;
        }

        let is_airing = ContentRepository::get_full_content(&state.pool, cid).await?
            .and_then(|c| c.metadata.first().map(|m| matches!(m.status, Some(Status::Ongoing))))
            .unwrap_or(false);

        if !is_airing {
            return Ok(());
        }

        let last_synced = UnitRepository::last_synced_at(&state.pool, cid).await?.unwrap_or(0);
        let now = chrono::Utc::now().timestamp();

        if now - last_synced < AIRING_SYNC_INTERVAL_SECS {
            return Ok(());
        }

        info!(cid = %cid, simkl_id = %simkl_id, "Airing series due for re-sync");
        Self::fetch_and_persist(state, cid, &simkl_id).await
    }

    #[instrument(skip(state))]
    async fn fetch_and_persist(state: &Arc<AppState>, cid: &str, simkl_id: &str) -> CoreResult<()> {
        let url = format!(
            "https://api.simkl.com/anime/episodes/{}?client_id={}",
            simkl_id, SIMKL_CLIENT_ID
        );

        let response = state
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Network(format!("Simkl request failed: {e}")))?;

        if !response.status().is_success() {
            warn!(cid = %cid, status = %response.status(), "Simkl returned non-2xx, skipping unit sync");
            return Ok(());
        }

        let episodes: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| CoreError::Parse(format!("Simkl response parse error: {e}")))?;

        let now = chrono::Utc::now().timestamp();
        let mut upserted = 0usize;

        for ep in &episodes {
            let unit_type   = ep.get("type").and_then(|v| v.as_str()).unwrap_or("episode");
            let unit_number = ep.get("episode").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let title       = ep.get("title").and_then(|v| v.as_str()).map(str::to_owned);
            let description = ep.get("description").and_then(|v| v.as_str()).map(str::to_owned);
            let released_at = ep.get("date").and_then(|v| v.as_str()).map(str::to_owned);
            let thumbnail_url = ep
                .get("img")
                .and_then(|v| v.as_str())
                .map(|img| format!("https://simkl.in/episodes/{}_m.jpg", img));

            let unit = ContentUnit {
                id: None,
                cid: cid.to_owned(),
                unit_number,
                content_type: unit_type.to_owned(),
                title,
                description,
                thumbnail_url,
                released_at,
                duration: None,
                absolute_number: None,
                created_at: now,
            };

            UnitRepository::upsert(&state.pool, &unit).await?;
            upserted += 1;
        }

        info!(cid = %cid, count = upserted, "Simkl unit sync complete");
        Ok(())
    }
}