use std::sync::Arc;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;
use hoshi_core::AppState;
use hoshi_core::error::CoreError;
use hoshi_core::extensions::types::{Extension, ExtensionType};
use crate::commands::extensions::ExtensionsResponse;

#[tauri::command]
pub async fn list_dev_extensions(
    state: State<'_, Arc<AppState>>,
) -> Result<ExtensionsResponse<Vec<Extension>>, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let list = manager.list_dev_extensions().iter().map(|e| (*e).clone()).collect();
    Ok(ExtensionsResponse { extensions: list })
}

#[tauri::command]
pub async fn create_dev_extension(
    state: State<'_, Arc<AppState>>,
    name: String,
    ext_type: ExtensionType,
    starter_code: String,
) -> Result<Extension, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.create_dev_extension(&name, ext_type, &starter_code).await
}

#[tauri::command]
pub async fn read_extension_source(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<String, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    manager.read_extension_source(&id).await
}

#[tauri::command]
pub async fn write_extension_source(
    state: State<'_, Arc<AppState>>,
    id: String,
    code: String,
) -> Result<Value, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    manager.write_extension_source(&id, code).await?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
pub async fn read_manifest_raw(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<String, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    manager.read_manifest_raw(&id).await
}

#[tauri::command]
pub async fn write_manifest_raw(
    state: State<'_, Arc<AppState>>,
    id: String,
    yaml: String,
) -> Result<Extension, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.write_manifest_raw(&id, yaml).await
}

#[tauri::command]
pub async fn delete_dev_extension(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Value, CoreError> {
    let mut manager = state.inner().extension_manager.write().await;
    manager.delete_dev_extension(&id).await?;
    Ok(json!({ "ok": true, "id": id }))
}

#[derive(Serialize)]
pub struct RunResult {
    result: Option<Value>,
    console: Vec<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn run_extension_function(
    state: State<'_, Arc<AppState>>,
    extension_id: String,
    function_name: String,
    args: Vec<Value>,
) -> Result<RunResult, CoreError> {
    let manager = state.inner().extension_manager.read().await;
    let http_client = state.inner().http_client.clone();

    let (result, console) = manager
        .call_extension_function_with_console(&extension_id, &function_name, args, http_client)
        .await;

    match result {
        Ok(value) => Ok(RunResult { result: Some(value), console, error: None }),
        Err(e) => Ok(RunResult { result: None, console, error: Some(e.to_string()) }),
    }
}