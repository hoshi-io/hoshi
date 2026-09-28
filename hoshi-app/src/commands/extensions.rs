use hoshi_core::{
    error::CoreError,
    state::AppState,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use hoshi_core::extensions::types::{Extension, ExtensionFeatures, LNReaderMarketplaceEntry, SoraMarketplaceEntry, TorrentSearchResult};
use hoshi_core::torrent::TorrentService;
use hoshi_core::torrent::types::TorrentLiveStats;
use crate::{require_auth, TauriSession};

#[derive(Serialize)]
pub struct ExtensionsResponse<T> {
    pub extensions: T,
}

#[tauri::command]
pub async fn get_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<ExtensionsResponse<Vec<Extension>>, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let list: Vec<Extension> = manager
        .list_extensions()
        .iter()
        .map(|e| (*e).clone())
        .collect();

    Ok(ExtensionsResponse { extensions: list })
}

#[tauri::command]
pub async fn install_extension(
    state: State<'_, Arc<AppState>>,
    manifest_url: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.install_extension(state.inner(), &manifest_url).await?;
    Ok(json!({ "ok": true, "extension": extension }))
}

#[tauri::command]
pub async fn install_lnreader_extension(
    state: State<'_, Arc<AppState>>,
    entry: LNReaderMarketplaceEntry,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.install_lnreader_extension(state.inner(), entry).await?;
    Ok(json!({ "ok": true, "extension": extension }))
}

#[tauri::command]
pub async fn install_sora_extension(
    state: State<'_, Arc<AppState>>,
    entry: SoraMarketplaceEntry,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.install_sora_extension(state.inner(), entry).await?;
    Ok(json!({ "ok": true, "extension": extension }))
}

#[tauri::command]
pub async fn uninstall_extension(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.uninstall_extension(&id).await?;
    Ok(json!({ "ok": true, "id": id }))
}

#[tauri::command]
pub async fn update_extension_settings(
    state: State<'_, Arc<AppState>>,
    id: String,
    settings: HashMap<String, Value>,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.update_extension_settings(&id, settings).await?;
    Ok(json!({ "ok": true, "id": id }))
}

#[tauri::command]
pub async fn get_extension_settings(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ExtensionFeatures, CoreError> {
    let manager = state.extension_manager.read().await;

    Ok(manager
        .get_settings(&id)
        .await
        .unwrap_or_else(|_| ExtensionFeatures {
            episode_servers: Some(vec!["default".into()]),
            supports_dub: Some(false),
        }))
}

#[tauri::command]
pub async fn get_extension_filters(
    state: State<'_, Arc<AppState>>,
    name: String,
) -> Result<Value, CoreError> {
    let manager = state.extension_manager.read().await;

    let filters = manager
        .get_filters(&name)
        .await
        .unwrap_or_default();
    
    Ok(json!({ "filters": filters }))
}

#[tauri::command]
pub async fn update_extension(
    state: State<'_, Arc<AppState>>,
    id: String,
    manifest_url: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.update_extension(state.inner(), &id, &manifest_url).await?;
    Ok(json!({ "ok": true, "extension": extension }))
}

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
    query: String,
    filters: Value,
    page: u32,
) -> Result<Value, CoreError> {
    let user_id = require_auth(&session).await?;
    let manager = state.inner().extension_manager.read().await;
    let picked = TorrentService::auto_select_torrent(
        state.inner(), &manager, user_id, &id, &query, filters, page,
    ).await?;
    Ok(json!({ "torrent": picked }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStreamInfo {
    pub session_id: String,
    pub url: String,
    pub total_size: u64,
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