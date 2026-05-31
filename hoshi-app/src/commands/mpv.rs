use hoshi_core::error::CoreError;
#[cfg(not(mobile))]
use hoshi_core::state::AppState;
use std::sync::Arc;
use tauri::State;
use hoshi_core::mpv::launch::{MpvLaunchOptions, MpvService};
use hoshi_core::mpv::service::MpvConfigService;
use crate::{require_auth, TauriSession};

#[cfg(not(mobile))]
#[tauri::command]
pub async fn launch_mpv(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    mut opts: MpvLaunchOptions,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;

    opts.state = Option::from(state.inner().clone());
    opts.user_id = user_id;

    let mpv = MpvService::launch(opts).await?;

    let mut guard = state.mpv.lock().unwrap();
    *guard = Some(mpv);

    Ok(())
}

#[cfg(not(mobile))]
#[tauri::command]
pub async fn is_mpv_running(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, CoreError> {
    let mut guard = state.mpv.lock().unwrap();

    let mpv = guard
        .as_mut()
        .ok_or(CoreError::Internal("".into()))?;

    Ok(mpv.is_running())
}

#[cfg(not(mobile))]
#[tauri::command]
pub async fn download_osc(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    name: String,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    MpvConfigService::download_osc(&state, user_id, &name).await
}

#[cfg(not(mobile))]
#[tauri::command]
pub async fn download_known_script(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    name: String,
) -> Result<String, CoreError> {
    use hoshi_core::mpv::registry::SCRIPT_REGISTRY;
    let user_id = require_auth(&session_state).await?;

    let script = SCRIPT_REGISTRY.iter()
        .find(|s| s.name == name)
        .ok_or_else(|| CoreError::BadRequest("error.mpv.unknown_script".into()))?;

    let filename = MpvConfigService::download_script(&state, script.url).await?;

    let mut config = hoshi_core::config::service::ConfigService::get_config(&state, user_id).await?;
    config.mpv.enabled_scripts.push(filename.clone());
    MpvConfigService::apply(&state.paths, &config.mpv)?;

    Ok(filename)
}