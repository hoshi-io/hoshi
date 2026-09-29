use std::sync::Arc;
use std::time::Instant;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentLiveStats {
    pub state: String,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub download_bps: u64,
    pub upload_bps: u64,
    pub peers: u64,
}

pub struct StreamSession {
    pub torrent: Arc<librqbit::ManagedTorrent>,
    pub torrent_id: usize,
    pub dir: String,
    pub file_index: usize,
    pub file_len: u64,
}

pub struct Lingering {
    pub since: Instant,
    pub dir: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStreamInfo {
    pub session_id: String,
    pub url: String,
    pub total_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheEntryState {
    /// Being streamed right now. Cannot be deleted.
    Active,
    Seeding,
    Cached,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentCacheFile {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentCacheEntry {
    pub key: String,
    pub name: String,
    pub size_bytes: u64,
    pub last_used_ms: u64,
    pub state: CacheEntryState,
    pub files: Vec<TorrentCacheFile>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStorageStats {
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub total_disk_bytes: u64,
    pub entry_count: usize,
    pub active_count: usize,
    pub seeding_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentCacheDeleteResult {
    pub freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentCacheClearResult {
    pub removed: usize,
    pub skipped: usize,
    pub freed_bytes: u64,
}