//! TEMPORARY: layering check (.github/workflows/smoke.yml). Inert unless HOSHI_SMOKE_VIDEO is set.

#[tauri::command]
pub fn smoke_video() -> Option<String> {
    std::env::var("HOSHI_SMOKE_VIDEO").ok()
}
