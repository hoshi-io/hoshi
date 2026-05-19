use serde_json::Value;
use tracing::{info, instrument, warn};

use crate::config::model::UserConfig;
use crate::config::repository::ConfigRepository;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

pub struct ConfigService;

impl ConfigService {
    #[instrument(skip(state))]
    pub async fn get_config(state: &AppState, user_id: i32) -> CoreResult<UserConfig> {
        ConfigRepository::get_config(&state.pool, user_id).await
    }

    #[instrument(skip(state, patch))]
    pub async fn patch_config(state: &AppState, user_id: i32, patch: Value) -> CoreResult<UserConfig> {
        if !patch.is_object() {
            warn!("Patch is not a JSON object");
            return Err(CoreError::BadRequest("error.config.invalid_patch_format".into()));
        }

        let new_config = ConfigRepository::patch_config(&state.pool, user_id, &patch).await?;
        info!("User configuration updated");
        Ok(new_config)
    }
}
