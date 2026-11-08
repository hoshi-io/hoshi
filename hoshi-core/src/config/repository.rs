use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;

use tracing::{error, warn};

use crate::config::model::UserConfig;
use crate::error::{CoreError, CoreResult};

pub struct ConfigRepository;

impl ConfigRepository {
    pub async fn get_config(pool: &SqlitePool, user_id: i32) -> CoreResult<UserConfig> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT config FROM UserConfig WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        Ok(match row {
            Some((raw,)) => parse_config_lenient(&raw, user_id),
            None => UserConfig::default(),
        })
    }

    pub async fn set_config(pool: &SqlitePool, user_id: i32, config: &UserConfig) -> CoreResult<()> {
        let raw = serde_json::to_string(config)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO UserConfig (user_id, config, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
                config     = excluded.config,
                updated_at = excluded.updated_at",
        )
            .bind(user_id)
            .bind(&raw)
            .bind(&now)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Read-modify-write, serialized by a process-wide lock so two quick patches
    /// (e.g. toggling two settings) can't overwrite each other.
    pub async fn patch_config(pool: &SqlitePool, user_id: i32, patch: &Value) -> CoreResult<UserConfig> {
        static PATCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
        let _guard = PATCH_LOCK.lock().await;

        let current = Self::get_config(pool, user_id).await?;
        let mut current_value = serde_json::to_value(&current)?;
        merge_json(&mut current_value, patch);
        let merged: UserConfig = serde_json::from_value(current_value).map_err(|e| {
            warn!(error = %e, user_id, "Rejected config patch: does not match the config schema");
            CoreError::BadRequest("error.config.invalid_patch".into())
        })?;
        Self::set_config(pool, user_id, &merged).await?;
        Ok(merged)
    }
}

/// Parses a stored config without ever discarding the whole thing.
///
/// A plain `from_str(..).unwrap_or_default()` resets *every* setting to default
/// if a single field fails to parse (e.g. a type change between versions), and
/// the next save then writes those defaults back for good. Instead, fall back to
/// applying the stored values one leaf at a time on top of the defaults, keeping
/// each value that still deserializes and dropping only the broken ones.
fn parse_config_lenient(raw: &str, user_id: i32) -> UserConfig {
    let stored: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, user_id, "Stored config is not valid JSON; using defaults");
            return UserConfig::default();
        }
    };
    if let Ok(cfg) = serde_json::from_value::<UserConfig>(stored.clone()) {
        return cfg;
    }

    let mut acc = match serde_json::to_value(UserConfig::default()) {
        Ok(v) => v,
        Err(_) => return UserConfig::default(),
    };
    let mut dropped = Vec::new();
    apply_leaves(&mut acc, &stored, &mut Vec::new(), &mut dropped);
    warn!(user_id, dropped = ?dropped, "Stored config had invalid fields; those were reset to defaults");

    serde_json::from_value(acc).unwrap_or_default()
}

/// Walks `stored` and copies each leaf into `acc`, keeping it only if the whole
/// config still deserializes afterwards.
fn apply_leaves(acc: &mut Value, stored: &Value, path: &mut Vec<String>, dropped: &mut Vec<String>) {
    match stored {
        Value::Object(map) if !map.is_empty() => {
            for (k, v) in map {
                path.push(k.clone());
                apply_leaves(acc, v, path, dropped);
                path.pop();
            }
        }
        leaf => {
            let Some(slot) = pointer_mut_create(acc, path) else { return };
            let previous = std::mem::replace(slot, leaf.clone());
            if serde_json::from_value::<UserConfig>(acc.clone()).is_err() {
                // Re-borrow: the slot is still there, restore what was in it.
                if let Some(slot) = pointer_mut_create(acc, path) {
                    *slot = previous;
                }
                dropped.push(path.join("."));
            }
        }
    }
}

fn pointer_mut_create<'a>(root: &'a mut Value, path: &[String]) -> Option<&'a mut Value> {
    let mut cur = root;
    for key in path {
        if !cur.is_object() {
            return None;
        }
        cur = cur.as_object_mut()?.entry(key.clone()).or_insert(Value::Null);
    }
    Some(cur)
}

fn merge_json(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, patch_val) in patch_map {
                let base_val = base_map.entry(key).or_insert(Value::Null);
                merge_json(base_val, patch_val);
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}