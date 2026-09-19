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

        let manifest_yaml = format!(
            "id: {id}\nname: {name}\nversion: {version}\ntype: novel\nlanguage: {lang}\nicon: {icon}\nsite: {site}\nauthor: lnreader\nmain: index.js\nsource: lnreader\n",
            id = prefixed_id,
            name    = entry.name,
            version = entry.version,
            lang    = entry.lang,
            icon    = entry.icon_url,
            site    = entry.site,
        );

        debug!(manifest = %manifest_yaml, "Deserializing generated manifest");

        let ext_dir = self.extensions_dir.join(&prefixed_id);
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_yaml)
            .await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("index.js"), &script)
            .await.map_err(CoreError::Io)?;

        let manifest: ExtensionManifest = serde_yaml::from_str(&manifest_yaml)
            .map_err(|e| {
                error!(error = ?e, "Generated manifest is invalid");
                CoreError::Parse("error.extension.invalid_manifest".into())
            })?;

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

        let manifest_yaml = format!(
            "id: {id}\nname: {name}\nversion: {version}\ntype: {ext_type}\nlanguage: {lang}\nicon: {icon}\nauthor: {author}\nmain: index.js\nsource: sora\nnote: {note}\nsoftsub: {softsub}\n",
            id       = prefixed_id,
            name     = entry.source_name,
            version  = module_manifest.version,
            ext_type = normalize_sora_type(&entry.ext_type),
            lang     = entry.language,
            icon     = entry.icon_url,
            author   = entry.author.name,
            note     = module_manifest.note.clone().unwrap_or_default(),
            softsub  = module_manifest.softsub,
        );

        let ext_dir = self.extensions_dir.join(&prefixed_id);
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("manifest.yaml"), &manifest_yaml).await.map_err(CoreError::Io)?;
        fs::write(ext_dir.join("index.js"), &script).await.map_err(CoreError::Io)?;

        let manifest: ExtensionManifest = serde_yaml::from_str(&manifest_yaml)
            .map_err(|e| { error!(error = ?e, "Generated manifest is invalid"); CoreError::Parse("error.extension.invalid_manifest".into()) })?;

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

        let manifest: ExtensionManifest = serde_yaml::from_slice(&manifest_bytes)
            .map_err(|e| {
                error!(error = ?e, "Downloaded manifest contains invalid YAML");
                CoreError::Parse("error.extension.invalid_manifest".into())
            })?;

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

        let ext_dir = self.extensions_dir.join(&manifest.id);

        fs::create_dir_all(&ext_dir).await.map_err(|e| {
            error!(error = ?e, path = %ext_dir.display(), "Failed to create extension directory");
            CoreError::Io(e)
        })?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_bytes).await.map_err(CoreError::Io)?;

        let script_filename = manifest.main.rsplit('/').next().unwrap_or("index.js");
        let script_path = ext_dir.join(script_filename);
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
        let preserved_settings = self.extensions.get(id)
            .map(|e| e.settings.clone())
            .unwrap_or_default();

        let mut extension = self.install_extension(state, manifest_url).await?;

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
        if !self.extensions.contains_key(id) {
            warn!(ext = %id, "Attempted to uninstall a non-existent extension");
            return Err(CoreError::NotFound("error.extension.not_found".into()));
        }

        let ext_dir = self.extensions_dir.join(id);
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir).await.map_err(|e| {
                error!(error = ?e, "Failed to delete extension directory");
                CoreError::Io(e)
            })?;
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