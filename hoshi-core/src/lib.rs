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
pub mod home;
pub mod config;
pub mod headless;
pub mod proxy;
pub mod progress;
pub mod discord;
pub mod logs;
pub mod diagnostics;
pub mod player;
pub mod torrent;

use crate::error::CoreResult;
use headless::HeadlessHandle;
pub use state::AppState;
use paths::AppPaths;
use std::sync::{Arc};
use std::time::Duration;
use reqwest::Client;
use tracker::provider::build_registry;
use tracing::{info, instrument, warn};
use crate::list::sync_queue::SyncQueue;
use crate::player::playback::PlaybackHandle;
use crate::player::thumbnail::ThumbnailService;
use crate::torrent::TorrentHandle;
use crate::tracker::sync::StartupSyncService;
use crate::home::service::HomeLayoutService;
use crate::content::repositories::cache::CacheRepository;

#[instrument(skip(log_store, paths, headless))]
pub async fn build(paths: AppPaths, headless: HeadlessHandle, log_store: logs::LogStore, ) -> CoreResult<Arc<AppState>> {
    info!("Starting Hoshi Core initialization...");
    let _ = std::fs::remove_dir_all(&paths.thumbnails);
    paths.ensure_dirs().map_err(|e| core_err!(Internal, "error.system.setup_failed", e))?;

    info!("Initializing unified database...");
    let db_manager = db::DatabaseManager::new(&paths).await?;

    let http_client = Client::builder()
        .read_timeout(Duration::from_secs(45))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10);
    // Android's verifier (rustls-platform-verifier, reqwest 0.13's default) reports any revocation
    // check failure as "revoked", and some builds fail it for certs without OCSP (Let's Encrypt),
    // so no request worked there. Verify against the bundled Mozilla roots instead, like 1.2.4.
    #[cfg(target_os = "android")]
    let http_client = http_client.tls_certs_only(
        webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .filter_map(|der| reqwest::Certificate::from_der(der).ok()),
    );
    let http_client = http_client
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

    let thumbnails = ThumbnailService::new(paths.thumbnails.clone());

    let state = Arc::new(AppState {
        pool: db_manager.pool().clone(),
        extension_manager: Arc::new(extension_manager),
        tracker_registry: Arc::new(build_registry(http_client.clone())),
        torrent: TorrentHandle::new(paths.torrent.clone()),
        paths: Arc::new(paths),
        playback: PlaybackHandle::new(),
        thumbnails,
        log_store,
        http_client,

        #[cfg(feature = "discord-rpc")]
        discord_rpc,
    });

    state.playback.add_protocol_hook({
        let torrent = state.torrent.clone();
        move |mpv| torrent::mpv_protocol::register(mpv, torrent.clone())
    });
    state.thumbnails.add_protocol_hook({
        let torrent = state.torrent.clone();
        move |mpv| torrent::mpv_protocol::register_thumb(mpv, torrent.clone())
    });
    state.torrent.spawn_reaper();

    HomeLayoutService::warmup(state.clone());
    tokio::spawn({
        let pool = state.pool.clone();
        async move {
            loop {
                if let Err(e) = CacheRepository::cleanup(&pool).await {
                    warn!(error = ?e, "Cache cleanup failed");
                }
                tokio::time::sleep(Duration::from_secs(24 * 3600)).await;
            }
        }
    });
    SyncQueue::start(state.clone()).await;
    StartupSyncService::run(state.clone());
    info!("Hoshi Core initialization completed successfully");
    Ok(state)
}