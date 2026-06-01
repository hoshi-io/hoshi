use hoshi_core::{
    error::CoreError,
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use hoshi_core::list::service::ListService;
use hoshi_core::list::types::{EntryHistoryResponse, FilterQuery, ListResponse, SingleEntryResponse, UpsertEntryBody, UpsertEntryResponse, UserStats};
use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn get_list(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: FilterQuery,
) -> Result<ListResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_list(&state, user_id, query).await
}

#[tauri::command]
pub async fn get_stats(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserStats, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_user_stats(&state, user_id).await
}

#[tauri::command]
pub async fn get_single_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<SingleEntryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_single_entry(&state, user_id, cid).await
}

#[tauri::command]
pub async fn upsert_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpsertEntryBody,
) -> Result<UpsertEntryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::upsert_entry(state.inner().clone(), user_id, body).await
}

#[tauri::command]
pub async fn delete_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::delete_entry(state.inner().clone(), user_id, cid).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_entry_history(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<EntryHistoryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_entry_history(&state, user_id, cid).await
}

#[tauri::command]
pub async fn get_activity_feed(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    limit: Option<i64>,
) -> Result<EntryHistoryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_activity_feed(&state, user_id, limit.unwrap_or(50)).await
}