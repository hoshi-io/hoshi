use crate::error::CoreResult;
use crate::core_err;
use crate::paths::AppPaths;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::migrate::Migrator;
use std::str::FromStr;
use tracing::{info, instrument};

pub struct DatabaseManager {
    pool: SqlitePool,
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

impl DatabaseManager {

    #[instrument(skip(paths))]
    pub async fn new(paths: &AppPaths) -> CoreResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(connect_options(paths)?)
            .await?;

        info!(
        path = %paths.database_path.display(),
        "Running database schema migrations"
    );

        MIGRATOR
            .run(&pool)
            .await
            .map_err(|e| core_err!(Internal, "error.system.migration_run_failed", e))?;

        info!("All database schemas applied successfully");

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
        .foreign_keys(true)
        .pragma("synchronous", "NORMAL")
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "30000000000"))
}

#[macro_export]
macro_rules! impl_from_row {
    ($struct:ty { $($field:ident),* $(,)? }) => {
        impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for $struct {
            fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(Self {
                    $($field: row.try_get(stringify!($field))?,)*
                })
            }
        }
    };
}