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