use std::collections::HashSet;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use dashmap::DashMap;
use librqbit::api::TorrentIdOrHash;
use librqbit::AddTorrentResponse;
use tokio::sync::{Mutex, Notify, RwLock};
use tracing::{info, warn};

use crate::config::model::TorrentConfig;
use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::torrent::types::{Lingering, StreamSession, TorrentLiveStats};

const METADATA_TIMEOUT: Duration = Duration::from_secs(45);
const INIT_TIMEOUT: Duration = Duration::from_secs(60);
const REAP_INTERVAL: Duration = Duration::from_secs(60);
const LAST_PLAYED_MARKER: &str = ".last_played";

#[derive(Clone)]
pub struct TorrentHandle(Arc<Inner>);

// libmpv2 wraps protocol callbacks in catch_unwind; only `open` can panic,
// and it does so before touching any shared state.
impl std::panic::RefUnwindSafe for TorrentHandle {}

struct Inner {
    data_dir: PathBuf,
    config: RwLock<TorrentConfig>,
    /// False until a real user config has been pushed in. The reaper
    /// refuses to delete anything while this is false.
    config_synced: AtomicBool,
    session: tokio::sync::OnceCell<Arc<librqbit::Session>>,
    rt: tokio::runtime::Handle,
    streams: DashMap<String, StreamSession>,
    /// Torrents kept running after playback (stop_seeding_on_playback_end = false).
    lingering: DashMap<usize, Lingering>,
    /// Held for the whole of add_magnet; the reaper try_locks it.
    gate: Mutex<()>,
    shutdown: Notify,
}

impl TorrentHandle {

    pub fn new(data_dir: PathBuf) -> Self {
        Self(Arc::new(Inner {
            data_dir,
            config: RwLock::new(TorrentConfig::default()),
            config_synced: AtomicBool::new(false),
            session: tokio::sync::OnceCell::new(),
            rt: tokio::runtime::Handle::current(),
            streams: DashMap::new(),
            lingering: DashMap::new(),
            gate: Mutex::new(()),
            shutdown: Notify::new(),
        }))
    }

    pub fn stats(&self, session_id: &str) -> Option<TorrentLiveStats> {
        let torrent = self.0.streams.get(session_id)?.torrent.clone();
        let v = serde_json::to_value(torrent.stats()).ok()?;

        let u = |p: &str| v.pointer(p).and_then(|x| x.as_u64()).unwrap_or(0);
        // librqbit's "mbps" is MiB/s despite the name
        let bps = |p: &str| {
            (v.pointer(p).and_then(|x| x.as_f64()).unwrap_or(0.0) * 1024.0 * 1024.0) as u64
        };

        Some(TorrentLiveStats {
            state: v.pointer("/state").and_then(|s| s.as_str()).unwrap_or("unknown").to_owned(),
            progress_bytes: u("/progress_bytes"),
            total_bytes: u("/total_bytes"),
            download_bps: bps("/live/download_speed/mbps"),
            upload_bps: bps("/live/upload_speed/mbps"),
            peers: u("/live/snapshot/peer_stats/live"),
        })
    }

    /// Called from `ConfigService::patch_config` and `TorrentService::sync_config`.
    pub async fn update_config(&self, config: TorrentConfig) {
        if let Some(session) = self.0.session.get() {
            Self::apply_limits(session, &config);
        }
        *self.0.config.write().await = config;
        self.0.config_synced.store(true, Ordering::Release);
    }

    fn apply_limits(session: &librqbit::Session, config: &TorrentConfig) {
        // librqbit wants bytes/sec; 0 or None means unlimited.
        let to_bps = |kib: Option<u32>| kib.and_then(|k| NonZeroU32::new(k.saturating_mul(1024)));
        session.ratelimits.set_download_bps(to_bps(config.download_rate_limit_kbps));
        session.ratelimits.set_upload_bps(to_bps(config.upload_rate_limit_kbps));
    }

    async fn ensure_session(&self) -> CoreResult<&Arc<librqbit::Session>> {
        self.0.session.get_or_try_init(|| async {
            let session = librqbit::Session::new(self.0.data_dir.clone())
                .await
                .map_err(|e| core_err!(Internal, "error.torrent.session_init_failed", e))?;
            Self::apply_limits(&session, &*self.0.config.read().await);
            Ok::<_, CoreError>(session)
        }).await
    }

    // ---------------------------------------------------------------- add

