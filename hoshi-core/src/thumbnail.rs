use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use libmpv2::events::Event;
use libmpv2::Mpv;
use lru::LruCache;
use tokio::sync::oneshot;
use tracing::{warn};

use crate::config::model::PlayerConfig;
use crate::playback::{LoadSpec, ProtocolHook};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceKind { File, Hls, Torrent }

impl SourceKind {
    pub fn detect(url: &str) -> Self {
        if url.starts_with("torrent://") { Self::Torrent }
        else if url.contains(".m3u8") { Self::Hls }
        else { Self::File }
    }
}

#[derive(Debug, Clone)]
pub struct ThumbSettings {
    pub interval: f64,
    pub width: u16,
    pub quality: u8,
    pub cache_size: usize,
    pub hwdec: bool,
    pub precise: bool,
}

impl ThumbSettings {
    /// None => thumbnails disabled for this source.
    pub fn from_config(p: &PlayerConfig, kind: SourceKind) -> Option<Self> {
        let allowed = p.thumbnails_enabled && match kind {
            SourceKind::File => true,
            SourceKind::Hls => p.thumbnail_hls,
            SourceKind::Torrent => p.thumbnail_torrent,
        };
        allowed.then(|| Self {
            interval: p.thumbnail_interval_seconds.max(1) as f64,
            width: p.thumbnail_width.clamp(80, 640),
            quality: p.thumbnail_quality.clamp(10, 95),
            cache_size: p.thumbnail_cache_size.max(16) as usize,
            hwdec: p.thumbnail_hwdec,
            precise: p.thumbnail_precise_seek,
        })
    }
}

pub struct Thumbnail { pub time: f64, pub jpeg: Vec<u8> }

#[derive(Debug)]
pub enum ThumbError { Disabled, Superseded, Unavailable }

enum Cmd {
    Open { spec: LoadSpec, kind: SourceKind, cfg: ThumbSettings, tmp_dir: PathBuf, done: oneshot::Sender<bool> },
    Get  { time: f64, reply: oneshot::Sender<Result<Thumbnail, ThumbError>> },
    Close { done: oneshot::Sender<()> },
}

#[derive(Clone)]
pub struct ThumbnailService {
    tx: Arc<Mutex<Option<Sender<Cmd>>>>,
    hooks: Arc<Mutex<Vec<ProtocolHook>>>,
    tmp_dir: PathBuf,
}

impl ThumbnailService {
    pub fn new(tmp_dir: PathBuf) -> Self {
        Self { tx: Arc::new(Mutex::new(None)), hooks: Arc::new(Mutex::new(vec![])), tmp_dir }
    }

    pub fn add_protocol_hook(&self, hook: impl Fn(&Mpv) -> libmpv2::Result<()> + Send + Sync + 'static) {
        self.hooks.lock().unwrap().push(Box::new(hook));
    }

    fn sender(&self) -> Sender<Cmd> {
        let mut g = self.tx.lock().unwrap();
        if let Some(tx) = g.as_ref() { return tx.clone(); }
        let (tx, rx) = mpsc::channel();
        let hooks = self.hooks.clone();
        std::thread::Builder::new()
            .name("thumbnails".into())
            .spawn(move || Worker::run(rx, hooks))
            .expect("spawn thumbnail worker");
        *g = Some(tx.clone());
        tx
    }

    /// Call from the same place as `playback.load`. Returns false if thumbs
    /// are disabled / unsupported for this source.
    pub async fn open(&self, spec: LoadSpec, cfg: &PlayerConfig) -> bool {
        let kind = SourceKind::detect(&spec.url);
        let Some(cfg) = ThumbSettings::from_config(cfg, kind) else {
            warn!(?kind, "thumb: disabled by config for this source kind");
            self.close().await;
            return false;
        };
        let (done, rx) = oneshot::channel();
        let tmp_dir = self.tmp_dir.clone();
        let _ = self.sender().send(Cmd::Open { spec, kind, cfg, tmp_dir, done });
        rx.await.unwrap_or(false)
    }

    pub async fn get(&self, time: f64) -> Result<Thumbnail, ThumbError> {
        let (reply, rx) = oneshot::channel();
        let guard = self.tx.lock().unwrap().clone();
        let Some(tx) = guard else { return Err(ThumbError::Disabled) };
        let _ = tx.send(Cmd::Get { time, reply });
        rx.await.unwrap_or(Err(ThumbError::Superseded))
    }

