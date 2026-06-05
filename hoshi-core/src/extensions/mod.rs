mod sandbox;
pub mod types;

use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::de::DeserializeOwned;
use tokio::fs;
use tracing::{debug, error, info, instrument, warn};
use types::{Extension, ExtensionManifest, ExtensionType, SettingDefinition};

pub type ExtensionStateStore = Arc<Mutex<HashMap<String, HashMap<String, Value>>>>;

use crate::error::{CoreError, CoreResult};
use crate::extensions::types::{Chapter, CompatLayer, Episode, EpisodeSource, ExtensionFeatures, ExtensionFilters, ExtensionMetadata, ExtensionSearchResult, LNReaderMarketplaceEntry, Page, TachiyomiMarketplaceEntry};
use crate::headless::{noop_headless, HeadlessHandle};
use crate::paths::AppPaths;
use crate::state::AppState;

const BASE: &str  = include_str!("base/Base.js");
const ANIME: &str = include_str!("base/Anime.js");
const MANGA: &str = include_str!("base/Manga.js");
const NOVEL: &str = include_str!("base/Novel.js");

const TACHIYOMI: &str = include_str!("compatibility/tachiyomi.js");
const LNREADER: &str = include_str!("compatibility/lnreader.js");
const SANDBOX_BOOTSTRAP: &str = include_str!("sandbox_bootstrap.js");

pub struct ExtensionManager {
    extensions: HashMap<String, Extension>,
    extensions_dir: PathBuf,
    headless: HeadlessHandle,
    extension_state: ExtensionStateStore,
}

