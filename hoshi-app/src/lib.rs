use tauri::{Manager, async_runtime};
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

pub mod proxy_protocol;

#[cfg(mobile)]
use crate::headless::headless_plugin::{init as headless_plugin_init, notify_done};

#[cfg(mobile)]
use crate::orientation::orientation_plugin::{
    init as orientation_plugin_init,
    lock_orientation,
    unlock_orientation,
    get_current_orientation,
};

#[cfg(mobile)]
use crate::intent::intent_plugin::{
    init as intent_plugin_init,
    launch_intent,
};

#[cfg(mobile)]
use crate::immersive::immersive_plugin::{
    init as immersive_plugin_init,
    enter_fullscreen,
    exit_fullscreen,
};



use crate::commands::i18n::load_locale;
use crate::commands::auth::{login, register, logout, get_current_profile};
use crate::commands::users::{get_all_users, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar};
use crate::commands::content::{get_trending, get_home_content, get_content, get_content_by_cid, update_content, search, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, search_extension};
use crate::commands::schedule::{get_schedule};
use crate::commands::list::{get_list, get_single_entry, upsert_entry, delete_entry, get_stats, get_entry_history, get_activity_feed};
use crate::commands::extensions::{get_image_request_headers, get_extensions, get_extension_filters, get_extension_settings, install_extension, install_tachiyomi_extension, install_lnreader_extension, update_extension, uninstall_extension, update_extension_settings};
use crate::commands::config::{get_user_config, patch_user_config};
use crate::commands::progress::{get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress};
use crate::commands::intergations::{list_trackers, add_integration, remove_integration, set_sync_enabled};
use crate::commands::logs::{get_system_logs, list_log_files, get_log_file, delete_log_file};

#[cfg(not(mobile))]
use crate::commands::mpv::{launch_mpv, is_mpv_running, download_osc, download_known_script};

#[cfg(feature = "discord-rpc")]
use crate::commands::discord::{set_activity, clear_activity};

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

pub fn run_inner() -> anyhow::Result<()> {
    let log_store = new_log_store();

    let memory_layer = MemoryLogLayer {
        store: log_store.clone(),
        limit: 1000,
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hoshi_app=debug,hoshi_core=debug,sandbox_js=debug".into()),
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
                .map_err(|e| anyhow::anyhow!("Failed to obtain app_data_dir: {}", e))?;

            let paths = hoshi_core::paths::AppPaths::from_base(base_dir);
            let headless = std::sync::Arc::new(headless::headless::TauriHeadless::new(app.handle().clone()));
            hoshi_core::logs::init_log_file(&paths.logs_path);

            #[cfg(any(windows, target_os = "linux"))]
            {
                app.deep_link().register_all().map_err(|e| {
                    error!("Failed to register deep link protocol: {}", e);
                    e
                })?;
            }

            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state(paths, headless, log_store).await
                    .map_err(|e| {
                        error!(error = ?e, "FATAL: Failed to build AppState during startup");
                        anyhow::anyhow!("AppState Error: {}", e)
                    })?;

                app.manage(state);
                app.manage(TauriSession::default());

                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_locale,
            get_system_logs, list_log_files, get_log_file, delete_log_file,
            login, register, logout,
            get_current_profile, get_all_users, get_me, update_me, delete_me, change_password, upload_avatar, delete_avatar,
            get_trending, get_home_content, get_content, get_content_by_cid, update_content, search, get_content_items, play_content_by_number, add_tracker_mapping, add_extension_source, update_extension_mapping, update_tracker_mapping, delete_tracker_mapping, search_extension,
            get_schedule,
            get_list, get_single_entry, upsert_entry, delete_entry, get_stats, get_entry_history, get_activity_feed,
            get_image_request_headers, get_extensions, get_extension_filters, get_extension_settings, install_extension, install_tachiyomi_extension, install_lnreader_extension, update_extension, uninstall_extension, update_extension_settings,
            get_user_config, patch_user_config,
            get_content_progress, get_continue_watching, update_anime_progress, update_chapter_progress,
            list_trackers, add_integration, remove_integration, set_sync_enabled,

            #[cfg(not(mobile))]
            launch_mpv,
            #[cfg(not(mobile))]
            is_mpv_running,
            #[cfg(not(mobile))]
            download_osc,
            #[cfg(not(mobile))]
            download_known_script,

            #[cfg(feature = "discord-rpc")]
            set_activity,
            #[cfg(feature = "discord-rpc")]
            clear_activity,
            #[cfg(mobile)]
            notify_done,
            #[cfg(mobile)]
            lock_orientation,
            #[cfg(mobile)]
            unlock_orientation,
            #[cfg(mobile)]
            get_current_orientation,
            #[cfg(mobile)]
            launch_intent,
            #[cfg(mobile)]
            enter_fullscreen,
            #[cfg(mobile)]
            exit_fullscreen,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri runtime error: {}", e))?;

    Ok(())
}

#[cfg(not(mobile))]
pub fn run() -> anyhow::Result<()> {
    run_inner()
}

#[cfg(mobile)]
#[tauri::mobile_entry_point]
pub fn run() {
    run_inner().expect("failed to run mobile app");
}