#[cfg(feature = "discord-rpc")]
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
#[cfg(feature = "discord-rpc")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "discord-rpc")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "discord-rpc")]
use tokio::sync::broadcast;
#[cfg(feature = "discord-rpc")]
use tracing::{warn, error, debug, instrument};

#[cfg(feature = "discord-rpc")]
use crate::config::repository::ConfigRepository;
#[cfg(feature = "discord-rpc")]
use crate::playback::PlaybackEvent;
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

/// Runs the Discord-RPC bridge loop, subscribing to `state.playback`'s mpv
/// event stream and driving Discord entirely from here — no frontend round
/// trip needed, since this has everything `set_activity` requires
/// (`AppState` for config lookup, `NowPlaying` for metadata,
/// `playback.duration()` for the timestamp math) without the frontend
/// resending anything per tick.
#[cfg(feature = "discord-rpc")]
pub async fn run_activity_bridge(state: Arc<AppState>) {
    let mut rx = state.playback.subscribe_events();

    let mut sent = false;

    loop {
        let event = match rx.recv().await {
            Ok(ev) => ev,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        };

        match event {
            PlaybackEvent::Position(pos) => {
                if sent {
                    continue;
                }
                let Some(np) = state.playback.now_playing() else {
                    continue;
                };
                let duration = state.playback.duration().await.unwrap_or(0.0);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let start = now - pos.floor() as i64;
                let end = (duration > 0.0).then(|| start + duration.floor() as i64);

                state
                    .discord_rpc
                    .set_activity(
                        &state,
                        np.user_id,
                        &np.title,
                        &np.episode_title,
                        np.cover_image.as_deref(),
                        Some(start),
                        end,
                        true,
                        np.nsfw,
                    )
                    .await;
                sent = true;
            }
            PlaybackEvent::PauseChanged(paused) => {
                let Some(np) = state.playback.now_playing() else {
                    continue;
                };
                if paused {
                    state
                        .discord_rpc
                        .set_activity(
                            &state,
                            np.user_id,
                            &np.title,
                            &np.episode_title,
                            np.cover_image.as_deref(),
                            None,
                            None,
                            true,
                            np.nsfw,
                        )
                        .await;
                } else {
                    sent = false;
                }
            }
            PlaybackEvent::Eof => {
                sent = false;
                state.discord_rpc.clear_activity();
            }
            PlaybackEvent::Error(_) => {
                sent = false;
                state.discord_rpc.clear_activity();
            }
            PlaybackEvent::Buffered(_) => {
                // Buffering progress doesn't affect Discord presence.
            }
        }
    }
}