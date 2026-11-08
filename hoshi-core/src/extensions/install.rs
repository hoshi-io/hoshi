use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, info, instrument, warn};

use super::{load_settings, persist_settings, ExtensionManager};
use crate::error::{CoreError, CoreResult};
use crate::extensions::types::{
    normalize_sora_type, Extension, ExtensionManifest, ExtensionType, LNReaderMarketplaceEntry,
    SoraMarketplaceEntry, SoraModuleManifest,
};
use crate::state::AppState;

/// Extension ids become directory names under the extensions folder (and are
/// later passed to `remove_dir_all` on uninstall), and they come from remote
/// manifests / marketplace JSON. Only allow a plain, single path component.
pub(crate) fn validate_extension_id(id: &str) -> CoreResult<()> {
    let ok = !id.is_empty()
        && id.len() <= 128
        && !id.starts_with('.')
        && id.chars().all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        Ok(())
    } else {
        warn!(id = %id, "Rejected extension id: must be a plain name ([A-Za-z0-9._-], not starting with '.')");
        Err(CoreError::Validation("error.extension.invalid_id".into()))
    }
}

/// The script file name inside the extension folder: one path component ending in `.js`.
pub(crate) fn validate_script_file(name: &str) -> CoreResult<()> {
    let ok = name.ends_with(".js")
        && !name.starts_with('.')
        && !name.contains(['/', '\\', ':'])
        && name.len() <= 128;
    if ok {
        Ok(())
    } else {
        warn!(main = %name, "Rejected extension script name");
        Err(CoreError::Validation("error.extension.invalid_script".into()))
    }
}

/// Builds a manifest as real YAML (never by string formatting, which breaks on
/// names containing ": ", "#", quotes or newlines), with optional extra keys.
fn manifest_yaml(manifest: &ExtensionManifest, extra: &[(&str, serde_yaml::Value)]) -> CoreResult<String> {
    let mut value = serde_yaml::to_value(manifest)
        .map_err(|e| { error!(error = ?e, "Failed to serialize manifest"); CoreError::Internal("error.extension.invalid_manifest".into()) })?;
    if let serde_yaml::Value::Mapping(map) = &mut value {
        for (k, v) in extra {
            map.insert(serde_yaml::Value::String((*k).to_string()), v.clone());
        }
    }
    serde_yaml::to_string(&value)
        .map_err(|e| { error!(error = ?e, "Failed to serialize manifest"); CoreError::Internal("error.extension.invalid_manifest".into()) })
}

impl Extension {
    fn from_manifest(
        manifest: ExtensionManifest,
        script_path: PathBuf,
        settings: HashMap<String, Value>,
    ) -> Self {
        Self {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "Unknown".to_string()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path,
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings,
            settings,
            source: manifest.source,
            dev: manifest.dev,
        }
    }
}

