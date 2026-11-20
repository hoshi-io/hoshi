use crate::{require_auth, TauriSession};
use hoshi_core::{
    config::types::UserConfig,
    config::service::ConfigService,
    error::CoreError,
    state::AppState,
};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_user_config(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserConfig, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ConfigService::get_config(&state, user_id).await
}

#[tauri::command]
pub async fn patch_user_config(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    patch: Value,
) -> Result<UserConfig, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ConfigService::patch_config(&state, user_id, patch).await
}