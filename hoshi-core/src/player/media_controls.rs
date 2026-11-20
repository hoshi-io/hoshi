//! OS media controls for the player: MPRIS on Linux, the media flyout (SMTC) on
//! Windows, Now Playing on macOS/iOS. A no-op on Android.

use std::ffi::c_void;
use std::sync::mpsc;
use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};
use tokio::sync::{broadcast, mpsc as tokio_mpsc};
use tracing::{debug, warn};

use super::playback::{PlaybackEvent, PlaybackHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaRequest {
    Next,
    Previous,
}

enum Update {
    Metadata { title: String, artist: String, cover: Option<String>, duration: Option<Duration> },
    Playback(MediaPlayback),
}

/// Seconds a bare "seek" key (no amount) jumps.
const SEEK_STEP: f64 = 10.0;
/// time-pos fires every frame; the OS only needs the position now and then
/// (it interpolates), plus right away after a seek.
const POSITION_RESYNC: f64 = 5.0;
const SEEK_THRESHOLD: f64 = 2.0;

/// Runs until the player's event stream closes. `hwnd` is required on Windows.
pub async fn run_media_controls(
    handle: PlaybackHandle,
    hwnd: Option<usize>,
    requests: broadcast::Sender<MediaRequest>,
) {
    let (update_tx, update_rx) = mpsc::channel::<Update>();
    let (command_tx, mut command_rx) = tokio_mpsc::unbounded_channel::<MediaControlEvent>();

    // souvlaki's handles aren't Send on every platform: they live on one thread.
    std::thread::Builder::new()
        .name("media-controls".into())
        .spawn(move || controls_thread(hwnd, update_rx, command_tx))
        .ok();

    let mut events = handle.subscribe_events();
    let mut tick = tokio::time::interval(Duration::from_secs(2));

    let mut current: Option<(String, i64)> = None;
    let mut paused = false;
    let mut last_sent_pos = f64::NEG_INFINITY;
    let mut last_pos = 0.0;

    let playback = |paused: bool, pos: f64| {
        let progress = Some(MediaPosition(Duration::from_secs_f64(pos.max(0.0))));
        if paused { MediaPlayback::Paused { progress } } else { MediaPlayback::Playing { progress } }
    };

    loop {
        tokio::select! {
            event = events.recv() => match event {
                Ok(PlaybackEvent::Position(pos)) => {
                    let seeked = (pos - last_pos).abs() > SEEK_THRESHOLD;
                    last_pos = pos;
                    if current.is_some() && (seeked || (pos - last_sent_pos).abs() >= POSITION_RESYNC) {
                        last_sent_pos = pos;
                        let _ = update_tx.send(Update::Playback(playback(paused, pos)));
                    }
                }
                Ok(PlaybackEvent::PauseChanged(p)) => {
                    paused = p;
                    if current.is_some() {
                        last_sent_pos = last_pos;
                        let _ = update_tx.send(Update::Playback(playback(paused, last_pos)));
                    }
                }
                Ok(PlaybackEvent::Eof) => {
                    let _ = update_tx.send(Update::Playback(MediaPlayback::Stopped));
                    current = None;
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
            Some(command) = command_rx.recv() => {
                if let Err(e) = handle_command(&handle, &requests, command, last_pos).await {
                    debug!(?e, "media control command failed");
                }
            }
            _ = tick.tick() => {
                // A new episode loaded or the player stopped: there's no event for either.
                let now = handle.now_playing();
                let key = now.as_ref().map(|np| (np.cid.clone(), np.episode));
                if key == current {
                    continue;
                }
                current = key;
                match now {
                    Some(np) => {
                        let duration = handle.duration().await.ok().filter(|d| *d > 0.0).map(Duration::from_secs_f64);
                        let title = if np.episode_title.is_empty() { format!("Episode {}", np.episode) } else { np.episode_title };
                        let _ = update_tx.send(Update::Metadata {
                            title,
                            artist: np.title,
                            cover: if np.nsfw { None } else { np.cover_image },
                            duration,
                        });
                        last_sent_pos = last_pos;
                        let _ = update_tx.send(Update::Playback(playback(paused, last_pos)));
                    }
                    None => {
                        let _ = update_tx.send(Update::Playback(MediaPlayback::Stopped));
                    }
                }
            }
        }
    }
}

async fn handle_command(
    handle: &PlaybackHandle,
    requests: &broadcast::Sender<MediaRequest>,
    command: MediaControlEvent,
    pos: f64,
) -> crate::error::CoreResult<()> {
    match command {
        MediaControlEvent::Play => handle.set_paused(false).await?,
        MediaControlEvent::Pause | MediaControlEvent::Stop => handle.set_paused(true).await?,
        MediaControlEvent::Toggle => { handle.toggle_pause().await?; }
        MediaControlEvent::Next => { let _ = requests.send(MediaRequest::Next); }
        MediaControlEvent::Previous => { let _ = requests.send(MediaRequest::Previous); }
        MediaControlEvent::Seek(dir) => handle.seek(signed(dir, SEEK_STEP), true).await?,
        MediaControlEvent::SeekBy(dir, by) => handle.seek(signed(dir, by.as_secs_f64()), true).await?,
        MediaControlEvent::SetPosition(MediaPosition(to)) => {
            if (to.as_secs_f64() - pos).abs() > 0.5 {
                handle.seek(to.as_secs_f64(), false).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn signed(dir: SeekDirection, secs: f64) -> f64 {
    match dir {
        SeekDirection::Forward => secs,
        SeekDirection::Backward => -secs,
    }
}

fn controls_thread(
    hwnd: Option<usize>,
    updates: mpsc::Receiver<Update>,
    commands: tokio_mpsc::UnboundedSender<MediaControlEvent>,
) {
    let config = PlatformConfig {
        display_name: "Hoshi",
        dbus_name: "hoshi",
        hwnd: hwnd.map(|h| h as *mut c_void),
    };
    let mut controls = match MediaControls::new(config) {
        Ok(c) => c,
        Err(e) => {
            warn!(?e, "OS media controls unavailable");
            return;
        }
    };
    if let Err(e) = controls.attach(move |event| {
        let _ = commands.send(event);
    }) {
        warn!(?e, "failed to attach OS media controls");
        return;
    }

    for update in updates {
        let result = match update {
            Update::Metadata { title, artist, cover, duration } => controls.set_metadata(MediaMetadata {
                title: Some(&title),
                artist: Some(&artist),
                album: Some(&artist),
                cover_url: cover.as_deref(),
                duration,
            }),
            Update::Playback(playback) => controls.set_playback(playback),
        };
        if let Err(e) = result {
            debug!(?e, "failed to update OS media controls");
        }
    }
}
