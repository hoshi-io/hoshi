use tauri::{AppHandle, Manager, path::BaseDirectory};
use hoshi_core::error::CoreError;

fn shader_files(mode: &str, tier: &str) -> Vec<&'static str> {
    match (mode, tier) {
        ("off", _) => vec![],

        ("a", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_M.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("a", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_VL.glsl",
            "Anime4K_Upscale_CNN_x2_VL.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        ("b", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_Soft_M.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("b", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_Soft_VL.glsl",
            "Anime4K_Upscale_CNN_x2_VL.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        ("c", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Upscale_Denoise_CNN_x2_M.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("c", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Upscale_Denoise_CNN_x2_VL.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        ("aa", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_M.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl", "Anime4K_Restore_CNN_S.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("aa", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_VL.glsl",
            "Anime4K_Upscale_CNN_x2_VL.glsl", "Anime4K_Restore_CNN_M.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        ("bb", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_Soft_M.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Restore_CNN_Soft_S.glsl",
            "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("bb", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Restore_CNN_Soft_VL.glsl",
            "Anime4K_Upscale_CNN_x2_VL.glsl", "Anime4K_AutoDownscalePre_x2.glsl",
            "Anime4K_AutoDownscalePre_x4.glsl", "Anime4K_Restore_CNN_Soft_M.glsl",
            "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        ("ca", "fast") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Upscale_Denoise_CNN_x2_M.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Restore_CNN_S.glsl", "Anime4K_Upscale_CNN_x2_S.glsl",
        ],
        ("ca", "hq") => vec![
            "Anime4K_Clamp_Highlights.glsl", "Anime4K_Upscale_Denoise_CNN_x2_VL.glsl",
            "Anime4K_AutoDownscalePre_x2.glsl", "Anime4K_AutoDownscalePre_x4.glsl",
            "Anime4K_Restore_CNN_M.glsl", "Anime4K_Upscale_CNN_x2_M.glsl",
        ],

        _ => vec![], // unknown combo, treat as off rather than erroring
    }
}

pub fn resolve_shader_chain(app: &AppHandle, mode: &str, tier: &str) -> Result<String, CoreError> {
    if tier == "off" {
        return Ok(String::new());
    }
    let files = shader_files(mode, tier);
    if files.is_empty() {
        return Ok(String::new()); // clears glsl-shaders, same as mpv's own CTRL+0 binding
    }

    let resource_dir = app
        .path()
        .resolve("resources/anime4k", BaseDirectory::Resource)
        .map_err(|e| CoreError::Internal(format!("failed to resolve anime4k resource dir: {e}")))?;

    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    let joined = files
        .iter()
        .map(|f| resource_dir.join(f).display().to_string())
        .collect::<Vec<_>>()
        .join(sep);

    Ok(joined)
}