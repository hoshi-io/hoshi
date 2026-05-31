use std::fs;
use tracing::{info, instrument};

use crate::config::model::MpvConfig;
use crate::error::{CoreError, CoreResult};
use crate::paths::AppPaths;
use crate::state::AppState;

use super::registry::{OscDest, OSC_REGISTRY};

pub struct MpvConfigService;

impl MpvConfigService {
    #[instrument(skip(state))]
    pub async fn download_osc(state: &AppState, user_id: i32, name: &str) -> CoreResult<()> {
        let bundle = OSC_REGISTRY
            .iter()
            .find(|b| b.name == name)
            .ok_or_else(|| CoreError::BadRequest("error.mpv.unknown_osc".into()))?;

        for file in bundle.files {
            let bytes = state
                .http_client
                .get(file.url)
                .send()
                .await
                .map_err(|e| CoreError::Network(e.to_string()))?
                .bytes()
                .await
                .map_err(|e| CoreError::Network(e.to_string()))?;

            let subdir = match file.dest {
                OscDest::Scripts => "scripts",
                OscDest::ScriptOpts => "script-opts",
                OscDest::Fonts => "fonts",
            };

            let filename = file.url.split('/').last().unwrap_or("unknown");
            let dest = state.paths.mpv_path
                .join("oscs").join(name)
                .join(subdir).join(filename);

            fs::create_dir_all(dest.parent().unwrap()).map_err(CoreError::Io)?;
            fs::write(&dest, bytes).map_err(CoreError::Io)?;
            info!(file = %filename, osc = %name, "Downloaded OSC file");
        }

        let mut config = crate::config::service::ConfigService::get_config(state, user_id).await?;
        config.mpv.active_osc = Some(name.to_string());

        Self::apply(&state.paths, &config.mpv)?;

        Ok(())
    }

    #[instrument(skip(state))]
    pub async fn download_script(state: &AppState, url: &str) -> CoreResult<String> {
        let filename = url
            .split('/')
            .last()
            .ok_or_else(|| CoreError::BadRequest("error.mpv.invalid_script_url".into()))?
            .to_string();

        let bytes = state
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let dest = state
            .paths
            .mpv_path
            .join("scripts-available")
            .join(&filename);

        fs::create_dir_all(dest.parent().unwrap()).map_err(CoreError::Io)?;
        fs::write(&dest, bytes).map_err(CoreError::Io)?;
        info!(script = %filename, "Downloaded standalone script");

        Ok(filename)
    }

    #[instrument(skip(paths, config))]
    pub fn apply(paths: &AppPaths, config: &MpvConfig) -> CoreResult<()> {
        Self::sync_osc(paths, config.active_osc.as_deref())?;
        Self::sync_scripts(paths, &config.enabled_scripts)?;
        Self::write_mpv_conf(paths, &config.extra_options)?;
        Ok(())
    }

