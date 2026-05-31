use std::sync::{Arc, Mutex};
use reqwest::Client;
use tokio::sync::RwLock;
use sqlx::SqlitePool;

use crate::db::DatabaseManager;
use crate::extensions::ExtensionManager;
use crate::headless::HeadlessHandle;
use crate::paths::AppPaths;
use crate::tracker::provider::TrackerRegistry;
use crate::logs::LogStore;

#[cfg(feature = "discord-rpc")]
use crate::discord::DiscordRpcService;
use crate::mpv::launch::MpvService;
#[cfg(not(any(target_os = "android", target_os = "ios")))]

#[derive(Clone)]
pub struct AppState {
    pub db:                Arc<DatabaseManager>,
    pub pool:              SqlitePool,
    pub extension_manager: Arc<RwLock<ExtensionManager>>,
    pub tracker_registry:  Arc<TrackerRegistry>,
    pub paths:             Arc<AppPaths>,
    pub headless:          HeadlessHandle,
    pub log_store:         LogStore,
    pub http_client:       Client,

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub mpv: Arc<Mutex<Option<MpvService>>>,

    #[cfg(feature = "discord-rpc")]
    pub discord_rpc: Arc<DiscordRpcService>,
}

impl AppState {
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}