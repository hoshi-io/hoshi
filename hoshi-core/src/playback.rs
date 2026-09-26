//! Platform-agnostic playback core wrapping [`libmpv2::Mpv`].

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use libmpv2::{Mpv, MpvInitializer};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Notify, RwLock};
use tracing::{info, instrument, warn};

use crate::core_err;
use crate::error::CoreResult;

#[derive(Debug, Clone, Serialize)]
pub struct Chapter {
    pub index: i64,
    pub title: Option<String>,
    pub time: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Track {
    pub id: i64,
    pub track_type: String, // "audio" | "video" | "sub"
    pub selected: bool,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    /// For video tracks in an HLS master playlist, the variant's nominal bitrate (bps).
    pub bitrate: Option<i64>,
    pub w: Option<i64>,
    pub h: Option<i64>,
}

#[derive(Debug)]
pub enum LoadMode {
    Replace,
    Append,
    AppendPlay,
}

impl LoadMode {
    fn as_str(&self) -> &'static str {
        match self {
            LoadMode::Replace => "replace",
            LoadMode::Append => "append",
            LoadMode::AppendPlay => "append-play",
        }
    }
}

/// Everything needed to start (or queue) a file in mpv in one shot.
#[derive(Debug, Clone, Default)]
pub struct LoadSpec {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub subtitles: Vec<ExternalSubtitle>,
    pub chapters: Vec<EpisodeChapter>,
    pub start_position: Option<f64>,
    pub now_playing: Option<NowPlaying>,
}