    pub fn list_oscs() -> Vec<&'static str> {
        OSC_REGISTRY.iter().map(|b| b.name).collect()
    }

    pub fn list_downloaded_oscs(paths: &AppPaths) -> CoreResult<Vec<String>> {
        let oscs_dir = paths.mpv_path.join("oscs");
        if !oscs_dir.exists() {
            return Ok(vec![]);
        }
        let names = fs::read_dir(&oscs_dir)
            .map_err(CoreError::Io)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.file_name() != ".active")
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        Ok(names)
    }

    pub fn list_available_scripts(paths: &AppPaths) -> CoreResult<Vec<String>> {
        let dir = paths.mpv_path.join("scripts-available");
        if !dir.exists() {
            return Ok(vec![]);
        }
        let names = fs::read_dir(&dir)
            .map_err(CoreError::Io)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |x| x == "lua" || x == "js")
            })
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        Ok(names)
    }

    fn sync_osc(paths: &AppPaths, osc: Option<&str>) -> CoreResult<()> {
        let active_marker = paths.mpv_path.join("oscs").join(".active");

        if active_marker.exists() {
            let prev = fs::read_to_string(&active_marker).map_err(CoreError::Io)?;
            let prev_bundle = paths.mpv_path.join("oscs").join(prev.trim());
            if prev_bundle.exists() {
                Self::remove_bundle_files(paths, &prev_bundle)?;
            }
        }

        if let Some(name) = osc {
            let bundle_path = paths.mpv_path.join("oscs").join(name);
            if !bundle_path.exists() {
                return Err(CoreError::NotFound(format!(
                    "OSC '{}' not downloaded yet",
                    name
                )));
            }
            Self::copy_bundle_files(paths, &bundle_path)?;
            fs::write(&active_marker, name).map_err(CoreError::Io)?;
            info!(osc = %name, "OSC activated");
        } else {
            let _ = fs::remove_file(&active_marker);
        }

        Ok(())
    }

    fn copy_bundle_files(paths: &AppPaths, bundle_path: &std::path::Path) -> CoreResult<()> {
        for subdir in &["scripts", "script-opts", "fonts"] {
            let src = bundle_path.join(subdir);
            if !src.exists() {
                continue;
            }
            let dest = paths.mpv_path.join(subdir);
            fs::create_dir_all(&dest).map_err(CoreError::Io)?;
            for entry in fs::read_dir(&src).map_err(CoreError::Io)? {
                let entry = entry.map_err(CoreError::Io)?;
                fs::copy(entry.path(), dest.join(entry.file_name())).map_err(CoreError::Io)?;
                info!(file = ?entry.file_name(), subdir = %subdir, "Copied bundle file to active dir");
            }
        }
        Ok(())
    }

    fn remove_bundle_files(paths: &AppPaths, bundle_path: &std::path::Path) -> CoreResult<()> {
        for subdir in &["scripts", "script-opts", "fonts"] {
            let src = bundle_path.join(subdir);
            if !src.exists() {
                continue;
            }
            let dest = paths.mpv_path.join(subdir);
            for entry in fs::read_dir(&src).map_err(CoreError::Io)? {
                let entry = entry.map_err(CoreError::Io)?;
                let target = dest.join(entry.file_name());
                if target.exists() {
                    fs::remove_file(&target).map_err(CoreError::Io)?;
                }
            }
        }
        Ok(())
    }

    fn sync_scripts(paths: &AppPaths, enabled: &[String]) -> CoreResult<()> {
        let available = paths.mpv_path.join("scripts-available");
        let active = paths.mpv_path.join("scripts");
        fs::create_dir_all(&active).map_err(CoreError::Io)?;

        if active.exists() {
            for entry in fs::read_dir(&active).map_err(CoreError::Io)? {
                let entry = entry.map_err(CoreError::Io)?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if !enabled.contains(&name) && available.join(&name).exists() {
                    fs::remove_file(entry.path()).map_err(CoreError::Io)?;
                }
            }
        }

        // Copy newly enabled scripts
        for script_name in enabled {
            let dest = active.join(script_name);
            if !dest.exists() {
                let src = available.join(script_name);
                if src.exists() {
                    fs::copy(&src, &dest).map_err(CoreError::Io)?;
                    info!(script = %script_name, "Script enabled");
                }
            }
        }

        Ok(())
    }

    fn write_mpv_conf(
        paths: &AppPaths,
        extra_options: &std::collections::HashMap<String, String>,
    ) -> CoreResult<()> {
        let mut lines = vec![
            "# Generated by Hoshi - do not edit manually".to_string(),
            "osc=no".to_string(),
        ];

        for (k, v) in extra_options {
            lines.push(format!("{}={}", k, v));
        }

        fs::write(paths.mpv_path.join("mpv.conf"), lines.join("\n"))
            .map_err(CoreError::Io)?;
        Ok(())
    }
}