#[cfg(feature = "discord-rpc")]
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
#[cfg(feature = "discord-rpc")]
use std::sync::Mutex;
#[cfg(feature = "discord-rpc")]
use tracing::{warn, error, debug, instrument};

#[cfg(feature = "discord-rpc")]
use crate::config::repository::ConfigRepository;
#[cfg(feature = "discord-rpc")]
use crate::state::AppState;

#[cfg(feature = "discord-rpc")]
pub struct DiscordRpcService {
    client: Mutex<Option<DiscordIpcClient>>,
    client_id: String,
}

#[cfg(feature = "discord-rpc")]
impl DiscordRpcService {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: Mutex::new(None),
            client_id: client_id.to_string(),
        }
    }

    #[instrument(skip(self, state, details, image_url), fields(title = %title, is_video = %is_video))]
    pub async fn set_activity(
        &self,
        state: &AppState,
        user_id: i32,
        title: &str,
        details: &str,
        image_url: Option<&str>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        is_video: bool,
        is_nsfw: bool,
    ) {
        let user_config = ConfigRepository::get_config(&state.pool, user_id)
            .await
            .unwrap_or_default();

        let config = &user_config.discord;

        if !config.enabled {
            debug!("Discord RPC is disabled in user config, clearing activity");
            self.clear_activity();
            return;
        }

        let mut lock = self.client.lock().unwrap();

        if lock.is_none() {
            debug!("Initializing new Discord IPC client");
            let mut client = DiscordIpcClient::new(&self.client_id);
            if let Err(e) = client.connect() {
                warn!(error = ?e, "Failed to connect to Discord client");
            } else {
                *lock = Some(client);
            }
        }

        if let Some(client) = lock.as_mut() {
            let hide_content = !config.show_title || (config.hide_nsfw && is_nsfw);

            let (final_details, final_state, final_image, final_start, final_end) = if hide_content {
                debug!("Hiding content details due to user preferences or NSFW flag");
                ("In App", "Home", None, None, None)
            } else {
                (title, details, image_url, start_time, end_time)
            };

            let mut assets = activity::Assets::new();

            if final_details.eq_ignore_ascii_case("In App") {
                assets = assets.large_image("icon2");
            } else if let Some(url) = final_image {
                assets = assets.large_image(url);
            } else {
                assets = assets.large_image("icon2");
            }

            let activity_type = if is_video && !hide_content {
                activity::ActivityType::Watching
            } else {
                activity::ActivityType::Playing
            };

            let mut payload = activity::Activity::new()
                .activity_type(activity_type)
                .details(final_details)
                .state(final_state)
                .assets(assets);

            if final_start.is_some() || final_end.is_some() {
                let mut timestamps = activity::Timestamps::new();
                if let Some(s) = final_start { timestamps = timestamps.start(s); }
                if let Some(e) = final_end { timestamps = timestamps.end(e); }
                payload = payload.timestamps(timestamps);
            }

            if let Err(e) = client.set_activity(payload) {
                error!(error = ?e, "Failed to send activity payload to Discord RPC");
                *lock = None;
            } else {
                debug!("Discord RPC activity updated successfully");
            }
        }
    }

    #[instrument(skip(self))]
    pub fn clear_activity(&self) {
        let mut lock = self.client.lock().unwrap();
        if let Some(client) = lock.as_mut() {
            if let Err(e) = client.clear_activity() {
                warn!(error = ?e, "Failed to clear Discord activity");
            } else {
                debug!("Discord activity cleared");
            }
        }
    }
}