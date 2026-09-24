use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::extensions::ExtensionManager;
use crate::logs::LogStore;
use crate::paths::AppPaths;
use crate::playback::PlaybackHandle;
use crate::tracker::provider::TrackerRegistry;

#[cfg(feature = "discord-rpc")]
use crate::discord::DiscordRpcService;

#[derive(Clone)]
pub struct AppState {
    pub pool:              SqlitePool,
    pub extension_manager: Arc<RwLock<ExtensionManager>>,
    pub tracker_registry:  Arc<TrackerRegistry>,
    pub paths:             Arc<AppPaths>,
    pub playback:          PlaybackHandle,
    pub log_store:         LogStore,
    pub http_client:       Client,

    #[cfg(feature = "discord-rpc")]
    pub discord_rpc: Arc<DiscordRpcService>,
}

impl AppState {
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}