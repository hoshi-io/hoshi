use serde_json::Value;

use crate::config::model::TorrentConfig;
use crate::config::service::ConfigService;
use crate::error::CoreResult;
use crate::extensions::types::TorrentSearchResult;
use crate::extensions::ExtensionManager;
use crate::state::AppState;

pub struct TorrentService;

impl TorrentService {
    pub fn pick_best_torrent<'a>(
        results: &'a [TorrentSearchResult],
        config: &TorrentConfig,
    ) -> Option<&'a TorrentSearchResult> {
        let candidates: Vec<&TorrentSearchResult> = results.iter()
            .filter(|t| t.seeders.unwrap_or(0).max(0) as u64 >= config.min_seeders as u64)
            .filter(|t| !config.exclude_keywords.iter()
                .any(|kw| t.title.to_lowercase().contains(&kw.to_lowercase())))
            .filter(|t| match config.require_batch {
                Some(want) => t.is_batch.unwrap_or(false) == want,
                None => true,
            })
            .collect();

        if candidates.is_empty() { return None; }

        let score = |t: &TorrentSearchResult| -> (i32, i32, i32, i64) {
            let group_score = t.release_group.as_deref()
                .and_then(|g| config.preferred_groups.iter().position(|p| p.eq_ignore_ascii_case(g)))
                .map(|pos| (config.preferred_groups.len() - pos) as i32)
                .unwrap_or(0);

            let res_score = t.resolution.as_deref()
                .map(|r| if r.eq_ignore_ascii_case(&config.preferred_resolution) { 1 } else { 0 })
                .unwrap_or(0);

            let codec_score = config.preferred_codec.as_deref()
                .map(|c| if t.title.to_lowercase().contains(&c.to_lowercase()) { 1 } else { 0 })
                .unwrap_or(0)
                + if config.prefer_dual_audio && t.title.to_lowercase().contains("dual") { 1 } else { 0 };

            (group_score, res_score, codec_score, t.seeders.unwrap_or(0))
        };

        candidates.into_iter().max_by_key(|t| score(t))
    }
    
    pub async fn auto_select_torrent(
        state: &AppState,
        manager: &ExtensionManager,
        user_id: i32,
        ext_id: &str,
        query: &str,
        filters: Value,
        page: u32,
    ) -> CoreResult<Option<TorrentSearchResult>> {
        let config = ConfigService::get_config(state, user_id).await?.torrent;

        if !config.auto_select {
            return Ok(None);
        }

        let results = manager.search_torrents(ext_id, query, filters, page).await?;
        Ok(Self::pick_best_torrent(&results, &config).cloned())
    }

    pub async fn sync_config(state: &AppState, user_id: i32) -> CoreResult<TorrentConfig> {
        let config = ConfigService::get_config(state, user_id).await?.torrent;
        state.torrent.update_config(config.clone()).await;
        Ok(config)
    }
}