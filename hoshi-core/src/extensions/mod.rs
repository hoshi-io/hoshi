mod install;
mod calls;
mod sandbox;
pub mod types;
pub mod html_query;
pub mod dev;

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::fs;
use tracing::{error, info, instrument, warn};
use types::{Extension, ExtensionManifest, ExtensionType, SettingDefinition};

pub type ExtensionStateStore = Arc<Mutex<HashMap<String, HashMap<String, Value>>>>;

use crate::error::{CoreError, CoreResult};
use crate::headless::{noop_headless, HeadlessHandle};
use crate::paths::AppPaths;

const BASE: &str  = include_str!("base/Base.js");
const ANIME: &str = include_str!("base/Anime.js");
const MANGA: &str = include_str!("base/Manga.js");
const NOVEL: &str = include_str!("base/Novel.js");
// const TACHIYOMI: &str = include_str!("compatibility/tachiyomi.js"); abandoned support for apk based extensions
const LNREADER: &str = include_str!("compatibility/lnreader.js");
const SORA: &str = include_str!("compatibility/sora.js");

static LNREADER_ARC: OnceLock<Arc<str>> = OnceLock::new();
static SORA_ARC: OnceLock<Arc<str>> = OnceLock::new();
const SANDBOX_BOOTSTRAP: &str = include_str!("sandbox_bootstrap.js");

pub struct ExtensionManager {
    extensions: HashMap<String, Extension>,
    extensions_dir: PathBuf,
    headless: HeadlessHandle,
    extension_state: ExtensionStateStore,
    http_client: reqwest::Client,
}

impl ExtensionManager {
    pub fn new(paths: &AppPaths, http_client: reqwest::Client) -> CoreResult<Self> {
        let extensions_dir = paths.base_dir.join("extensions");
        Ok(Self {
            extensions: HashMap::new(),
            extensions_dir,
            headless: noop_headless(),
            extension_state: Arc::new(Mutex::new(HashMap::new())),
            http_client
        })
    }

    pub fn list_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().filter(|e| !e.dev).collect()
    }

    pub fn list_dev_extensions(&self) -> Vec<&Extension> {
        self.extensions.values().filter(|e| e.dev).collect()
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
                source: manifest.source,
                dev: manifest.dev,
            };

            self.extensions.insert(manifest.id, extension);
            loaded_count += 1;
        }

        info!(count = loaded_count, "Extensions loaded from disk successfully");
        Ok(())
    }
}

pub(super) async fn load_settings(
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

pub(super) async fn persist_settings(ext_dir: &PathBuf, settings: &HashMap<String, Value>) {
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