#[derive(Debug, Clone)]
pub struct ExternalSubtitle {
    pub url: String,
    pub title: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeChapter {
    pub start: f64,
    pub end: f64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlaying {
    pub cid: String,
    pub episode: i64,
    pub title: String,
    pub episode_title: String,
    pub cover_image: Option<String>,
    pub nsfw: bool,
    pub total_episodes: i64,
    /// The user this session belongs to
    pub user_id: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PlaybackEvent {
    Position(f64),
    Buffered(f64),
    Eof,
    PauseChanged(bool),
    Error(String),
}

#[derive(Clone)]
pub struct PlaybackHandle {
    inner: Arc<RwLock<Option<Arc<Mpv>>>>,
    session: Arc<AtomicU64>,
    shutdown_notify: Arc<Notify>,
    chapters_file: Arc<Mutex<Option<PathBuf>>>,
    now_playing: Arc<Mutex<Option<NowPlaying>>>,
    events: broadcast::Sender<PlaybackEvent>,
}

impl PlaybackHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            session: Arc::new(AtomicU64::new(0)),
            shutdown_notify: Arc::new(Notify::new()),
            chapters_file: Arc::new(Mutex::new(None)),
            now_playing: Arc::new(Mutex::new(None)),
            events: broadcast::channel(64).0,
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<PlaybackEvent> {
        self.events.subscribe()
    }

    pub fn now_playing(&self) -> Option<NowPlaying> {
        self.now_playing.lock().unwrap().clone()
    }

    pub fn subscribe_shutdown(&self) -> Arc<Notify> {
        self.shutdown_notify.clone()
    }

    pub fn session(&self) -> u64 {
        self.session.load(Ordering::SeqCst)
    }

    /// Initializes the mpv core. Idempotent.
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> CoreResult<()> {
        let mut guard = self.inner.write().await;
        if guard.is_some() {
            warn!("playback core already initialized, ignoring duplicate initialize()");
            return Ok(());
        }

        let mpv = tokio::task::spawn_blocking(build_mpv)
            .await
            .map_err(|e| core_err!(Internal, "error.playback.init_failed", e))?
            .map_err(|e| core_err!(Internal, "error.playback.init_failed", e))?;

        let mpv = Arc::new(mpv);
        *guard = Some(mpv.clone());
        self.session.fetch_add(1, Ordering::SeqCst);

        spawn_event_loop(mpv, self.events.clone());

        info!("playback core initialized");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn stop(&self) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;

        self.cleanup_chapters_file();
        *self.now_playing.lock().unwrap() = None;

        self.run(move || -> libmpv2::Result<()> {
            mpv.command("stop", &[])?;
            mpv.set_property("vid", "no")
        })
            .await
    }

    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> CoreResult<()> {
        let mut guard = self.inner.write().await;
        let Some(mpv) = guard.take() else {
            return Ok(());
        };
        drop(guard);

        self.shutdown_notify.notify_waiters();

        tokio::task::spawn_blocking(move || drop(mpv))
            .await
            .map_err(|e| core_err!(Internal, "error.playback.shutdown_failed", e))?;

        self.cleanup_chapters_file();

        info!("playback core shut down");
        Ok(())
    }

    /// Loads a file/URL into mpv along with its headers, external subtitles,
    /// chapter markers and an optional start position, all in one `loadfile`
    /// call.
    #[instrument(skip(self, spec))]
    pub async fn load(&self, spec: LoadSpec, mode: LoadMode) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;

        let mut other_headers = Vec::with_capacity(spec.headers.len());
        let mut user_agent: Option<String> = None;
        for (k, v) in &spec.headers {
            if k.eq_ignore_ascii_case("user-agent") {
                user_agent = Some(v.clone());
            } else {
                other_headers.push((k.clone(), v.clone()));
            }
        }

        let header_value = other_headers
            .iter()
            .map(|(k, v)| mpv_list_escape(&format!("{k}: {v}")))
            .collect::<Vec<_>>()
            .join(",");
        {
            let mpv = mpv.clone();
            self.run(move || mpv.set_property("http-header-fields", header_value))
                .await?;
        }
        {
            let mpv = mpv.clone();
            let ua = user_agent.unwrap_or_default();
            self.run(move || mpv.set_property("user-agent", ua)).await?;
        }

        self.cleanup_chapters_file();

        *self.now_playing.lock().unwrap() = spec.now_playing.clone();

        let mut options: Vec<String> = Vec::new();

        if let Some(start) = spec.start_position {
            if start > 0.0 {
                options.push(format!("start={start}"));
            }
        }

        if !spec.chapters.is_empty() {
            let path = std::env::temp_dir().join(format!("hoshi-chapters-{}.ffmeta", self.session()));
            write_chapters_file(&spec.chapters, &path)
                .map_err(|e| core_err!(Internal, "error.playback.chapters_file_failed", e))?;
            options.push(format!("chapters-file={}", mpv_list_escape(&path.to_string_lossy())));
            *self.chapters_file.lock().unwrap() = Some(path);
        }

        let options_str = options.join(",");
        let mode_str = mode.as_str();
        let url = spec.url;
        let subtitles = spec.subtitles;

        self.run(move || -> libmpv2::Result<()> {
            mpv.set_property("vid", "auto")?;

            mpv.command("loadfile", &[&url, mode_str, "-1", &options_str])?;

            for sub in &subtitles {
                let title = sub.title.as_deref().unwrap_or("");
                let lang = sub.lang.as_deref().unwrap_or("");
                let _ = mpv.command("sub-add", &[&sub.url, "auto", title, lang]);
            }
            Ok(())
        })
            .await
    }

    /// Back-compat alias for a bare URL load with no headers/subs/chapters.
    pub async fn load_stream(&self, url: String) -> CoreResult<()> {
        self.load(LoadSpec { url, ..Default::default() }, LoadMode::Replace).await
    }

    #[instrument(skip(self))]
    pub async fn toggle_pause(&self) -> CoreResult<bool> {
        let mpv = self.require_mpv().await?;
        self.run(move || -> libmpv2::Result<bool> {
            let paused: bool = mpv.get_property("pause").unwrap_or(false);
            mpv.set_property("pause", !paused)?;
            Ok(!paused)
        })
            .await
    }

    #[instrument(skip(self))]
    pub async fn set_paused(&self, paused: bool) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.set_property("pause", paused)).await
    }

    // ---- volume / mute -------------------------------------------------

    #[instrument(skip(self))]
    pub async fn set_volume(&self, volume: f64) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.set_property("volume", volume)).await
    }

    #[instrument(skip(self))]
    pub async fn volume(&self) -> CoreResult<f64> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.get_property::<f64>("volume")).await
    }

    #[instrument(skip(self))]
    pub async fn set_muted(&self, muted: bool) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.set_property("mute", muted)).await
    }

    #[instrument(skip(self))]
    pub async fn muted(&self) -> CoreResult<bool> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.get_property::<bool>("mute")).await
    }

    // ---- seek / position / speed ----------------------------------------
    #[instrument(skip(self))]
    pub async fn seek(&self, target: f64, relative: bool) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        let mode = if relative { "relative" } else { "absolute" };
        let target_str = target.to_string();
        self.run(move || mpv.command("seek", &[&target_str, mode])).await
    }

    #[instrument(skip(self))]
    pub async fn position(&self) -> CoreResult<f64> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.get_property::<f64>("time-pos")).await
    }

    #[instrument(skip(self))]
    pub async fn duration(&self) -> CoreResult<f64> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.get_property::<f64>("duration")).await
    }

    #[instrument(skip(self))]
    pub async fn set_speed(&self, speed: f64) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.set_property("speed", speed)).await
    }

    // ---- chapters ---------------------------------------------------------

    #[instrument(skip(self))]
    pub async fn chapters(&self) -> CoreResult<Vec<Chapter>> {
        let mpv = self.require_mpv().await?;
        self.run(move || -> libmpv2::Result<Vec<Chapter>> {
            let count: i64 = mpv.get_property("chapter-list/count").unwrap_or(0);
            let mut chapters = Vec::with_capacity(count.max(0) as usize);
            for i in 0..count {
                let title: Option<String> =
                    mpv.get_property(&format!("chapter-list/{i}/title")).ok();
                let time: f64 = mpv
                    .get_property(&format!("chapter-list/{i}/time"))
                    .unwrap_or(0.0);
                chapters.push(Chapter { index: i, title, time });
            }
            Ok(chapters)
        })
            .await
    }

    /// Jumps to a chapter by index (as returned by `chapters()`).
    #[instrument(skip(self))]
    pub async fn set_chapter(&self, index: i64) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || mpv.set_property("chapter", index)).await
    }

    // ---- tracks (audio / video / subtitle) ---------------------------------

    #[instrument(skip(self))]
    pub async fn tracks(&self) -> CoreResult<Vec<Track>> {
        let mpv = self.require_mpv().await?;
        self.run(move || -> libmpv2::Result<Vec<Track>> {
            let count: i64 = mpv.get_property("track-list/count").unwrap_or(0);
            let mut tracks = Vec::with_capacity(count.max(0) as usize);
            for i in 0..count {
                let p = |suffix: &str| format!("track-list/{i}/{suffix}");
                let id: i64 = mpv.get_property(&p("id")).unwrap_or(-1);
                let track_type: String = mpv.get_property(&p("type")).unwrap_or_default();
                let selected: bool = mpv.get_property(&p("selected")).unwrap_or(false);
                let title: Option<String> = mpv.get_property(&p("title")).ok();
                let lang: Option<String> = mpv.get_property(&p("lang")).ok();
                let codec: Option<String> = mpv.get_property(&p("codec")).ok();
                let bitrate: Option<i64> = mpv.get_property(&p("demux-bitrate")).ok();
                let w: Option<i64> = mpv.get_property(&p("demux-w")).ok();
                let h: Option<i64> = mpv.get_property(&p("demux-h")).ok();
                tracks.push(Track { id, track_type, selected, title, lang, codec, bitrate, w, h });
            }
            Ok(tracks)
        })
            .await
    }

    /// Selects an audio track by mpv track id (see `tracks()`). `None` disables audio.
    pub async fn set_audio_track(&self, id: Option<i64>) -> CoreResult<()> {
        self.set_track_property("aid", id).await
    }

    pub async fn set_video_track(&self, id: Option<i64>) -> CoreResult<()> {
        self.set_track_property("vid", id).await
    }

    pub async fn set_subtitle_track(&self, id: Option<i64>) -> CoreResult<()> {
        self.set_track_property("sid", id).await
    }

    async fn set_track_property(&self, prop: &'static str, id: Option<i64>) -> CoreResult<()> {
        let mpv = self.require_mpv().await?;
        self.run(move || match id {
            Some(id) => mpv.set_property(prop, id),
            None => mpv.set_property(prop, "no"),
        })
            .await
    }

    // ---- internals ----------------------------------------------------

    pub async fn mpv_handle(&self) -> CoreResult<Arc<Mpv>> {
        self.require_mpv().await
    }

    async fn require_mpv(&self) -> CoreResult<Arc<Mpv>> {
        self.inner
            .read()
            .await
            .clone()
            .ok_or_else(|| core_err!(Internal, "error.playback.not_initialized"))
    }

    fn cleanup_chapters_file(&self) {
        if let Some(old) = self.chapters_file.lock().unwrap().take() {
            let _ = std::fs::remove_file(old);
        }
    }

    /// Runs a blocking mpv call on a blocking thread and flattens both
    /// error layers into `CoreResult`.
    async fn run<F, T, E>(&self, f: F) -> CoreResult<T>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: std::fmt::Debug + Send + 'static,
    {
        tokio::task::spawn_blocking(f)
            .await
            .map_err(|e| core_err!(Internal, "error.playback.command_failed", e))?
            .map_err(|e| core_err!(Internal, "error.playback.command_failed", e))
    }
}

