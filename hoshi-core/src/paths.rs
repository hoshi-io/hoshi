use std::path::PathBuf;
use tracing::debug;
use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub base_dir: PathBuf,
    pub database_path: PathBuf,
    pub extensions_path: PathBuf,
    pub logs_path: PathBuf,
    pub mpv_path: PathBuf,
}

impl AppPaths {
    pub fn from_base(base: PathBuf) -> Self {
        Self {
            database_path: base.join("app.db"),
            extensions_path: base.join("extensions"),
            logs_path: base.join("logs"),
            mpv_path: base.join("mpv"),
            base_dir: base,
        }
    }

    pub fn ensure_dirs(&self) -> CoreResult<()> {
        ensure_dir(&self.base_dir)?;
        ensure_dir(&self.extensions_path)?;
        ensure_dir(&self.logs_path)?;
        ensure_dir(&self.mpv_path)?;
        Ok(())
    }
}

fn ensure_dir(path: &PathBuf) -> CoreResult<()> {
    if !path.exists() {
        debug!(path = %path.display(), "Creating required application directory");
        std::fs::create_dir_all(path).map_err(CoreError::Io)?;
    }
    Ok(())
}