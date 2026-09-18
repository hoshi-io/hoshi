use crate::error::{CoreError, CoreResult};
use crate::paths::AppPaths;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use tracing::{info, instrument};

pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    #[instrument(skip(paths))]
    pub async fn new(paths: &AppPaths) -> CoreResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(connect_options(paths)?)
            .await?;

        info!(path = %paths.database_path.display(), "Running database schema migrations");
        let migrator = Migrator::new(Path::new("../migrations"))
            .await
            .map_err(|e| CoreError::Internal(format!("migration setup failed: {e}")))?;
        migrator
            .run(&pool)
            .await
            .map_err(|e| CoreError::Internal(format!("migration run failed: {e}")))?;
        info!("All database schemas applied successfully");

        info!(path = %paths.database_path.display(), "Connected to SQLite database (sqlx pool)");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn connect_options(paths: &AppPaths) -> CoreResult<SqliteConnectOptions> {
    let url = format!("sqlite:{}", paths.database_path.display());

    Ok(SqliteConnectOptions::from_str(&url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true))
}