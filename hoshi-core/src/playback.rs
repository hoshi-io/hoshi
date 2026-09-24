//! Platform-agnostic playback core wrapping [`libmpv2::Mpv`].

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use libmpv2::{Mpv, MpvInitializer};
use serde::Serialize;
use tokio::sync::{Notify, RwLock};
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
///
/// Headers, subtitles and chapters are all applied as part of the same
/// `load()` call rather than as separate follow-up commands, so there's no
/// window where the file is playing without its subs/headers/chapters
/// attached.
#[derive(Debug, Clone, Default)]
pub struct LoadSpec {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub subtitles: Vec<ExternalSubtitle>,
    pub chapters: Vec<ChapterMark>,
    pub start_position: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ExternalSubtitle {
    pub url: String,
    pub title: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChapterMark {
    pub title: Option<String>,
    pub time: f64,
}

#[derive(Clone)]
pub struct PlaybackHandle {
    inner: Arc<RwLock<Option<Arc<Mpv>>>>,
    /// Bumped on every successful `initialize()`. Lets long-lived consumers
    /// (the render surface) tell mpv sessions apart across shutdown/restart.
    session: Arc<AtomicU64>,
    /// Notified right before `shutdown()` drops the core's own `Arc<Mpv>`.
    /// Anything else holding a clone (the platform render surface) MUST drop
    /// its clone in response, or the refcount never reaches 0 and mpv (and
    /// its audio device) is never actually torn down.
    shutdown_notify: Arc<Notify>,
    /// Path of the temp FFMETADATA chapters file backing the currently
    /// loaded file's `chapters-file`, if any. Tracked so we can delete the
    /// previous one instead of leaking a file into the temp dir per episode.
    chapters_file: Arc<Mutex<Option<PathBuf>>>,
}

impl PlaybackHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            session: Arc::new(AtomicU64::new(0)),
            shutdown_notify: Arc::new(Notify::new()),
            chapters_file: Arc::new(Mutex::new(None)),
        }
    }

    /// Subscribe to shutdown notifications. Any holder of a cloned
    /// `Arc<Mpv>` (obtained via `mpv_handle()`) should call this once and,
    /// each time it fires, drop its clone.
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

        *guard = Some(Arc::new(mpv));
        self.session.fetch_add(1, Ordering::SeqCst);
        info!("playback core initialized");
        Ok(())
    }

    /// Tears down the mpv core safely on a blocking thread. Notifies
    /// subscribers first so any other `Arc<Mpv>` holder can release its
    /// clone; otherwise this only decrements a refcount that never hits 0.
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

        // Headers: applied as a session-wide property (not a per-file
        // loadfile option) right before loadfile. `http-header-fields`
        // isn't documented as reliably scoped to a single playlist entry,
        // so setting it globally is what actually covers the main stream
        // *and* whatever segment/sub-file requests mpv makes right after
        // for this same file. mpv/ffmpeg makes these requests itself, not
        // the webview, so there's no CORS involved at all here — this
        // replaces the old proxy-URL approach entirely, it isn't just an
        // alternative to it.
        let header_value = if spec.headers.is_empty() {
            String::new()
        } else {
            spec.headers
                .iter()
                .map(|(k, v)| mpv_list_escape(&format!("{k}: {v}")))
                .collect::<Vec<_>>()
                .join(",")
        };
        {
            let mpv = mpv.clone();
            self.run(move || mpv.set_property("http-header-fields", header_value))
                .await?;
        }

        // Drop the previous file's temp chapters file before writing a new
        // one, so these don't pile up in the temp dir across episodes.
        self.cleanup_chapters_file();

        let mut options: Vec<String> = Vec::new();
        for sub in &spec.subtitles {
            options.push(format!("sub-file={}", mpv_list_escape(&sub.url)));
        }

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
        self.run(move || mpv.command("loadfile", &[&url, mode_str, "-1", &options_str]))
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

    /// `target` in seconds. `relative` seeks from the current position
    /// instead of to an absolute position.
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
        Ok(())
    })
}

/// Escapes a value for embedding in an mpv list-style option/property
/// (e.g. `http-header-fields`, or a value inside a `loadfile` options
/// string) using mpv's own `%LEN%value` escape: mpv reads exactly `LEN`
/// raw bytes after the second `%`, so the value can contain commas, colons,
/// or anything else without needing per-character escaping.
fn mpv_list_escape(value: &str) -> String {
    format!("%{}%{}", value.len(), value)
}

/// Writes extension-provided chapter markers out as an FFMETADATA1 chapters
/// file, suitable for mpv's `chapters-file` option. `END` for each chapter
/// is set to the next chapter's start (or an arbitrary +1h sentinel for the
/// last one) — mpv only uses `time` for chapter navigation (see
/// `chapters()`), so END just needs to not overlap the next entry.
fn write_chapters_file(chapters: &[ChapterMark], path: &Path) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    writeln!(f, ";FFMETADATA1")?;
    for (i, c) in chapters.iter().enumerate() {
        let start_ms = (c.time * 1000.0).round() as i64;
        let end_ms = chapters
            .get(i + 1)
            .map(|next| (next.time * 1000.0).round() as i64)
            .unwrap_or(start_ms + 3_600_000);
        writeln!(f, "[CHAPTER]")?;
        writeln!(f, "TIMEBASE=1/1000")?;
        writeln!(f, "START={start_ms}")?;
        writeln!(f, "END={end_ms}")?;
        if let Some(title) = &c.title {
            writeln!(f, "title={}", escape_ffmetadata(title))?;
        }
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