use crate::{require_auth, TauriSession};
use hoshi_core::tracker::types::{AddIntegrationRequest};
use hoshi_core::{
    error::CoreError,
    state::AppState,
    tracker::service::{TrackerService, import_from_tracker_by_name},
};
use std::sync::Arc;
use tauri::{State, AppHandle, Emitter};
use hoshi_core::tracker::types::{SuccessResponse, TrackerInfoResponse};

#[tauri::command]
pub async fn list_trackers(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<TrackerInfoResponse>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::list_trackers(&state, user_id).await
}

#[tauri::command]
pub async fn add_integration(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: AddIntegrationRequest,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;

    TrackerService::add_integration(Arc::clone(&state), user_id, body.clone()).await?;

    let state_clone = Arc::clone(&state);
    let tracker_name = body.tracker_name.clone();

    tokio::spawn(async move {
        let tracker_name_clone = tracker_name.clone();

        let result = import_from_tracker_by_name(
            &state_clone,
            user_id,
            &tracker_name,
            move |event| {
                let _ = app.emit("tracker:import", &event);
            },
        ).await;

        if let Err(e) = result {
            tracing::error!(
                error = ?e,
                tracker = %tracker_name_clone,
                "Initial import failed"
            );
        }
    });

    Ok(SuccessResponse { success: true })
}

#[tauri::command]
pub async fn remove_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::remove_integration(&state, user_id, &tracker_name).await
}

#[tauri::command]
pub async fn set_sync_enabled(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
    enabled: bool,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::set_sync_enabled(&state, user_id, &tracker_name, enabled).await
}