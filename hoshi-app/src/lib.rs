use tauri::{Manager, Emitter, async_runtime};
use tauri_plugin_deep_link::DeepLinkExt;
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use hoshi_core::error::CoreError;
use tracing::{error};
use hoshi_core::logs::{new_log_store, MemoryLogLayer};

pub mod commands;
pub mod headless;
pub mod orientation;
pub mod intent;
pub mod immersive;
pub mod player_surface;

pub mod proxy_protocol;

#[cfg(mobile)]
use crate::{
    headless::headless_plugin::{init as headless_plugin_init},
    orientation::orientation_plugin::{init as orientation_plugin_init},
    intent::intent_plugin::{init as intent_plugin_init},
    immersive::immersive_plugin::{init as immersive_plugin_init}
};

#[derive(Default)]
pub struct TauriSession {
    pub user_id: RwLock<Option<String>>,
}

pub async fn require_auth(session_state: &TauriSession) -> Result<i32, CoreError> {
    let user = session_state.user_id.read().await;

    let uid_str = match &*user {
        Some(uid) => uid.clone(),
        None => return Err(CoreError::AuthError("error.auth.unauthorized".into())),
    };

    uid_str.parse::<i32>().map_err(|_| {
        CoreError::AuthError("error.auth.invalid_user_id".into())
    })
}

fn spawn_player_event_listener(app_handle: tauri::AppHandle, state: std::sync::Arc<hoshi_core::AppState>) {
    let mut rx = state.playback.subscribe_events();
    async_runtime::spawn(async move {
        loop {
            let event = match rx.recv().await {
                Ok(ev) => ev,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };

            let emit_result = match event {
                hoshi_core::playback::PlaybackEvent::Position(pos) => app_handle.emit("player://position", pos),
                hoshi_core::playback::PlaybackEvent::Eof => app_handle.emit("player://eof", ()),
                hoshi_core::playback::PlaybackEvent::PauseChanged(paused) => app_handle.emit("player://pause-changed", paused),
                hoshi_core::playback::PlaybackEvent::Error(msg) => app_handle.emit("player://error", msg),
            };

            if let Err(e) = emit_result {
                error!(error = ?e, "failed to emit player:// event to frontend");
            }
        }
    });
}

pub fn run_inner() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let log_store = new_log_store();

    let memory_layer = MemoryLogLayer {
        store: log_store.clone(),
        limit: 1000,
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hoshii_lib=debug,hoshi_core=debug,sandbox_js=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(memory_layer)
        .init();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(not(mobile))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }));
    }

    #[cfg(mobile)]
    {
        builder = builder
            .plugin(headless_plugin_init())
            .plugin(orientation_plugin_init())
            .plugin(immersive_plugin_init())
            .plugin(intent_plugin_init());
    }

    builder
        .register_asynchronous_uri_scheme_protocol("proxy", proxy_protocol::handle_async)
        .setup(move |app| {
            let base_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to obtain app_data_dir: {e}"))?;

            let paths = hoshi_core::paths::AppPaths::from_base(base_dir);
            let headless = std::sync::Arc::new(headless::headless::TauriHeadless::new(app.handle().clone()));
            hoshi_core::logs::init_log_file(&paths.logs_path);

            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link().register_all()?;
            }

            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state(paths, headless, log_store).await
                    .map_err(|e| {
                        error!(error = ?e, "FATAL: Failed to build AppState during startup");
                        e
                    })?;

                app.manage(state);
                app.manage(TauriSession::default());

                Ok::<(), CoreError>(())
            })?;

            // Forwards core's mpv event stream (position/pause/eof) to the
            // frontend as player:// Tauri events. Safe to spawn immediately
            {
                spawn_player_event_listener(app.handle().clone(), app.state::<std::sync::Arc<hoshi_core::AppState>>().inner().clone())
            }

            // Discord RPC is driven entirely off the same core event stream
            // from here on
            #[cfg(feature = "discord-rpc")]
            {
                let state = app.state::<std::sync::Arc<hoshi_core::AppState>>().inner().clone();
                async_runtime::spawn(hoshi_core::discord::run_activity_bridge(state));
            }

            // Needed for libmpv integration on linux webview
            #[cfg(target_os = "linux")]
            {
                let state = app.state::<std::sync::Arc<hoshi_core::AppState>>().inner().clone();
                let window = app
                    .get_webview_window("main")
                    .ok_or("main window not found during setup")?;
                player_surface::attach(&window, state)?;
            }

            Ok(())
        })
        .invoke_handler(commands::generate_handlers())
        .run(tauri::generate_context!())
        .map_err(|e| format!("Tauri runtime error: {e}"))?;

    Ok(())
}

#[cfg(not(mobile))]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_inner()
}

#[cfg(mobile)]
#[tauri::mobile_entry_point]
pub fn run() {
    run_inner().expect("failed to run mobile app");
}