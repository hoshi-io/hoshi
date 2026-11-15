use std::collections::HashMap;
use std::sync::Arc;
use hoshi_core::content::repositories::content::ContentRepository;
use serde_json::{json, Value};
use tauri::State;
use hoshi_core::AppState;
use hoshi_core::error::CoreError;
use hoshi_core::extensions::types::TorrentSearchResult;
use hoshi_core::torrent::TorrentService;
use hoshi_core::torrent::choices::{TorrentChoice, TorrentChoiceRepository};
use tracing::warn;
use hoshi_core::torrent::types::{
    TorrentCacheClearResult, TorrentCacheDeleteResult, TorrentCacheEntry, TorrentCacheUsage,
    TorrentLiveStats, TorrentStorageStats, TorrentStreamInfo,
};
use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn search_torrents(
    state: State<'_, Arc<AppState>>,
    id: String,
    query: String,
    filters: Value,
    page: u32,
    cid: Option<String>,
    episode: Option<u32>,
) -> Result<Value, CoreError> {
    let manager = &state.extension_manager;
    let filters = match (cid, episode) {
        (Some(cid), Some(ep)) => TorrentService::with_content_ids(state.inner(), &cid, ep, filters).await,
        _ => filters,
    };
    let results: Vec<TorrentSearchResult> = manager.search_torrents(&id, &query, filters, page).await?;
    Ok(json!({ "results": results }))
}

#[tauri::command]
pub async fn get_magnet(
    state: State<'_, Arc<AppState>>,
    id: String,
    content_id: String,
) -> Result<Value, CoreError> {
    let manager = &state.extension_manager;
    let magnet = manager.get_magnet(&id, &content_id).await?;
    Ok(json!({ "magnet": magnet }))
}

#[tauri::command]
pub async fn auto_select_torrent(
    state: State<'_, Arc<AppState>>,
    session: State<'_, TauriSession>,
    id: String,
    cid: String,
    episode: u32,
    filters: Value,
    page: u32,
) -> Result<Value, CoreError> {
    let user_id = require_auth(&session).await?;
    let manager = &state.extension_manager;
    let picked = TorrentService::auto_select_torrent(
        state.inner(), manager, user_id, &id, &cid, episode, filters, page,
    ).await?;
    Ok(json!({ "torrent": picked }))
}

#[tauri::command]
pub async fn start_torrent_stream(
    state: State<'_, Arc<AppState>>,
    session: State<'_, TauriSession>,
    id: String,
    content_id: String,
    magnet: Option<String>,
    cid: Option<String>,
    episode: Option<u32>,
    pack_only: Option<bool>,
    title: Option<String>,
) -> Result<TorrentStreamInfo, CoreError> {
    let user_id = require_auth(&session).await?;
    TorrentService::sync_config(state.inner(), user_id).await?;

    let magnet = match magnet.filter(|m| m.starts_with("magnet:")) {
        Some(m) => m,
        None => {
            let manager = &state.extension_manager;
            manager.get_magnet(&id, &content_id).await?
        }
    };

    let target = match (&cid, episode) {
        (Some(cid), Some(ep)) => TorrentService::build_target(state.inner(), cid, ep).await?,
        _ => None,
    };

    let added = state
        .inner()
        .torrent
        .add_magnet(&magnet, target.as_ref(), pack_only.unwrap_or(false))
        .await?;

    // Remember what played this episode, so coming back doesn't search again.
    if let (Some(cid), Some(ep)) = (&cid, episode) {
        let title = title.as_deref().unwrap_or(&content_id);
        if let Err(e) = TorrentChoiceRepository::save(
            &state.pool, user_id, cid, &id, ep, &content_id, title, &magnet, added.is_pack,
        ).await {
            warn!(error = ?e, "failed to save torrent choice");
        }
    }

    Ok(TorrentStreamInfo {
        url: format!("torrent://{}", added.session_id),
        session_id: added.session_id,
        total_size: added.total_size,
    })
}

