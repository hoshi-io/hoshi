// dev.rs
use std::path::PathBuf;
use tokio::fs;
use crate::error::{CoreError, CoreResult};
use super::{ExtensionManager, load_settings, persist_settings};
use super::types::{Extension, ExtensionManifest, ExtensionType};

impl ExtensionManager {
    pub async fn create_dev_extension(
        &mut self,
        name: &str,
        ext_type: ExtensionType,
        starter_code: &str,
    ) -> CoreResult<Extension> {
        let id = format!("{}-dev", slugify(name));
        let ext_dir = self.extensions_dir.join(&id);

        if ext_dir.exists() {
            return Err(CoreError::Internal("error.extension.already_exists".into()));
        }
        fs::create_dir_all(&ext_dir).await.map_err(CoreError::Io)?;

        fs::write(ext_dir.join("main.js"), starter_code).await.map_err(CoreError::Io)?;

        let manifest = ExtensionManifest {
            id: id.clone(),
            name: name.to_string(),
            version: "0.0.1".to_string(),
            author: Some("dev".to_string()),
            icon: None,
            ext_type: ext_type.clone(),
            main: "main.js".to_string(),
            language: "en".to_string(),
            nsfw: false,
            skip_default_processing: false,
            settings: vec![],
            source: None,
            dev: true,
        };

        let yaml = serde_yaml::to_string(&manifest)
            .map_err(|e| CoreError::Internal(format!("error.extension.manifest_serialize: {e}")))?;
        fs::write(ext_dir.join("manifest.yaml"), yaml).await.map_err(CoreError::Io)?;

        let extension = Extension {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "Unknown".into()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path: ext_dir.join(&manifest.main),
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings.clone(),
            settings: std::collections::HashMap::new(),
            source: manifest.source,
            dev: true,
        };

        self.extensions.insert(id, extension.clone());
        Ok(extension)
    }

    pub async fn read_extension_source(&self, ext_id: &str) -> CoreResult<String> {
        let ext = self.dev_ext(ext_id)?;
        fs::read_to_string(&ext.script_path).await.map_err(CoreError::Io)
    }

    pub async fn write_extension_source(&self, ext_id: &str, code: String) -> CoreResult<()> {
        let ext = self.dev_ext(ext_id)?;
        fs::write(&ext.script_path, code).await.map_err(CoreError::Io)
    }

    pub async fn read_manifest_raw(&self, ext_id: &str) -> CoreResult<String> {
        let ext = self.dev_ext(ext_id)?;
        let manifest_path = ext.script_path.parent().unwrap().join("manifest.yaml");
        fs::read_to_string(&manifest_path).await.map_err(CoreError::Io)
    }

    pub async fn write_manifest_raw(&mut self, ext_id: &str, yaml: String) -> CoreResult<Extension> {
        let ext_dir = {
            let ext = self.dev_ext(ext_id)?;
            ext.script_path.parent().unwrap().to_path_buf()
        };

        let mut manifest: ExtensionManifest = serde_yaml::from_str(&yaml)
            .map_err(|e| CoreError::Internal(format!("error.extension.invalid_manifest: {e}")))?;
        manifest.dev = true; // force, ignore whatever the user wrote

        if manifest.id != ext_id {
            return Err(CoreError::Internal("error.extension.id_mismatch".into()));
        }

        let script_path = ext_dir.join(&manifest.main);
        if !script_path.exists() {
            return Err(CoreError::NotFound("error.extension.script_missing".into()));
        }

        fs::write(ext_dir.join("manifest.yaml"), &yaml).await.map_err(CoreError::Io)?;

        let settings = load_settings(&ext_dir, &manifest.settings).await;
        let extension = Extension {
            id: manifest.id.clone(),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author.unwrap_or_else(|| "Unknown".into()),
            icon: manifest.icon,
            ext_type: manifest.ext_type,
            script_path,
            language: manifest.language,
            nsfw: manifest.nsfw,
            skip_default_processing: manifest.skip_default_processing,
            setting_definitions: manifest.settings.clone(),
            settings,
            source: manifest.source,
            dev: true,
        };

        self.extensions.insert(ext_id.to_string(), extension.clone());
        Ok(extension)
    }

    pub async fn delete_dev_extension(&mut self, ext_id: &str) -> CoreResult<()> {
        let ext_dir = {
            let ext = self.dev_ext(ext_id)?;
            ext.script_path.parent().unwrap().to_path_buf()
        };
        fs::remove_dir_all(&ext_dir).await.map_err(CoreError::Io)?;
        self.extensions.remove(ext_id);
        Ok(())
    }

    fn dev_ext(&self, ext_id: &str) -> CoreResult<&Extension> {
        self.extensions.get(ext_id)
            .filter(|e| e.dev)
            .ok_or_else(|| CoreError::NotFound("error.extension.not_found".into()))
    }
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}