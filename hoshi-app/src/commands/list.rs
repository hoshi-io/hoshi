use hoshi_core::{
    error::CoreError,
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use hoshi_core::list::service::ListService;
use hoshi_core::list::sync_queue::SyncQueue;
use hoshi_core::list::types::{BatchHandle, BatchIssue, BatchStatus, BatchWriteResponse, EntryHistoryResponse, FilterQuery, ListResponse, SingleEntryResponse, SyncStatus, UpsertEntryBody, UpsertEntryResponse, UserStats};
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

#[tauri::command]
pub async fn batch_upsert_entries(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    entries: Vec<UpsertEntryBody>,
) -> Result<BatchWriteResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::batch_upsert_entries(state.inner().clone(), user_id, entries).await
}

#[tauri::command]
pub async fn batch_delete_entries(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cids: Vec<String>,
) -> Result<BatchWriteResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::batch_delete_entries(state.inner().clone(), user_id, cids).await
}

/// "On tracker X but not Y" button: push these entries to `tracker` (Y) only.
#[tauri::command]
pub async fn push_to_tracker(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker: String,
    cids: Vec<String>,
) -> Result<BatchHandle, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::push_to_tracker(&state.pool, user_id, &tracker, &cids).await
}

#[tauri::command]
pub async fn get_batch_status(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    batch_id: String,
) -> Result<BatchStatus, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::batch_status(&state.pool, user_id, &batch_id).await
}

/// Jobs of a batch that ended FAILED or SKIPPED (with reasons), for the sync panel.
#[tauri::command]
pub async fn get_batch_issues(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    batch_id: String,
) -> Result<Vec<BatchIssue>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::batch_issues(&state.pool, user_id, &batch_id).await
}

/// "Retry now" for a paused lane (offline / auth expired / rate limited).
#[tauri::command]
pub async fn resume_sync_lane(
    session_state: State<'_, TauriSession>,
    tracker: String,
) -> Result<(), CoreError> {
    require_auth(&session_state).await?;
    SyncQueue::resume_lane(&tracker);
    Ok(())
}

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<SyncStatus, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::sync_status(&state.pool, user_id).await
}

#[tauri::command]
pub async fn retry_failed_syncs(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    batch_id: Option<String>,
) -> Result<usize, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::retry_failed(&state.pool, user_id, batch_id.as_deref()).await
}

#[tauri::command]
pub async fn cancel_batch(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    batch_id: String,
) -> Result<usize, CoreError> {
    let user_id = require_auth(&session_state).await?;
    SyncQueue::cancel_batch(&state.pool, user_id, &batch_id).await
}