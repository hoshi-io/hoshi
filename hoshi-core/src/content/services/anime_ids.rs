use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::core_err;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

const TSV_URL: &str = "https://raw.githubusercontent.com/nattadasu/animeApi/refs/heads/v3/database/animeapi.tsv";
const TSV_FILE: &str = "anime_ids.tsv";
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

const ALLOWED: &[&str] = &[
    "anidb", "anilist", "animeplanet", "hikka", "kitsu", "myanimelist",
    "notify", "shikimori", "simkl", "trakt", "annict",
];
const INDEXED: &[&str] = &["anilist", "mal", "kitsu", "simkl"];

static CACHE: Mutex<Option<(SystemTime, Arc<AnimeIdIndex>)>> = Mutex::const_new(None);

pub struct AnimeIdIndex {
    text: String,
    cols: Vec<(usize, &'static str)>,
    lines: Vec<(usize, usize)>,
    by_key: HashMap<String, usize>,
}

impl AnimeIdIndex {

    pub async fn get(state: &Arc<AppState>) -> CoreResult<Arc<AnimeIdIndex>> {
        let mut guard = CACHE.lock().await;

        if let Some((loaded_at, idx)) = guard.as_ref() {
            if loaded_at.elapsed().map(|a| a < MAX_AGE).unwrap_or(false) {
                return Ok(idx.clone());
            }
        }

        let path = state.paths.base_dir.join(TSV_FILE);

        if !Self::file_is_fresh(&path).await {
            info!("Refreshing anime cross-ID dump");
            if let Err(e) = Self::download(state, &path).await {
                warn!(error = ?e, "Anime cross-ID dump download failed");
                if !path.exists() {
                    return Err(e);
                }
            }
        }

        let text = tokio::fs::read_to_string(&path).await.map_err(CoreError::Io)?;
        let idx = tokio::task::spawn_blocking(move || Self::parse(text))
            .await
            .map_err(|e| CoreError::Internal(format!("anime id index task failed: {e}").into()))??;

        info!(rows = idx.lines.len(), keys = idx.by_key.len(), "Anime cross-ID index loaded");
        let idx = Arc::new(idx);
        *guard = Some((SystemTime::now(), idx.clone()));
        Ok(idx)
    }

    pub fn lookup(&self, tracker: &str, id: &str) -> Option<HashMap<String, String>> {
        let tracker = match tracker.to_lowercase().as_str() {
            "mal" | "myanimelist" => "mal",
            "anilist" => "anilist",
            "kitsu" => "kitsu",
            "simkl" => "simkl",
            _ => return None,
        };
        // MAL IDs are stored as "anime:123" in the app
        let id = if tracker == "mal" {
            id.split_once(':').map_or(id, |(_, raw)| raw)
        } else {
            id
        };

        let row = *self.by_key.get(&format!("{tracker}:{id}"))?;
        let (start, end) = self.lines[row];
        let fields: Vec<&str> = self.text[start..end].split('\t').collect();

        let mut out = HashMap::new();
        for (col, name) in &self.cols {
            if let Some(v) = fields.get(*col).map(|v| v.trim()).filter(|v| !v.is_empty()) {
                out.insert((*name).to_string(), v.to_string());
            }
        }
        Some(out)
    }

    fn parse(text: String) -> CoreResult<Self> {
        let mut lines = Vec::new();
        let mut cols: Vec<(usize, &'static str)> = Vec::new();
        let mut by_key = HashMap::new();
        let mut offset = 0usize;
        let mut first = true;

        for raw in text.split_inclusive('\n') {
            let start = offset;
            offset += raw.len();
            let line = raw.trim_end_matches(['\n', '\r']);

            if first {
                first = false;
                for (i, h) in line.split('\t').enumerate() {
                    if let Some(a) = ALLOWED.iter().find(|a| **a == h) {
                        cols.push((i, if *a == "myanimelist" { "mal" } else { *a }));
                    }
                }
                if !cols.iter().any(|(_, n)| *n == "anilist") {
                    return Err(CoreError::Internal("error.enrichment.bad_id_dump".into()));
                }
                continue;
            }
            if line.is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            let row = lines.len();
            let mut any = false;
            for (col, name) in &cols {
                if !INDEXED.contains(name) {
                    continue;
                }
                if let Some(v) = fields.get(*col).map(|v| v.trim()).filter(|v| !v.is_empty()) {
                    by_key.entry(format!("{name}:{v}")).or_insert(row);
                    any = true;
                }
            }
            if any {
                lines.push((start, start + line.len()));
            }
        }

        Ok(Self { text, cols, lines, by_key })
    }

    async fn file_is_fresh(path: &std::path::Path) -> bool {
        match tokio::fs::metadata(path).await.and_then(|m| m.modified()) {
            Ok(modified) => modified.elapsed().map(|a| a < MAX_AGE).unwrap_or(false),
            Err(_) => false,
        }
    }

    async fn download(state: &Arc<AppState>, path: &std::path::Path) -> CoreResult<()> {
        let bytes = state.http_client.get(TSV_URL).send().await
            .map_err(|e| core_err!(Network, "error.system.network", e))?
            .error_for_status()
            .map_err(|e| core_err!(Network, "error.system.network", e))?
            .bytes().await
            .map_err(|e| core_err!(Network, "error.system.network", e))?;

        if !bytes.starts_with(b"title\t") {
            return Err(CoreError::Internal("error.enrichment.bad_id_dump".into()));
        }

        // write to a temp file, then rename, so a crash never leaves a half file
        let tmp = path.with_extension("tsv.tmp");
        tokio::fs::write(&tmp, &bytes).await.map_err(CoreError::Io)?;
        tokio::fs::rename(&tmp, path).await.map_err(CoreError::Io)?;
        Ok(())
    }
}