    pub async fn add_magnet(&self, magnet: &str) -> CoreResult<(String, u64)> {
        let dir = magnet_key(magnet)
            .ok_or_else(|| core_err!(BadRequest, "error.torrent.invalid_magnet"))?;

        // Serialises adds and keeps the reaper away from what we're about to use.
        let _gate = self.0.gate.lock().await;

        let config = self.0.config.read().await.clone();

        // During an episode switch the old and new sessions briefly coexist
        // (the old one is stopped after mpv moves on), so a cap below 2
        // would reject every switch.
        if self.0.streams.len() >= config.max_concurrent_torrents.max(2) {
            return Err(core_err!(BadRequest, "error.torrent.concurrency_limit_reached"));
        }

        // Make room before checking free space.
        self.reap_inner(Some(&dir)).await;

        let free_space = fs2::available_space(&self.0.data_dir)
            .map_err(|e| core_err!(Internal, "error.torrent.disk_check_failed", e))?;
        if free_space < config.min_free_disk_space_bytes {
            return Err(core_err!(BadRequest, "error.torrent.insufficient_disk_space"));
        }

        let session = self.ensure_session().await?;

        // add_torrent resolves magnet metadata from peers/DHT and has no
        // timeout of its own. Dropping the future on timeout cancels it.
        let response = tokio::time::timeout(
            METADATA_TIMEOUT,
            session.add_torrent(
                librqbit::AddTorrent::from_url(magnet),
                Some(librqbit::AddTorrentOptions {
                    output_folder: Some(self.0.data_dir.join(&dir).to_string_lossy().into_owned()),
                    overwrite: true, // re-adding a cached torrent must reuse its files
                    ..Default::default()
                }),
            ),
        )
            .await
            .map_err(|_| core_err!(BadRequest, "error.torrent.metadata_timeout"))?
            .map_err(|e| core_err!(Internal, "error.torrent.add_failed", e))?;

        let (torrent_id, torrent) = match response {
            AddTorrentResponse::Added(id, h) | AddTorrentResponse::AlreadyManaged(id, h) => (id, h),
            _ => return Err(core_err!(Internal, "error.torrent.add_failed_no_handle")),
        };

        // Active again, so no longer a reaper candidate.
        self.0.lingering.remove(&torrent_id);

        let (file_index, file_len) = match Self::wait_ready(&torrent).await {
            Ok(v) => v,
            Err(e) => {
                // Don't leak a torrent nobody will stream, unless a live
                // session shares it.
                if !self.in_use(torrent_id) {
                    self.release_torrent(torrent_id, true).await;
                }
                return Err(e);
            }
        };

        let session_id = uuid::Uuid::new_v4().to_string();
        self.0.streams.insert(
            session_id.clone(),
            StreamSession { torrent, torrent_id, dir, file_index, file_len },
        );

        Ok((session_id, file_len))
    }

    async fn wait_ready(torrent: &Arc<librqbit::ManagedTorrent>) -> CoreResult<(usize, u64)> {
        tokio::time::timeout(INIT_TIMEOUT, torrent.wait_until_initialized())
            .await
            .map_err(|_| core_err!(BadRequest, "error.torrent.init_timeout"))?
            .map_err(|e| core_err!(Internal, "error.torrent.metadata_resolve_failed", e))?;

        Self::pick_largest_file(torrent)
            .ok_or_else(|| core_err!(Internal, "error.torrent.no_files_found"))
    }

    fn pick_largest_file(torrent: &Arc<librqbit::ManagedTorrent>) -> Option<(usize, u64)> {
        torrent.with_metadata(|m| {
            m.file_infos.iter().enumerate()
                .max_by_key(|(_, f)| f.len)
                .map(|(i, f)| (i, f.len))
        }).ok().flatten()
    }

    /// Sync lookup, called from mpv's demuxer thread (not an async context).
    pub fn lookup_session(
        &self,
        session_id: &str,
    ) -> Option<(Arc<librqbit::ManagedTorrent>, usize, u64, tokio::runtime::Handle)> {
        self.0.streams.get(session_id).map(|s| {
            (s.torrent.clone(), s.file_index, s.file_len, self.0.rt.clone())
        })
    }

    // ------------------------------------------------------------ teardown

    fn in_use(&self, torrent_id: usize) -> bool {
        self.0.streams.iter().any(|s| s.torrent_id == torrent_id)
    }

    /// Removes the torrent from the librqbit session. `delete_files = false`
    /// keeps the data on disk as a cache for rewatches.
    async fn release_torrent(&self, torrent_id: usize, delete_files: bool) {
        self.0.lingering.remove(&torrent_id);
        let Some(session) = self.0.session.get() else { return };
        if let Err(e) = session.delete(TorrentIdOrHash::Id(torrent_id), delete_files).await {
            warn!(torrent_id, error = ?e, "failed to release torrent from session");
        }
    }

    pub async fn remove_session(&self, session_id: &str) -> CoreResult<()> {
        let Some((_, s)) = self.0.streams.remove(session_id) else {
            return Ok(());
        };

        // Another live session (same magnet loaded twice) still needs it.
        if self.in_use(s.torrent_id) {
            return Ok(());
        }

        let config = self.0.config.read().await.clone();
        // A TTL of 0 means "don't cache": wipe the files with the torrent.
        let delete_files = config.finished_file_ttl_seconds == 0;

        if !delete_files {
            // Last-played marker: the reaper's LRU signal.
            let marker = self.0.data_dir.join(&s.dir).join(LAST_PLAYED_MARKER);
            let _ = tokio::fs::write(marker, b"").await;
        }

        if config.stop_seeding_on_playback_end || delete_files {
            self.release_torrent(s.torrent_id, delete_files).await;
        } else {
            self.0.lingering.insert(
                s.torrent_id,
                Lingering { since: Instant::now(), dir: s.dir },
            );
        }
        Ok(())
    }

