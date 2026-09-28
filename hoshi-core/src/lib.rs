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
pub mod discord;
pub mod logs;
pub mod playback;
pub mod torrent;

use crate::error::CoreResult;
use headless::HeadlessHandle;
pub use state::AppState;
use paths::AppPaths;
use std::sync::{Arc};
use std::time::Duration;
use reqwest::Client;
use tokio::sync::RwLock;
use tracker::provider::build_registry;
use tracing::{info, instrument};
use crate::content::services::home::HomeService;
use crate::playback::PlaybackHandle;
use crate::torrent::TorrentHandle;
use crate::tracker::sync::StartupSyncService;

#[instrument(skip(log_store, paths, headless))]
pub async fn build_app_state(paths: AppPaths, headless: HeadlessHandle, log_store: logs::LogStore, ) -> CoreResult<Arc<AppState>> {
    info!("Starting Hoshi Core initialization...");
    paths.ensure_dirs().map_err(|e| core_err!(Internal, "error.system.setup_failed", e))?;

    info!("Initializing unified database...");
    let db_manager = db::DatabaseManager::new(&paths).await?;

    let http_client = Client::builder()
        .timeout(Duration::from_secs(45))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build()
        .map_err(|e| core_err!(Internal, "error.system.http_client_failed", e))?;

    info!("Loading extensions from disk...");
    let mut extension_manager = extensions::ExtensionManager::new(&paths, http_client.clone())
        .map_err(|e| core_err!(Internal, "error.system.setup_failed", e))?;

    extension_manager.load_extensions().await
        .map_err(|e| core_err!(Internal, "error.system.setup_failed", e))?;

    extension_manager.set_headless(headless.clone());

    #[cfg(feature = "discord-rpc")]
    let discord_rpc = {
        info!("Initializing Discord Rich Presence...");
        Arc::new(crate::discord::DiscordRpcService::new("1486110945452228719"))
    };

    let state = Arc::new(AppState {
        pool: db_manager.pool().clone(),
        extension_manager: Arc::new(RwLock::new(extension_manager)),
        tracker_registry: Arc::new(build_registry(http_client.clone())),
        torrent: TorrentHandle::new(paths.torrent.clone()),
        paths: Arc::new(paths),
        playback: PlaybackHandle::new(),
        log_store,
        http_client,

        #[cfg(feature = "discord-rpc")]
        discord_rpc,
    });

    state.playback.add_protocol_hook({
        let torrent = state.torrent.clone();
        move |mpv| torrent::mpv_protocol::register(mpv, torrent.clone())
    });
    state.torrent.spawn_reaper();

    HomeService::warmup(state.clone());
    StartupSyncService::run(state.clone());
    info!("Hoshi Core initialization completed successfully");
    Ok(state)
}