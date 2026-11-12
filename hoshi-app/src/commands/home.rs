use std::sync::Arc;
use tauri::State;
use hoshi_core::content::models::ContentType;
use hoshi_core::content::types::{DiscoverItem, DiscoverQuery};
use hoshi_core::error::CoreError;
use hoshi_core::home::service::HomeLayoutService;
use hoshi_core::home::types::{HomeLayout, SequelCard};
use hoshi_core::state::AppState;
use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn get_home_layout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<HomeLayout, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeLayoutService::get_layout(&state, user_id).await
}

#[tauri::command]
pub async fn get_default_home_layout() -> Result<HomeLayout, CoreError> {
    Ok(HomeLayoutService::default_layout())
}

#[tauri::command]
pub async fn save_home_layout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    layout: HomeLayout,
) -> Result<HomeLayout, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeLayoutService::save_layout(&state, user_id, layout).await
}

#[tauri::command]
pub async fn reset_home_layout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<HomeLayout, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeLayoutService::reset_layout(&state, user_id).await
}

#[tauri::command]
pub async fn discover(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: DiscoverQuery,
    page: Option<u32>,
) -> Result<Vec<DiscoverItem>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeLayoutService::discover(state.inner(), user_id, &query, page.unwrap_or(1)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_unstarted_sequels(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    content_type: ContentType,
    limit: Option<i64>,
) -> Result<Vec<SequelCard>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeLayoutService::unstarted_sequels(&state, user_id, content_type, limit.unwrap_or(20)).await
}