impl Default for PlaybackHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Observes mpv's own event loop for the properties/events we care about
/// and rebroadcasts them as [`PlaybackEvent`]s.
fn spawn_event_loop(mpv: Arc<Mpv>, events: broadcast::Sender<PlaybackEvent>) {
    std::thread::spawn(move || {
        if let Err(e) = mpv.observe_property("time-pos", libmpv2::Format::Double, 1) {
            warn!(?e, "failed to observe time-pos, position events disabled");
        }
        if let Err(e) = mpv.observe_property("pause", libmpv2::Format::Flag, 2) {
            warn!(?e, "failed to observe pause, pause events disabled");
        }
        if let Err(e) = mpv.observe_property("demuxer-cache-time", libmpv2::Format::Double, 3) {
            warn!(?e, "failed to observe demuxer-cache-time, buffer events disabled");
        }

        loop {
            match mpv.wait_event(1.0) {
                Some(Ok(libmpv2::events::Event::PropertyChange {
                            name: "time-pos",
                            change: libmpv2::events::PropertyData::Double(pos),
                            ..
                        })) => {
                    let _ = events.send(PlaybackEvent::Position(pos));
                }
                Some(Ok(libmpv2::events::Event::PropertyChange {
                            name: "pause",
                            change: libmpv2::events::PropertyData::Flag(paused),
                            ..
                        })) => {
                    let _ = events.send(PlaybackEvent::PauseChanged(paused));
                }
                Some(Ok(libmpv2::events::Event::PropertyChange {
                            name: "demuxer-cache-time",
                            change: libmpv2::events::PropertyData::Double(cache_time),
                            ..
                        })) => {
                    if cache_time >= 0.0 {
                        let _ = events.send(PlaybackEvent::Buffered(cache_time));
                    }
                }
                Some(Ok(libmpv2::events::Event::EndFile(reason))) => {
                    if matches!(reason, libmpv2::mpv_end_file_reason::Eof) {
                        let _ = events.send(PlaybackEvent::Eof);
                    }
                }
                Some(Ok(libmpv2::events::Event::Shutdown)) => break,
                Some(Err(e)) => {
                    warn!(?e, "mpv event error");
                    let _ = events.send(PlaybackEvent::Error(format!("{e:?}")));
                }
                // Timeout (None) or an event we don't care about: keep polling.
                _ => continue,
            }
        }
    });
}

