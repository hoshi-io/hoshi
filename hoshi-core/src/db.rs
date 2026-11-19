use crate::error::CoreResult;
use crate::core_err;
use crate::paths::AppPaths;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::migrate::Migrator;
use sqlx::{Executor, Row, Sqlite, Transaction};
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

        repair_legacy_schema(&pool)
            .await
            .map_err(|e| core_err!(Internal, "error.system.migration_run_failed", e))?;
        repair_edited_checksums(&pool)
            .await
            .map_err(|e| core_err!(Internal, "error.system.migration_run_failed", e))?;

        // A database touched by a newer build has migrations this build doesn't know;
        // skip them instead of refusing to start (they're additive).
        let migrator = Migrator {
            migrations: MIGRATOR.migrations.clone(),
            ignore_missing: true,
            locking: MIGRATOR.locking,
        };
        migrator
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

/// v1.2.2–v1.2.4 shipped edited copies of migrations 4 and 6 (an index and the
/// TrackerSyncJob table were added in place; they now live in 0010 and 0008). Databases
/// created by those builds recorded the edited checksums, so point them back at the
/// released files, which sqlx would otherwise reject as "modified".
/// Each edit is listed with its LF and CRLF (Windows CI checkout) checksum.
const EDITED_CHECKSUMS: &[(i64, &str)] = &[
    (4, "c3325a4ba1270d9cd7456efd0ef5b8faabd228327204159604db7066821ef0d59198a886cf84b88a594dc29535f0ed80"),
    (4, "016d61a64fd1043a094394fc4fe34c8c14dee2336bd1a5ca46bf1ddf93166e85759cc14733d152a293d0ba2fdf3bce3f"),
    (6, "3d3fe3f3e74c4fb19b78070d624242405594657fb468db14d918a78e12d08547a2f5823fee5ac7af212f9a1f02e68294"),
    (6, "7e22557d87e7c0dc1770dcf660146b5a99c108afc65a5fda254b7c1efeea7e42fc9cc5f766e14c2e1f65090fa41d2d32"),
    // Dev builds after v1.2.4 ran 0008 before TrackerSyncJob was created at its top.
    (8, "d52b9a698db3a8e690ff2915d99856bab86a039826461b29cb74794a404071e98716602810e4c9b7279e2bcd3b6146f9"),
    (8, "8bbf484f579206d097b719e34e2f7d8144e43a66d533e32a9eff451cfce02b892ce4f9f1cf9225d337da0171ec105a48"),
];

/// Builds before `*.sql eol=lf` embedded CRLF migrations on Windows, so the same file has
/// two checksums depending on where the build ran. Accept either line-ending variant.
fn line_ending_variants(sql: &str) -> [Vec<u8>; 2] {
    use sha2::{Digest, Sha384};
    let lf = sql.replace("\r\n", "\n");
    let crlf = lf.replace('\n', "\r\n");
    [Sha384::digest(lf.as_bytes()).to_vec(), Sha384::digest(crlf.as_bytes()).to_vec()]
}

async fn repair_edited_checksums(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    if !table_exists(pool, "_sqlx_migrations").await? {
        return Ok(());
    }
    let applied = sqlx::query("SELECT version, checksum FROM _sqlx_migrations")
        .fetch_all(pool)
        .await?;
    for row in applied {
        let version: i64 = row.try_get("version")?;
        let stored: Vec<u8> = row.try_get("checksum")?;
        let Some(current) = MIGRATOR.iter().find(|m| m.version == version) else { continue };
        if stored == current.checksum.as_ref() {
            continue;
        }
        let known = line_ending_variants(&current.sql).contains(&stored)
            || EDITED_CHECKSUMS.iter().any(|(v, hex)| *v == version && hex_decode(hex) == stored);
        if !known {
            continue;
        }
        sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(current.checksum.as_ref())
            .bind(version)
            .execute(pool)
            .await?;
        info!(version, "Repaired checksum of a migration built with other line endings or edited in place");
    }
    Ok(())
}

