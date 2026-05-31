pub mod state;
pub mod db;
pub mod extensions;
pub mod users;
pub mod auth;
pub mod list;
pub mod content;
pub mod tracker;
pub mod paths;
pub mod error;
pub mod schedule;
pub mod config;
pub mod headless;
pub mod proxy;
pub mod progress;
pub mod backup;
pub mod discord;
pub mod logs;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod mpv;

use crate::error::{CoreError, CoreResult};
use headless::HeadlessHandle;
use state::AppState;
use paths::AppPaths;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use reqwest::Client;
use tokio::sync::RwLock;
use tracker::provider::build_registry;
use tracing::{info, error, instrument};
use crate::content::services::home::HomeService;
use crate::tracker::sync::StartupSyncService;

#[instrument(skip(log_store, paths, headless))]
pub async fn build_app_state(
    paths: AppPaths,
    headless: HeadlessHandle,
    log_store: logs::LogStore,
) -> CoreResult<Arc<AppState>> {
    info!("Starting Hoshi Core initialization...");

    paths.ensure_dirs().map_err(|e| {
        error!("Failed to create application directories: {}", e);
        CoreError::Internal("error.system.setup_failed".into())
    })?;

    info!("Initializing unified database...");
    db::init_all_databases(&paths).await?;

    let db_manager = db::DatabaseManager::new(&paths).await?;
    let pool = db_manager.pool().clone();
    let db = Arc::new(db_manager);

    info!("Loading extensions from disk...");
    let mut extension_manager = extensions::ExtensionManager::new(&paths).map_err(|e| {
        error!("Failed to initialize extension manager: {}", e);
        CoreError::Internal("error.system.setup_failed".into())
    })?;

    extension_manager.load_extensions().await.map_err(|e| {
        error!("Failed to load extensions: {}", e);
        CoreError::Internal("error.system.setup_failed".into())
    })?;

    extension_manager.set_headless(headless.clone());
    let ext_manager_arc = Arc::new(RwLock::new(extension_manager));

    #[cfg(feature = "discord-rpc")]
    let discord_rpc = {
        info!("Initializing Discord Rich Presence...");
        Arc::new(crate::discord::DiscordRpcService::new("1486110945452228719"))
    };

    let http_client = Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build()
        .map_err(|e| CoreError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    info!("Building tracker registry");
    let tracker_registry = Arc::new(build_registry(http_client.clone()));

    let state = Arc::new(AppState {
        db,
        pool,
        extension_manager: ext_manager_arc,
        tracker_registry,
        paths: Arc::new(paths),
        headless,
        log_store,
        http_client,

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        mpv: Arc::new(Mutex::new(None)),

        #[cfg(feature = "discord-rpc")]
        discord_rpc,
    });

    HomeService::warmup(state.clone());
    StartupSyncService::run(state.clone());

    info!("Hoshi Core initialization completed successfully");
    Ok(state)
}

#[macro_export]
macro_rules! impl_from_row {
    ($struct:ty { $($field:ident),* $(,)? }) => {
        impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for $struct {
            fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(Self {
                    $($field: row.try_get(stringify!($field))?,)*
                })
            }
        }
    };
}