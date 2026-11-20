//! iOS twin of `android.rs`: PlayerSurfacePlugin.swift hands us a CAMetalLayer and
//! mpv renders into it through MoltenVK (`gpu-context=moltenvk`, `wid` = layer pointer).

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager, Wry};
use tracing::{error, info, warn};

use hoshi_core::player::playback::RenderTarget;

const GPU_CONTEXT: &str = "moltenvk";

static APP_HANDLE: OnceLock<AppHandle<Wry>> = OnceLock::new();

static PENDING_LAYER: Mutex<Option<i64>> = Mutex::new(None);

pub fn attach(app: &AppHandle<Wry>) {
    if APP_HANDLE.set(app.clone()).is_err() {
        warn!("ios player_surface::attach called more than once, ignoring");
        return;
    }
    app.manage(crate::player_surface::PlayerReadyNotify(std::sync::Arc::new(
        tokio::sync::Notify::new(),
    )));
}

fn app_state() -> Option<std::sync::Arc<hoshi_core::AppState>> {
    APP_HANDLE
        .get()
        .and_then(|app| app.try_state::<std::sync::Arc<hoshi_core::AppState>>())
        .map(|s| s.inner().clone())
}

fn notify_ready() {
    if let Some(app) = APP_HANDLE.get() {
        app.state::<crate::player_surface::PlayerReadyNotify>().0.notify_one();
    }
}

fn hand_to_mpv(state: std::sync::Arc<hoshi_core::AppState>, wid: i64) {
    tauri::async_runtime::spawn(async move {
        let result = if state.playback.mpv_handle().await.is_ok() {
            state.playback.attach_surface(wid).await
        } else {
            state
                .playback
                .initialize(RenderTarget::NativeWindow { wid, gpu_context: GPU_CONTEXT })
                .await
        };

        if let Err(e) = &result {
            error!(?e, "failed to attach CAMetalLayer to mpv");
        }
        // Wake waiters either way so a failure reaches the frontend instead of hanging it.
        notify_ready();
    });
}

pub fn flush_pending_surface() {
    let Some(wid) = PENDING_LAYER.lock().unwrap().take() else {
        return;
    };
    let Some(state) = app_state() else {
        *PENDING_LAYER.lock().unwrap() = Some(wid);
        return;
    };
    info!(wid, "flushing CAMetalLayer queued before AppState was ready");
    hand_to_mpv(state, wid);
}

/// Called from Swift on the main thread. The layer stays alive (owned by its view)
/// until `hoshi_player_surface_destroyed`.
#[unsafe(no_mangle)]
pub extern "C" fn hoshi_player_surface_created(layer: *mut c_void) {
    let wid = layer as usize as i64;
    let Some(state) = app_state() else {
        warn!(wid, "surface created before AppState was ready, queuing");
        *PENDING_LAYER.lock().unwrap() = Some(wid);
        return;
    };
    info!(wid, "handing CAMetalLayer to mpv");
    hand_to_mpv(state, wid);
}

#[unsafe(no_mangle)]
pub extern "C" fn hoshi_player_surface_destroyed() {
    PENDING_LAYER.lock().unwrap().take();
    let Some(state) = app_state() else { return };
    tauri::async_runtime::spawn(async move {
        if state.playback.mpv_handle().await.is_err() {
            return;
        }
        if let Err(e) = state.playback.detach_surface().await {
            error!(?e, "failed to detach CAMetalLayer from mpv");
        }
    });
}