/// Before sqlx migrations (<= v1.1.3) the schema files were re-run with `IF NOT EXISTS`
/// on every start, so changes to existing tables never reached older databases.
/// Brings those tables to the v1.2 baseline; a no-op on anything newer.
async fn repair_legacy_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // v1.0.x: profile / sync columns on integrations.
    if tx_table_exists(&mut tx, "UserIntegration").await? {
        let cols = columns(&mut tx, "UserIntegration").await?;
        for (name, ddl) in [
            ("display_name", "TEXT"),
            ("avatar_url", "TEXT"),
            ("profile_url", "TEXT"),
            ("total_entries", "INTEGER"),
            ("last_synced_at", "DATETIME"),
            ("score_format", "TEXT"),
            ("sync_interval", "INTEGER NOT NULL DEFAULT 3600"),
        ] {
            if !cols.iter().any(|c| c == name) {
                tx.execute(format!("ALTER TABLE UserIntegration ADD COLUMN {name} {ddl}").as_str()).await?;
                info!(column = name, "Added missing UserIntegration column");
            }
        }
    }

    // <= v1.1.0: unit sync timestamp.
    if tx_table_exists(&mut tx, "content_units").await?
        && !columns(&mut tx, "content_units").await?.iter().any(|c| c == "updated_at")
    {
        tx.execute("ALTER TABLE content_units ADD COLUMN updated_at INTEGER").await?;
        info!("Added missing content_units.updated_at");
    }

    // <= v1.1.0: relations keyed by target cid only. The old rows can't be converted
    // (no tracker ids / titles); relations are re-fetched with the content.
    let mut rebuild_relations = false;
    if tx_table_exists(&mut tx, "content_relations").await?
        && !columns(&mut tx, "content_relations").await?.iter().any(|c| c == "target_tracker_name")
    {
        tx.execute("DROP TABLE content_relations").await?;
        rebuild_relations = true;
        info!("Dropped legacy content_relations table");
    }

    // v1.1.0-v1.1.1: `entry_id NOT NULL ... ON DELETE SET NULL` made list entries undeletable.
    let mut rebuild_changes = false;
    if tx_table_exists(&mut tx, "ListEntryChange").await? {
        let set_null = sqlx::query("SELECT on_delete FROM pragma_foreign_key_list('ListEntryChange') WHERE \"from\" = 'entry_id'")
            .fetch_optional(&mut *tx)
            .await?
            .is_some_and(|r| r.get::<String, _>(0).eq_ignore_ascii_case("SET NULL"));
        if set_null {
            tx.execute("ALTER TABLE ListEntryChange RENAME TO ListEntryChange_legacy").await?;
            rebuild_changes = true;
        }
    }

    // Tables no current code uses.
    tx.execute("DROP TABLE IF EXISTS pending_relations").await?;
    tx.execute("DROP TABLE IF EXISTS ListBackup").await?;

    // The released migrations are all `IF NOT EXISTS`, so re-running them recreates exactly
    // the dropped tables. Already-applied databases won't run them again through sqlx.
    if rebuild_relations {
        tx.execute(include_str!("../migrations/0004_content_details.sql")).await?;
        tx.execute(include_str!("../migrations/0010_relation_target_tracker_index.sql")).await?;
    }
    if rebuild_changes {
        tx.execute(include_str!("../migrations/0006_user_library.sql")).await?;
        tx.execute(
            "INSERT INTO ListEntryChange (id, entry_id, user_id, changed_at, source, tracker, field, old_value, new_value)
             SELECT id, entry_id, user_id, changed_at, source, tracker, field, old_value, new_value
               FROM ListEntryChange_legacy
              WHERE entry_id IN (SELECT id FROM ListEntry)",
        )
        .await?;
        tx.execute("DROP TABLE ListEntryChange_legacy").await?;
        // Its indexes went with the legacy table; recreate them.
        tx.execute(include_str!("../migrations/0006_user_library.sql")).await?;
        info!("Rebuilt ListEntryChange with ON DELETE CASCADE");
    }

    tx.commit().await
}

async fn table_exists(pool: &SqlitePool, name: &str) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?
        .is_some())
}

async fn tx_table_exists(tx: &mut Transaction<'_, Sqlite>, name: &str) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?")
        .bind(name)
        .fetch_optional(&mut **tx)
        .await?
        .is_some())
}

async fn columns(tx: &mut Transaction<'_, Sqlite>, table: &str) -> Result<Vec<String>, sqlx::Error> {
    Ok(sqlx::query("SELECT name FROM pragma_table_info(?)")
        .bind(table)
        .fetch_all(&mut **tx)
        .await?
        .into_iter()
        .map(|r| r.get::<String, _>(0))
        .collect())
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
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