    // -------------------------------------------------------------- reaper

    /// Spawn once, at startup. Stops on `shutdown()`.
    pub fn spawn_reaper(&self) {
        let this = self.clone();
        self.0.rt.spawn(async move {
            let mut tick = tokio::time::interval(REAP_INTERVAL);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            tick.tick().await; // first tick is immediate; skip it
            loop {
                tokio::select! {
                    _ = tick.tick() => this.reap_once().await,
                    _ = this.0.shutdown.notified() => break,
                }
            }
        });
    }

    async fn reap_once(&self) {
        // An add in progress owns the cache right now; try again next tick.
        let Ok(_gate) = self.0.gate.try_lock() else { return };
        self.reap_inner(None).await;
    }

    /// Caller must hold `gate` (or be `reap_once`, which takes it).
    async fn reap_inner(&self, extra_protected: Option<&str>) {
        if !self.0.config_synced.load(Ordering::Acquire) {
            return; // never delete user data based on default settings
        }
        let config = self.0.config.read().await.clone();
        let ttl = Duration::from_secs(config.finished_file_ttl_seconds);

        // Stop lingering torrents that outlived the TTL.
        let expired: Vec<usize> = self.0.lingering.iter()
            .filter(|l| l.value().since.elapsed() >= ttl)
            .map(|l| *l.key())
            .collect();
        for id in expired {
            if !self.in_use(id) {
                self.release_torrent(id, false).await;
            }
        }

        // Evict cached data on disk.
        let mut protected: HashSet<String> =
            self.0.streams.iter().map(|s| s.dir.clone()).collect();
        protected.extend(self.0.lingering.iter().map(|l| l.value().dir.clone()));
        protected.extend(extra_protected.map(str::to_owned));

        let root = self.0.data_dir.clone();
        let cap = config.max_disk_usage_bytes;
        if let Err(e) = tokio::task::spawn_blocking(move || evict_cache(&root, &protected, ttl, cap)).await {
            warn!(error = ?e, "torrent cache eviction task failed");
        }
    }

    pub async fn shutdown(&self) -> CoreResult<()> {
        self.0.shutdown.notify_one();
        if let Some(session) = self.0.session.get() {
            session.stop().await;
        }
        Ok(())
    }
}


/// Extracts the btih from a magnet URI, lowercased. Used as the torrent's
/// folder name, so it's restricted to [a-z0-9] (no path traversal possible).
fn magnet_key(magnet: &str) -> Option<String> {
    let query = magnet.strip_prefix("magnet:?")?;
    query
        .split('&')
        .find_map(|kv| {
            kv.strip_prefix("xt=urn:btih:")
                .or_else(|| kv.strip_prefix("xt=urn%3Abtih%3A"))
        })
        .map(|h| h.to_ascii_lowercase())
        .filter(|h| is_cache_dir_name(h))
}

/// 40-char hex or 32-char base32 info-hash.
fn is_cache_dir_name(name: &str) -> bool {
    (name.len() == 40 || name.len() == 32) && name.bytes().all(|b| b.is_ascii_alphanumeric())
}

struct CacheEntry {
    name: String,
    path: PathBuf,
    size: u64,
    last_used: SystemTime,
}

/// Blocking. Deletes expired entries and, oldest first, enough others to get
/// under `cap`. Protected entries are never touched but count toward the total.
fn evict_cache(root: &Path, protected: &HashSet<String>, ttl: Duration, cap: u64) {
    let Ok(read_dir) = std::fs::read_dir(root) else { return };

    let mut entries: Vec<CacheEntry> = read_dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            (is_cache_dir_name(&name) && path.is_dir()).then(|| {
                let (size, last_used) = dir_stats(&path);
                CacheEntry { name, path, size, last_used }
            })
        })
        .collect();

    let mut total: u64 = entries.iter().map(|e| e.size).sum();
    entries.sort_by_key(|e| e.last_used); // oldest first
    let now = SystemTime::now();

    for e in entries {
        if protected.contains(&e.name) {
            continue;
        }
        let expired = now.duration_since(e.last_used).map_or(false, |age| age >= ttl);
        if !expired && total <= cap {
            continue;
        }
        match std::fs::remove_dir_all(&e.path) {
            Ok(()) => {
                total = total.saturating_sub(e.size);
                info!(dir = %e.name, bytes = e.size, "evicted torrent cache entry");
            }
            Err(err) => warn!(dir = %e.name, error = ?err, "failed to evict torrent cache entry"),
        }
    }
}

/// (total bytes, newest mtime) for a directory tree. Symlinks aren't followed.
fn dir_stats(path: &Path) -> (u64, SystemTime) {
    let mut size = 0u64;
    let mut newest = SystemTime::UNIX_EPOCH;
    let mut stack = vec![path.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if let Ok(m) = meta.modified() {
                newest = newest.max(m);
            }
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                size += meta.len();
            }
        }
    }

    // Count the folder's own mtime so empty folders still age out.
    if let Ok(m) = std::fs::metadata(path).and_then(|m| m.modified()) {
        newest = newest.max(m);
    }
    (size, newest)
}