/// The torrent saved for this episode, or the latest pack used for the show on this source.
#[tauri::command]
pub async fn get_torrent_choice(
    state: State<'_, Arc<AppState>>,
    session: State<'_, TauriSession>,
    id: String,
    cid: String,
    episode: u32,
) -> Result<Option<TorrentChoice>, CoreError> {
    let user_id = require_auth(&session).await?;
    TorrentChoiceRepository::find(&state.pool, user_id, &cid, &id, episode).await
}

#[tauri::command]
pub async fn prefetch_torrent_episode(
    state: State<'_, Arc<AppState>>,
    session_id: String,
    cid: String,
    episode: u32,
) -> Result<Value, CoreError> {
    let Some(target) = TorrentService::build_target(state.inner(), &cid, episode).await? else {
        return Ok(json!({ "inTorrent": false }));
    };
    let in_torrent = state.inner().torrent.prefetch_episode(&session_id, &target)?;
    Ok(json!({ "inTorrent": in_torrent }))
}

#[tauri::command]
pub async fn stop_torrent_stream(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<Value, CoreError> {
    state.inner().torrent.remove_session(&session_id).await?;
    Ok(json!({ "ok": true, "sessionId": session_id }))
}

#[tauri::command]
pub async fn get_torrent_stats(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<Option<TorrentLiveStats>, CoreError> {
    Ok(state.inner().torrent.stats(&session_id))
}

#[tauri::command]
pub async fn get_torrent_storage_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<TorrentStorageStats, CoreError> {
    state.inner().torrent.storage_stats().await
}

#[tauri::command]
pub async fn list_torrent_cache(
    state: State<'_, Arc<AppState>>,
    session: State<'_, TauriSession>,
) -> Result<Vec<TorrentCacheEntry>, CoreError> {
    let mut entries = state.inner().torrent.list_cache().await?;

    // Attach the shows/episodes each cached torrent played, from saved choices.
    let Ok(user_id) = require_auth(&session).await else { return Ok(entries) };
    let usages = TorrentChoiceRepository::usages(&state.pool, user_id).await.unwrap_or_default();
    let mut titles: HashMap<String, String> = HashMap::new();
    for entry in &mut entries {
        let mut by_cid: Vec<TorrentCacheUsage> = Vec::new();
        for (_, cid, ep) in usages.iter().filter(|(hash, _, _)| *hash == entry.key) {
            match by_cid.iter_mut().find(|u| &u.cid == cid) {
                Some(u) => u.episodes.push(*ep),
                None => by_cid.push(TorrentCacheUsage { cid: cid.clone(), title: String::new(), episodes: vec![*ep] }),
            }
        }
        for usage in &mut by_cid {
            if !titles.contains_key(&usage.cid) {
                let title = ContentRepository::get_full_content(&state.pool, &usage.cid).await.ok().flatten()
                    .and_then(|fc| fc.primary_metadata().map(|m| m.title.clone()))
                    .unwrap_or_default();
                titles.insert(usage.cid.clone(), title);
            }
            usage.title = titles[&usage.cid].clone();
        }
        entry.used_by = by_cid;
    }
    Ok(entries)
}

#[tauri::command]
pub async fn delete_torrent_cache_entry(
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<TorrentCacheDeleteResult, CoreError> {
    let freed_bytes = state.inner().torrent.delete_cache_entry(&key).await?;
    Ok(TorrentCacheDeleteResult { freed_bytes })
}

#[tauri::command]
pub async fn clear_torrent_cache(
    state: State<'_, Arc<AppState>>,
) -> Result<TorrentCacheClearResult, CoreError> {
    state.inner().torrent.clear_cache().await
}

#[tauri::command]
pub async fn get_torrent_cache_dir(
    state: State<'_, Arc<AppState>>,
) -> Result<Value, CoreError> {
    let path = state.inner().torrent.cache_dir().to_string_lossy().into_owned();
    Ok(json!({ "path": path }))
}