impl ExtensionManager {
    #[instrument(skip(self, state, entry))]
    pub async fn install_lnreader_extension(
        &mut self,
        state: &AppState,
        entry: LNReaderMarketplaceEntry,
    ) -> CoreResult<Extension> {
        info!(id = %entry.id, "Installing LNReader extension");

        let script = state.http_client
            .get(&entry.url)
            .send()
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to download LNReader plugin JS");
                CoreError::Network("error.extension.install_network_failed".into())
            })?
            .text()
            .await
            .map_err(|_| CoreError::Network("error.extension.install_network_failed".into()))?;
        let prefixed_id = format!("lnr_{}", entry.id);
        validate_extension_id(&prefixed_id)?;

        let manifest = ExtensionManifest {
            id: prefixed_id.clone(),
            name: entry.name.clone(),
            version: entry.version.clone(),
            author: Some("lnreader".into()),
            ext_type: ExtensionType::Novel,
            main: "index.js".into(),
            icon: Some(entry.icon_url.clone()),
            language: entry.lang.clone(),
            nsfw: false,
            skip_default_processing: false,
            settings: vec![],
            source: Some("lnreader".into()),
            dev: false,
        };
        let manifest_yaml = manifest_yaml(&manifest, &[("site", entry.site.clone().into())])?;

        let ext_dir = self.extensions_dir.join(&prefixed_id);
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_yaml)
            .await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("index.js"), &script)
            .await.map_err(CoreError::Io)?;

        let settings = load_settings(&ext_dir, &manifest.settings).await;
        let script_path = ext_dir.join("index.js");
        let extension = Extension::from_manifest(manifest, script_path, settings);

        self.extensions.insert(extension.id.clone(), extension.clone());
        info!(ext = %extension.id, "LNReader extension installed successfully");

        Ok(extension)
    }

    #[instrument(skip(self, state, entry))]
    pub async fn install_sora_extension(
        &mut self,
        state: &AppState,
        entry: SoraMarketplaceEntry,
    ) -> CoreResult<Extension> {
        info!(id = %entry.id, "Installing Sora extension");

        let module_manifest: SoraModuleManifest = state.http_client
            .get(&entry.manifest_url)
            .send()
            .await
            .map_err(|e| { error!(error = ?e, "Failed to fetch Sora manifest"); CoreError::Network("error.extension.install_network_failed".into()) })?
            .json()
            .await
            .map_err(|e| { error!(error = ?e, "Invalid Sora manifest JSON"); CoreError::Parse("error.extension.invalid_manifest".into()) })?;

        let script = state.http_client
            .get(&module_manifest.script_url)
            .send()
            .await
            .map_err(|e| { error!(error = ?e, "Failed to download Sora module JS"); CoreError::Network("error.extension.install_network_failed".into()) })?
            .text()
            .await
            .map_err(|_| CoreError::Network("error.extension.install_network_failed".into()))?;

        let prefixed_id = format!("sora_{}", entry.id);
        validate_extension_id(&prefixed_id)?;

        let ext_type: ExtensionType = serde_yaml::from_str(normalize_sora_type(&entry.ext_type))
            .unwrap_or(ExtensionType::Unknown);
        let manifest = ExtensionManifest {
            id: prefixed_id.clone(),
            name: entry.source_name.clone(),
            version: module_manifest.version.clone(),
            author: Some(entry.author.name.clone()),
            ext_type,
            main: "index.js".into(),
            icon: Some(entry.icon_url.clone()),
            language: entry.language.clone(),
            nsfw: false,
            skip_default_processing: false,
            settings: vec![],
            source: Some("sora".into()),
            dev: false,
        };
        let manifest_yaml = manifest_yaml(&manifest, &[
            ("note", module_manifest.note.clone().unwrap_or_default().into()),
            ("softsub", module_manifest.softsub.into()),
        ])?;

        let ext_dir = self.extensions_dir.join(&prefixed_id);
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("manifest.yaml"), &manifest_yaml).await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("index.js"), &script).await.map_err(CoreError::Io)?;

        let settings = load_settings(&ext_dir, &manifest.settings).await;
        let script_path = ext_dir.join("index.js");

        let mut extension = Extension::from_manifest(manifest, script_path, settings);
        extension.nsfw = false;

        self.extensions.insert(extension.id.clone(), extension.clone());
        info!(ext = %extension.id, "Sora extension installed successfully");
        Ok(extension)
    }

    #[instrument(skip(self, state, manifest_url))]
    pub async fn install_extension(&mut self, state: &AppState, manifest_url: &str) -> CoreResult<Extension> {
        self.install_extension_checked(state, manifest_url, None).await
    }

    /// Downloads and validates everything first, writes to disk last.
    /// `expected_id`: on update, the new manifest must keep the same id.
    async fn install_extension_checked(
        &mut self,
        state: &AppState,
        manifest_url: &str,
        expected_id: Option<&str>,
    ) -> CoreResult<Extension> {
        info!(url = %manifest_url, "Starting extension installation");

        let response = state
            .http_client
            .get(manifest_url)
            .send()
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to connect to manifest URL");
                CoreError::Network("error.extension.install_network_failed".into())
            })?;

        if !response.status().is_success() {
            error!(status = %response.status(), url = %manifest_url, "Manifest server returned HTTP error");
            return Err(CoreError::Network("error.extension.install_network_failed".into()));
        }

        let manifest_bytes = response.bytes().await
            .map_err(|_e| CoreError::Network("error.extension.install_network_failed".into()))?;

        let mut manifest: ExtensionManifest = serde_yaml::from_slice(&manifest_bytes)
            .map_err(|e| {
                error!(error = ?e, "Downloaded manifest contains invalid YAML");
                CoreError::Parse("error.extension.invalid_manifest".into())
            })?;

        validate_extension_id(&manifest.id)?;

        if let Some(expected) = expected_id {
            if manifest.id != expected {
                warn!(expected = %expected, got = %manifest.id, "Extension update rejected: manifest id changed");
                return Err(CoreError::Validation("error.extension.id_mismatch".into()));
            }
        }

        if manifest.ext_type == ExtensionType::Unknown {
            warn!(ext = %manifest.id, "Extension rejected: Unsupported type declared");
            return Err(CoreError::Validation("error.extension.unsupported_type".into()));
        }

        if !manifest.main.ends_with(".js") {
            warn!(ext = %manifest.id, "Extension rejected: Main script is not .js");
            return Err(CoreError::Validation("error.extension.invalid_script".into()));
        }

        let script_url = if manifest.main.starts_with("http://") || manifest.main.starts_with("https://") {
            manifest.main.clone()
        } else {
            let base = manifest_url.rsplit_once('/').map(|(b, _)| b).unwrap_or(manifest_url);
            format!("{}/{}", base, manifest.main)
        };

        let script_filename = manifest.main
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("index.js")
            .to_string();
        validate_script_file(&script_filename)?;

        debug!(ext = %manifest.id, url = %script_url, "Downloading extension script");
        let script_response = state
            .http_client
            .get(&script_url)
            .send()
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to connect to script URL");
                CoreError::Network("error.extension.install_network_failed".into())
            })?;

        if !script_response.status().is_success() {
            error!(status = %script_response.status(), url = %script_url, "Script server returned HTTP error");
            return Err(CoreError::Network("error.extension.install_network_failed".into()));
        }

        let script_bytes = script_response.bytes().await
            .map_err(|_| CoreError::Network("error.extension.install_network_failed".into()))?;

        // Keep the remote manifest as-is (unknown keys included), except `main`.
        let mut saved_manifest: serde_yaml::Value = serde_yaml::from_slice(&manifest_bytes)
            .map_err(|_| CoreError::Parse("error.extension.invalid_manifest".into()))?;
        if let serde_yaml::Value::Mapping(map) = &mut saved_manifest {
            map.insert("main".into(), script_filename.clone().into());
        }
        let saved_manifest = serde_yaml::to_string(&saved_manifest)
            .map_err(|_| CoreError::Internal("error.extension.invalid_manifest".into()))?;
        manifest.main = script_filename.clone();

        let ext_dir = self.extensions_dir.join(&manifest.id);

        fs::create_dir_all(&ext_dir).await.map_err(|e| {
            error!(error = ?e, path = %ext_dir.display(), "Failed to create extension directory");
            CoreError::Io(e)
        })?;

        fs::write(ext_dir.join("manifest.yaml"), &saved_manifest).await.map_err(CoreError::Io)?;

        let script_path = ext_dir.join(&script_filename);
        fs::write(&script_path, &script_bytes).await.map_err(CoreError::Io)?;

        let settings = load_settings(&ext_dir, &manifest.settings).await;
        persist_settings(&ext_dir, &settings).await;

        let mut extension = Extension::from_manifest(manifest, script_path, settings);
        extension.source = None;

        self.extensions.insert(extension.id.clone(), extension.clone());
        info!(ext = %extension.id, "Extension installed and loaded successfully");

        Ok(extension)
    }

    #[instrument(skip(self, state, manifest_url))]
    pub async fn update_extension(&mut self, state: &AppState, id: &str, manifest_url: &str) -> CoreResult<Extension> {
        validate_extension_id(id)?;
        let preserved_settings = self.extensions.get(id)
            .map(|e| e.settings.clone())
            .unwrap_or_default();

        // Same id is enforced, so the install writes into this extension's own folder.
        let mut extension = self.install_extension_checked(state, manifest_url, Some(id)).await?;

        for (key, value) in preserved_settings {
            extension.settings.entry(key).or_insert(value);
        }

        let ext_dir = self.extensions_dir.join(id);
        persist_settings(&ext_dir, &extension.settings).await;

        self.extensions.insert(id.to_string(), extension.clone());

        info!(ext = %id, "Extension updated successfully");
        Ok(extension)
    }

    #[instrument(skip(self))]
    pub async fn uninstall_extension(&mut self, id: &str) -> CoreResult<()> {
        let Some(ext) = self.extensions.get(id) else {
            warn!(ext = %id, "Attempted to uninstall a non-existent extension");
            return Err(CoreError::NotFound("error.extension.not_found".into()));
        };

        // Delete the folder the extension was actually loaded from, and only if it
        // is a direct child of the extensions folder. Never build the path from the id.
        let ext_dir = ext.script_path.parent().map(|p| p.to_path_buf());
        let safe_dir = ext_dir.filter(|d| {
            d.parent() == Some(self.extensions_dir.as_path())
                && d.file_name().is_some_and(|n| n != "." && n != "..")
        });
        match safe_dir {
            Some(dir) if dir.exists() => {
                fs::remove_dir_all(&dir).await.map_err(|e| {
                    error!(error = ?e, "Failed to delete extension directory");
                    CoreError::Io(e)
                })?;
            }
            Some(_) => {}
            None => {
                error!(ext = %id, "Refusing to delete extension folder outside the extensions directory");
                return Err(CoreError::Validation("error.extension.invalid_id".into()));
            }
        }

        self.extensions.remove(id);

        if let Ok(mut store) = self.extension_state.lock() {
            store.remove(id);
        }

        info!(ext = %id, "Extension uninstalled successfully");
        Ok(())
    }

    #[instrument(skip(self, updates))]
    pub async fn update_extension_settings(
        &mut self,
        id: &str,
        updates: HashMap<String, Value>,
    ) -> CoreResult<()> {
        let extension = self.extensions.get_mut(id).ok_or_else(|| {
            warn!(ext = %id, "Attempted to update settings for a non-existent extension");
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        for (key, value) in updates {
            extension.settings.insert(key, value);
        }

        let ext_dir = self.extensions_dir.join(id);
        persist_settings(&ext_dir, &extension.settings).await;

        debug!(ext = %id, "Extension settings updated successfully");
        Ok(())
    }
}