    /// Stops decoding and releases the file (and any torrent stream) WITHOUT
    /// destroying the mpv core.
    pub async fn close(&self) {
        let tx = self.tx.lock().unwrap().clone();
        if let Some(tx) = tx {
            let (done, rx) = oneshot::channel();
            if tx.send(Cmd::Close { done }).is_ok() { let _ = rx.await; }
        }
    }

    pub fn shutdown(&self) { *self.tx.lock().unwrap() = None; } // dropping the sender ends the worker
}

struct Worker {
    mpv: Mpv,
    cfg: Option<ThumbSettings>,
    kind: SourceKind,
    open_id: u64,
    duration: f64,
    cache: LruCache<(u64, u32), Arc<Vec<u8>>>,
    tmp: PathBuf,
}

impl Worker {
    fn run(rx: Receiver<Cmd>, hooks: Arc<Mutex<Vec<ProtocolHook>>>) {
        let mpv = match build_thumb_mpv(&hooks) {
            Ok(m) => m,
            Err(e) => { warn!(?e, "thumbnail core init failed"); return; }
        };
        let mut w = Worker {
            mpv, cfg: None, kind: SourceKind::File, open_id: 0, duration: 0.0,
            cache: LruCache::new(NonZeroUsize::new(200).unwrap()),
            tmp: std::env::temp_dir().join(format!("hoshi-thumb-{}.jpg", std::process::id())),
        };
        let mut pending: Option<Cmd> = None;

        loop {
            let cmd = match pending.take() {
                Some(c) => c,
                None => match rx.recv() { Ok(c) => c, Err(_) => break },
            };
            match cmd {
                Cmd::Open { spec, kind, cfg, tmp_dir, done } => {
                    w.tmp = tmp_dir.join(format!("hoshi-thumb-{}.jpg", std::process::id()));
                    let _ = done.send(w.open(spec, kind, cfg));
                }
                Cmd::Close { done } => { w.close(); let _ = done.send(()); },
                Cmd::Get { time, reply } => {
                    // Coalesce: keep only the newest Get; stop at any other command.
                    let (mut time, mut reply) = (time, reply);
                    while let Ok(next) = rx.try_recv() {
                        match next {
                            Cmd::Get { time: t, reply: r } => {
                                let _ = reply.send(Err(ThumbError::Superseded));
                                time = t; reply = r;
                            }
                            other => { pending = Some(other); break; }
                        }
                    }
                    let _ = reply.send(w.grab(time));
                }
            }
        }
        let _ = w.mpv.command("quit", &[]);
    }

    fn open(&mut self, spec: LoadSpec, kind: SourceKind, cfg: ThumbSettings) -> bool {
        self.open_id += 1;
        self.cache.clear();
        self.cache.resize(NonZeroUsize::new(cfg.cache_size).unwrap());
        self.kind = kind;
        self.duration = 0.0;

        while self.mpv.wait_event(0.0).is_some() {}

        let r = (|| -> libmpv2::Result<()> {
            let m = &self.mpv;

            // Thumbnails need CPU-readable frames. On Android, mediacodec decodes into a
            // surface that vo=null can't read back, so screenshot-to-file fails.
            let hwdec = if cfg!(target_os = "android") || !cfg.hwdec { "no" } else { "auto-safe" };
            if let Err(e) = m.set_property("hwdec", hwdec) { warn!(?e, "thumb: hwdec failed"); }
            if let Err(e) = m.set_property("hr-seek", if cfg.precise { "yes" } else { "no" }) {
                warn!(?e, "thumb: hr-seek failed");
            }
            if let Err(e) = m.command("vf", &["set", &format!("scale={}:-2", cfg.width)]) {
                warn!(?e, "thumb: vf set failed");
            }
            m.set_property("screenshot-jpeg-quality", cfg.quality as i64)?;
            m.set_property("hls-bitrate", "min")?;
            apply_headers(m, &spec.headers)?;

            let url = if kind == SourceKind::Torrent {
                spec.url.replacen("torrent://", "torrentthumb://", 1)
            } else { spec.url.clone() };

            m.command("loadfile", &[&url, "replace"])?;
            Ok(())
        })();
        if r.is_err() { return false; }
        self.cfg = Some(cfg);

        // wait for the file to be ready, then read the duration
        if !self.wait(|e| matches!(e, Event::FileLoaded), Duration::from_secs(8)) {
            warn!("thumb: FileLoaded not received in 8s");
            self.close();
            return false;
        }
        self.duration = self.mpv.get_property::<f64>("duration").unwrap_or(0.0);
        warn!(duration = self.duration, "thumb: opened");
        self.duration > 0.0
    }

