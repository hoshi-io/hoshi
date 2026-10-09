use std::sync::Arc;
use serde_json::{json, Value};
use tauri::State;
use hoshi_core::AppState;
use hoshi_core::error::CoreError;
use hoshi_core::extensions::types::TorrentSearchResult;
use hoshi_core::torrent::TorrentService;
use hoshi_core::torrent::types::{
    TorrentCacheClearResult, TorrentCacheDeleteResult, TorrentCacheEntry,
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
) -> Result<Value, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let results: Vec<TorrentSearchResult> = manager.search_torrents(&id, &query, filters, page).await?;
    Ok(json!({ "results": results }))
}

#[tauri::command]
pub async fn get_magnet(
    state: State<'_, Arc<AppState>>,
    id: String,
    content_id: String,
) -> Result<Value, CoreError> {
    let manager = state.inner().extension_manager.read().await;
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
    let manager = state.inner().extension_manager.read().await;
    let picked = TorrentService::auto_select_torrent(
        state.inner(), &manager, user_id, &id, &cid, episode, filters, page,
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
) -> Result<TorrentStreamInfo, CoreError> {
    let user_id = require_auth(&session).await?;
    TorrentService::sync_config(state.inner(), user_id).await?;
    let magnet = match magnet.filter(|m| m.starts_with("magnet:")) {
        Some(m) => m,
        None => {
            let manager = state.inner().extension_manager.read().await;
            manager.get_magnet(&id, &content_id).await?
        }
    };

    let (session_id, total_size) = state.inner().torrent.add_magnet(&magnet).await?;

    Ok(TorrentStreamInfo {
        url: format!("torrent://{session_id}"),
        session_id,
        total_size,
    })
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
) -> Result<Vec<TorrentCacheEntry>, CoreError> {
    state.inner().torrent.list_cache().await
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