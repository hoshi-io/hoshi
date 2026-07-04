use hoshi_core::{
    error::CoreError,
    state::AppState,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use hoshi_core::extensions::types::{Extension, ExtensionFeatures, LNReaderMarketplaceEntry, SoraMarketplaceEntry, TachiyomiMarketplaceEntry};

#[derive(Serialize)]
pub struct ExtensionsResponse<T> {
    extensions: T,
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
pub async fn install_tachiyomi_extension(
    state: State<'_, Arc<AppState>>,
    download_url: String,
    entry: TachiyomiMarketplaceEntry,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    let extension = manager.install_tachiyomi_extension(state.inner(), &download_url, entry).await?;
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
pub async fn get_image_request_headers(
    state: State<'_, Arc<AppState>>,
    extension_id: String,
    image_url: String,
    chapter_url: String,
) -> Result<HashMap<String, String>, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    manager.get_image_request_headers(&extension_id, &image_url, &chapter_url).await
}