    fn close(&mut self) {
        let _ = self.mpv.command("stop", &[]);
        self.cfg = None;
        self.cache.clear();
    }

    fn grab(&mut self, time: f64) -> Result<Thumbnail, ThumbError> {
        let cfg = self.cfg.as_ref().ok_or(ThumbError::Disabled)?.clone();
        let bucket = (time.max(0.0) / cfg.interval).floor() as u32;
        let snapped = (bucket as f64 * cfg.interval).min((self.duration - 1.0).max(0.0));

        if let Some(hit) = self.cache.get(&(self.open_id, bucket)) {
            return Ok(Thumbnail { time: snapped, jpeg: hit.as_ref().clone() });
        }

        let timeout = if self.kind == SourceKind::File { Duration::from_secs(3) } else { Duration::from_secs(5) };
        let flags = if cfg.precise { "absolute+exact" } else { "absolute+keyframes" };
        self.mpv.command("seek", &[&snapped.to_string(), flags])
            .map_err(|e| { warn!(?e, "thumb: seek failed"); ThumbError::Unavailable })?;

        if !self.wait(|e| matches!(e, Event::PlaybackRestart), timeout) {
            warn!(snapped, "thumb: no PlaybackRestart");
            return Err(ThumbError::Unavailable);
        }

        let path = self.tmp.to_string_lossy().to_string();
        self.mpv.command("screenshot-to-file", &[&path, "video"])
            .map_err(|e| { warn!(?e, path = %path, "thumb: screenshot failed"); ThumbError::Unavailable })?;
        let jpeg = std::fs::read(&self.tmp)
            .map_err(|e| { warn!(?e, "thumb: read failed"); ThumbError::Unavailable })?;
        let _ = std::fs::remove_file(&self.tmp);

        self.cache.put((self.open_id, bucket), Arc::new(jpeg.clone()));
        Ok(Thumbnail { time: snapped, jpeg })
    }

    fn wait(&self, pred: impl Fn(&Event) -> bool, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.mpv.wait_event(0.1) {
                Some(Ok(e)) if pred(&e) => return true,
                Some(Ok(Event::EndFile(reason))) => {
                    warn!(reason, "thumb: EndFile while waiting");
                    return false;
                }
                Some(Err(e)) => warn!(?e, "thumb: mpv event error"),
                _ => {}
            }
        }
        false
    }
}

fn build_thumb_mpv(hooks: &Arc<Mutex<Vec<ProtocolHook>>>) -> libmpv2::Result<Mpv> {
    let mpv = Mpv::with_initializer(|i| {
        i.set_option("vo", "null")?;
        i.set_option("ao", "null")?;
        i.set_option("aid", "no")?;
        i.set_option("sid", "no")?;
        i.set_option("pause", "yes")?;
        i.set_option("idle", "yes")?;
        i.set_option("keep-open", "yes")?;
        i.set_option("ytdl", "no")?;
        i.set_option("cache", "yes")?;
        i.set_option("demuxer-max-bytes", "16MiB")?;
        i.set_option("demuxer-readahead-secs", "2")?;
        i.set_option("audio-display", "no")?;
        i.set_option("demuxer-lavf-o", "multiple_requests=1")?;
        Ok(())
    })?;
    for h in hooks.lock().unwrap().iter() { h(&mpv)?; }
    Ok(mpv)
}

fn apply_headers(m: &Mpv, headers: &[(String, String)]) -> libmpv2::Result<()> {
    let esc = |v: &str| format!("%{}%{}", v.len(), v);
    let mut ua = String::new();
    let mut other = Vec::new();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("user-agent") { ua = v.clone(); }
        else { other.push(esc(&format!("{k}: {v}"))); }
    }
    m.set_property("http-header-fields", other.join(","))?;
    m.set_property("user-agent", ua)
}