impl ExtensionManager {
    pub fn new(paths: &AppPaths) -> CoreResult<Self> {
        let extensions_dir = paths.base_dir.join("extensions");
        Ok(Self {
            extensions: HashMap::new(),
            extensions_dir,
            headless: noop_headless(),
            extension_state: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn list_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().collect()
    }

    pub fn get_extensions_by_type(&self, target_type: ExtensionType) -> Vec<String> {
        self.extensions.values()
            .filter(|e| e.ext_type == target_type)
            .map(|e| e.id.clone())
            .collect()
    }

    pub fn is_nsfw(&self, extension_id: &str) -> bool {
        self.extensions
            .get(extension_id)
            .map(|ext| ext.nsfw)
            .unwrap_or(false)
    }

    pub fn skip_default_processing(&self, extension_id: &str) -> bool {
        self.extensions
            .get(extension_id)
            .map(|ext| ext.skip_default_processing)
            .unwrap_or(false)
    }

    pub fn content_type(&self, extension_id: &str) -> crate::content::models::ContentType {
        use crate::content::models::ContentType;
        match self.extensions.get(extension_id).map(|e| &e.ext_type) {
            Some(ExtensionType::Manga) => ContentType::Manga,
            Some(ExtensionType::Novel) => ContentType::Novel,
            _ => ContentType::Anime,
        }
    }

    pub fn set_headless(&mut self, headless: HeadlessHandle) {
        self.headless = headless;
    }

    #[instrument(skip(self))]
    pub async fn load_extensions(&mut self) -> CoreResult<()> {
        let mut entries = fs::read_dir(&self.extensions_dir).await.map_err(CoreError::Io)?;
        let mut loaded_count = 0;

        while let Some(entry) = entries.next_entry().await.map_err(CoreError::Io)? {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("manifest.yaml");
            if !manifest_path.exists() { continue; }

            let yaml_content = match fs::read_to_string(&manifest_path).await {
                Ok(c) => c,
                Err(e) => {
                    warn!(path = %manifest_path.display(), error = ?e, "Could not read manifest file");
                    continue;
                }
            };

            let manifest: ExtensionManifest = match serde_yaml::from_str(&yaml_content) {
                Ok(m) => m,
                Err(e) => {
                    error!(path = %manifest_path.display(), error = ?e, "Invalid YAML format in manifest");
                    continue;
                }
            };

            let script_path = path.join(&manifest.main);
            if !script_path.exists() {
                error!(ext = %manifest.id, expected_path = %script_path.display(), "Main JS file declared in manifest is missing");
                continue;
            }

            match script_path.extension().and_then(|e| e.to_str()) {
                Some("js") => {}
                _ => {
                    warn!(ext = %manifest.id, script = %script_path.display(), "Only .js extension scripts are supported");
                    continue;
                }
            }

            let settings = load_settings(&path, &manifest.settings).await;

            let extension = Extension {
                id: manifest.id.clone(),
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
                source: manifest.source
            };

            self.extensions.insert(manifest.id, extension);
            loaded_count += 1;
        }

        info!(count = loaded_count, "Extensions loaded from disk successfully");
        Ok(())
    }

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

        let extension = Extension {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "Unknown".to_string()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path: ext_dir.join("index.js"),
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings,
            settings,
            source: manifest.source,
        };

        self.extensions.insert(manifest.id.clone(), extension.clone());
        info!(ext = %extension.id, "LNReader extension installed successfully");

        Ok(extension)
    }

    pub async fn install_tachiyomi_extension(
        &mut self,
        state: &AppState,
        download_url: &str,
        entry: TachiyomiMarketplaceEntry,
    ) -> CoreResult<Extension> {
        let bytes = state.http_client
            .get(download_url)
            .send().await
            .map_err(|e| CoreError::Network(e.to_string()))?
            .bytes().await
            .map_err(|e| CoreError::Network(e.to_string()))?
            .to_vec();

        let translated = tokio::task::spawn_blocking(move || {
            apktojs::apk_to_js(&bytes)
        })
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        let prefixed_id = format!("tachi_{}", entry.pkg);
        let ext_dir = self.extensions_dir.join(&prefixed_id);
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;

        let js_path = ext_dir.join("index.js");
        fs::write(&js_path, &translated.js).await.map_err(CoreError::Io)?;

        let nsfw = entry.nsfw != 0;
        let icon_url = entry.icon_url.clone().unwrap_or_else(|| {
            format!("{}/icon/{}.png", entry.repo_url, entry.pkg)
        });

        let staging_extension = Extension {
            id: prefixed_id.clone(),
            name: entry.sources.first().map(|s| s.name.clone()).unwrap_or_else(|| entry.name.clone()),
            version: entry.version.clone(),
            author: "tachiyomi".to_string(),
            icon: Option::from(icon_url.clone()),
            ext_type: ExtensionType::Manga,
            script_path: js_path.clone(),
            language: entry.lang.clone(),
            nsfw,
            skip_default_processing: false,
            setting_definitions: vec![],
            settings: HashMap::new(),
            source: Some("tachiyomi".to_string()),
        };
        self.extensions.insert(prefixed_id.clone(), staging_extension);

        let unique_langs: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            entry.sources.iter()
                .map(|s| s.lang.clone())
                .filter(|l| seen.insert(l.clone()))
                .collect()
        };

        let mut settings: Vec<Value> = if unique_langs.len() > 1 {
            vec![json!({
                "key": "language",
                "label": "Language",
                "type": "select",
                "default": unique_langs[0],
                "options": unique_langs.iter().map(|l| json!({
                    "value": l,
                    "label": l.to_uppercase()
                })).collect::<Vec<_>>()
            })]
        } else {
            vec![]
        };

        match self.get_tachiyomi_settings(&prefixed_id).await {
            Ok(discovered) => {
                let existing_keys: std::collections::HashSet<String> = settings
                    .iter()
                    .filter_map(|s| {
                        s.get("key")
                            .and_then(|k| k.as_str())
                            .map(String::from)
                    })
                    .collect();

                for pref in discovered {
                    if let Some(key) = pref.get("key").and_then(|k| k.as_str()) {
                        if !existing_keys.contains(key) {
                            settings.push(pref);
                        }
                    }
                }
            }
            Err(e) => {
                debug!(
                    ext = %prefixed_id,
                    error = ?e,
                    "__getTachiyomiSettings failed; skipping preference discovery"
                );
            }
        }

        let manifest_json = json!({
            "id": prefixed_id,
            "name": entry.sources.first().map(|s| s.name.clone()).unwrap_or_else(|| entry.name),
            "version": entry.version,
            "type": "manga",
            "language": entry.lang,
            "author": "tachiyomi",
            "main": "index.js",
            "source": "tachiyomi",
            "nsfw": nsfw,
            "settings": settings,
            "icon": icon_url,
        });

        let manifest_yaml = serde_yaml::to_string(&manifest_json)
            .map_err(|e| CoreError::Parse(e.to_string()))?;

        fs::write(ext_dir.join("manifest.yaml"), &manifest_yaml)
            .await.map_err(CoreError::Io)?;

        let manifest: ExtensionManifest = serde_yaml::from_str(&manifest_yaml)
            .map_err(|e| {
                error!(error = ?e, "Generated Tachiyomi manifest is invalid");
                CoreError::Parse("error.extension.invalid_manifest".into())
            })?;

        let loaded_settings = load_settings(&ext_dir, &manifest.settings).await;

        let extension = Extension {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "tachiyomi".to_string()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path: js_path,
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings,
            settings: loaded_settings,
            source: manifest.source,
        };

        self.extensions.insert(manifest.id.clone(), extension.clone());
        info!(ext = %extension.id, "Tachiyomi extension installed and settings reflected successfully");

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

        let extension = Extension {
            id: manifest.id.clone(),
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
            source: None
        };

        self.extensions.insert(manifest.id.clone(), extension.clone());
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

    #[instrument(skip(self, args))]
    pub async fn call_extension_function(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> CoreResult<Value> {
        let extension = self.extensions.get(extension_id).ok_or_else(|| {
            error!(ext = %extension_id, func = %function_name, "Attempted to call function on unloaded extension");
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        if !extension.script_path.exists() {
            error!(ext = %extension_id, path = %extension.script_path.display(), "Extension script file missing from disk");
            return Err(CoreError::NotFound("error.extension.script_missing".into()));
        }

        let extension_code = fs::read_to_string(&extension.script_path).await.map_err(CoreError::Io)?;

        let compat = match extension.source.as_deref() {
            Some("lnreader") => Some(CompatLayer::Lnreader(LNREADER.to_string())),
            Some("tachiyomi") => Some(CompatLayer::Tachiyomi(TACHIYOMI.to_string())),
            _ => None,
        };

        sandbox::execute_in_quickjs(
            extension_code,
            function_name.to_string(),
            args,
            self.headless.clone(),
            extension.settings.clone(),
            extension_id.to_string(),
            Arc::clone(&self.extension_state),
            compat
        ).await
    }

    #[instrument(skip(self, args))]
    async fn call_typed_function<T: DeserializeOwned>(
        &self,
        extension_id: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> CoreResult<T> {
        let raw_value = self.call_extension_function(extension_id, function_name, args).await?;

        serde_json::from_value(raw_value).map_err(|e| {
            error!(ext = %extension_id, func = %function_name, error = ?e, "Failed to deserialize response");
            CoreError::Internal("error.content.invalid_extension_response".into())
        })
    }

    pub async fn get_image_request_headers(
        &self,
        ext_id: &str,
        image_url: &str,
    ) -> CoreResult<HashMap<String, String>> {
        let extension = self.extensions.get(ext_id).ok_or_else(|| {
            CoreError::NotFound("error.extension.not_found".into())
        })?;

        if extension.source.as_deref() != Some("tachiyomi") {
            return Err(CoreError::BadRequest("error.extension.not_tachiyomi".into()));
        }

        self.call_typed_function(
            ext_id,
            "getImageRequestHeaders",
            vec![json!(image_url)],
        ).await
    }

    pub async fn get_tachiyomi_settings(&self, ext_id: &str) -> CoreResult<Vec<Value>> {
        self.call_typed_function(ext_id, "__getTachiyomiSettings", vec![]).await
    }

    pub async fn get_settings(&self, ext_id: &str) -> CoreResult<ExtensionFeatures> {
        self.call_typed_function(ext_id, "getStreamingSettings", vec![]).await
    }

    pub async fn get_filters(&self, ext_id: &str) -> CoreResult<ExtensionFilters> {
        self.call_typed_function(ext_id, "getFilters", vec![]).await
    }

    pub async fn search(&self, ext_id: &str, query: &str, filters: Value, page: u32) -> CoreResult<Vec<ExtensionSearchResult>> {

        self.call_typed_function(ext_id, "search", vec![json!(query), filters, json!(page)]).await
    }

    pub async fn get_metadata(&self, ext_id: &str, content_id: &str) -> CoreResult<ExtensionMetadata> {
        self.call_typed_function(ext_id, "getMetadata", vec![json!(content_id)]).await
    }

    pub async fn find_episodes(&self, ext_id: &str, content_id: &str) -> CoreResult<Vec<Episode>> {
        self.call_typed_function(ext_id, "findEpisodes", vec![json!(content_id)]).await
    }

    pub async fn find_chapters(&self, ext_id: &str, content_id: &str) -> CoreResult<Vec<Chapter>> {
        self.call_typed_function(ext_id, "findChapters", vec![json!(content_id)]).await
    }

    pub async fn find_episode_server(
        &self,
        ext_id: &str,
        content_id: &str,
        server: &str,
        category: &str
    ) -> CoreResult<EpisodeSource> {
        self.call_typed_function(
            ext_id,
            "findEpisodeServer",
            vec![json!(content_id), json!(server), json!(category)]
        ).await
    }

    pub async fn find_manga_pages(&self, ext_id: &str, chapter_id: &str) -> CoreResult<Vec<Page>> {
        self.call_typed_function(ext_id, "findChapterPages", vec![json!(chapter_id)]).await
    }

    pub async fn find_novel_html(&self, ext_id: &str, chapter_id: &str) -> CoreResult<String> {
        self.call_typed_function(ext_id, "findChapterPages", vec![json!(chapter_id)]).await
    }
}

async fn load_settings(
    ext_dir: &PathBuf,
    definitions: &[SettingDefinition],
) -> HashMap<String, Value> {
    let mut settings: HashMap<String, Value> = definitions
        .iter()
        .map(|d| (d.key.clone(), d.default.clone()))
        .collect();

    let settings_path = ext_dir.join("settings.json");
    if settings_path.exists() {
        if let Ok(raw) = fs::read_to_string(&settings_path).await {
            if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&raw) {
                for def in definitions {
                    if let Some(user_value) = map.get(&def.key) {
                        settings.insert(def.key.clone(), user_value.clone());
                    }
                }
            }
        }
    }

    settings
}

async fn persist_settings(ext_dir: &PathBuf, settings: &HashMap<String, Value>) {
    let path = ext_dir.join("settings.json");
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json).await {
                warn!("Could not write settings.json to {:?}: {}", path, e);
            }
        }
        Err(e) => warn!("Could not serialise settings for {:?}: {}", path, e),
    }
}