fn build_mpv() -> libmpv2::Result<Mpv> {
    unsafe {
        let c_locale = std::ffi::CString::new("C").expect("no interior nul");
        libc::setlocale(libc::LC_NUMERIC, c_locale.as_ptr());
    }

    Mpv::with_initializer(|init: MpvInitializer| {
        init.set_option("vo", "libmpv")?;
        init.set_option("hwdec", "auto-safe")?;
        init.set_option("keep-open", "yes")?;
        init.set_option("idle", "yes")?;
        init.set_option("deband", "yes")?;
        init.set_option("ytdl", "no")?;
        init.set_option("demuxer-lavf-o", "multiple_requests=1")?;
        //init.set_option("msg-level", "all=debug")?;
        //init.set_option("log-file", "/tmp/hoshi-mpv.log")?;

        Ok(())
    })
}

fn mpv_list_escape(value: &str) -> String {
    format!("%{}%{}", value.len(), value)
}

fn write_chapters_file(chapters: &[EpisodeChapter], path: &Path) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    writeln!(f, ";FFMETADATA1")?;
    for c in chapters {
        let start_ms = (c.start * 1000.0).round() as i64;
        let end_ms = (c.end * 1000.0).round() as i64;
        writeln!(f, "[CHAPTER]")?;
        writeln!(f, "TIMEBASE=1/1000")?;
        writeln!(f, "START={start_ms}")?;
        writeln!(f, "END={end_ms}")?;
        writeln!(f, "title={}", escape_ffmetadata(&c.title))?;
    }
    Ok(())
}

/// FFMETADATA values need `=`, `;`, `#`, `\` and newlines backslash-escaped.
fn escape_ffmetadata(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if matches!(ch, '=' | ';' | '#' | '\\' | '\n') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}