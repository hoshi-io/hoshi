use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::CoreResult;
use crate::tracker::types::{IntegrationCredentials, TrackerIntegration, TrackerMapping};

pub struct TrackerRepository;

impl TrackerRepository {
    pub async fn save_integration(
        pool: &SqlitePool,
        user_id: i32,
        tracker_name: &str,
        tracker_user_id: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        token_type: &str,
        expires_at: i64,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        profile_url: Option<&str>,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
        INSERT INTO UserIntegration
            (user_id, tracker_name, tracker_user_id, access_token, refresh_token,
             token_type, expires_at, display_name, avatar_url, profile_url)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(user_id, tracker_name) DO UPDATE SET
            tracker_user_id = excluded.tracker_user_id,
            access_token    = excluded.access_token,
            refresh_token   = excluded.refresh_token,
            token_type      = excluded.token_type,
            expires_at      = excluded.expires_at,
            display_name    = excluded.display_name,
            avatar_url      = excluded.avatar_url,
            profile_url     = excluded.profile_url,
            updated_at      = strftime('%s', 'now')
        "#,
        )
            .bind(user_id)
            .bind(tracker_name)
            .bind(tracker_user_id)
            .bind(access_token)
            .bind(refresh_token)
            .bind(token_type)
            .bind(expires_at)
            .bind(display_name)
            .bind(avatar_url)
            .bind(profile_url)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_credentials(
        pool: &SqlitePool,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<Option<IntegrationCredentials>> {
        let row: Option<(String, Option<String>, String)> = sqlx::query_as(
            "SELECT access_token, refresh_token, tracker_user_id
             FROM UserIntegration
             WHERE user_id = ? AND tracker_name = ?",
        )
            .bind(user_id)
            .bind(tracker_name)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(access_token, refresh_token, tracker_user_id)| {
            IntegrationCredentials { access_token, refresh_token, tracker_user_id }
        }))
    }

    pub async fn delete_integration(
        pool: &SqlitePool,
        user_id: i32,
        tracker_name: &str,
    ) -> CoreResult<bool> {
        let result = sqlx::query(
            "DELETE FROM UserIntegration WHERE user_id = ? AND tracker_name = ?",
        )
            .bind(user_id)
            .bind(tracker_name)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_integrations(
        pool: &SqlitePool,
        user_id: i32,
    ) -> CoreResult<Vec<TrackerIntegration>> {
        let rows: Vec<(
            i32,
            String,
            String,
            String,
            Option<String>,
            String,
            i64,
            i32,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i32>,
            Option<i64>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT user_id, tracker_name, tracker_user_id, access_token, refresh_token,
                token_type, expires_at, sync_enabled,
                display_name, avatar_url, profile_url, total_entries,
                CAST(last_synced_at AS INTEGER) as last_synced_at,
                CAST(created_at AS TEXT) as created_at,
                CAST(updated_at AS TEXT) as updated_at
         FROM UserIntegration WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|(
                      user_id,
                      tracker_name,
                      tracker_user_id,
                      access_token,
                      refresh_token,
                      token_type,
                      expires_at,
                      sync_enabled,
                      display_name,
                      avatar_url,
                      profile_url,
                      total_entries,
                      last_synced_at,
                      created_at,
                      updated_at,
                  )| {
                TrackerIntegration {
                    user_id,
                    tracker_name,
                    tracker_user_id,
                    access_token,
                    refresh_token,
                    token_type,
                    expires_at,
                    sync_enabled: sync_enabled == 1,
                    display_name,
                    avatar_url,
                    profile_url,
                    total_entries,
                    last_synced_at,
                    created_at: created_at.and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
                    updated_at: updated_at.and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
                }
            })
            .collect())
    }

    pub async fn set_sync_enabled(
        pool: &SqlitePool,
        user_id: i32,
        tracker_name: &str,
        enabled: bool,
    ) -> CoreResult<()> {
        sqlx::query(
            "UPDATE UserIntegration
             SET sync_enabled = ?, updated_at = strftime('%s', 'now')
             WHERE user_id = ? AND tracker_name = ?",
        )
            .bind(enabled)
            .bind(user_id)
            .bind(tracker_name)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn add_mapping(pool: &SqlitePool, mapping: &TrackerMapping) -> CoreResult<()> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO tracker_mappings
                (cid, tracker_name, tracker_id, tracker_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
            .bind(&mapping.cid)
            .bind(&mapping.tracker_name)
            .bind(&mapping.tracker_id)
            .bind(&mapping.tracker_url)
            .bind(mapping.created_at)
            .bind(now)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_mapping_id(
        pool: &SqlitePool,
        cid: &str,
        tracker_name: &str,
        new_id: &str,
    ) -> CoreResult<usize> {
        let now = Utc::now().timestamp();
        let result = sqlx::query(
            "UPDATE tracker_mappings
             SET tracker_id = ?, updated_at = ?
             WHERE cid = ? AND tracker_name = ?",
        )
            .bind(new_id)
            .bind(now)
            .bind(cid)
            .bind(tracker_name)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    pub async fn delete_mapping(
        pool: &SqlitePool,
        cid: &str,
        tracker_name: &str,
    ) -> CoreResult<usize> {
        let result = sqlx::query(
            "DELETE FROM tracker_mappings WHERE cid = ? AND tracker_name = ?",
        )
            .bind(cid)
            .bind(tracker_name)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    pub async fn get_mappings_by_cid(
        pool: &SqlitePool,
        cid: &str,
    ) -> CoreResult<Vec<TrackerMapping>> {
        let rows: Vec<(String, String, String, Option<String>, i64, i64)> =
            sqlx::query_as(
                "SELECT cid, tracker_name, tracker_id, tracker_url,
                         created_at, updated_at
                 FROM tracker_mappings WHERE cid = ?",
            )
                .bind(cid)
                .fetch_all(pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(cid, tracker_name, tracker_id, tracker_url,
                      created_at, updated_at)| {
                TrackerMapping {
                    cid,
                    tracker_name,
                    tracker_id,
                    tracker_url,
                    created_at,
                    updated_at,
                }
            })
            .collect())
    }

    pub async fn find_tracker_id_by_cid(
        pool: &SqlitePool,
        cid: &str,
        tracker_name: &str,
    ) -> CoreResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT tracker_id FROM tracker_mappings WHERE cid = ? AND tracker_name = ?",
        )
            .bind(cid)
            .bind(tracker_name)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(tracker_id,)| tracker_id))
    }

    pub async fn find_cid_by_tracker(
        pool: &SqlitePool,
        tracker_name: &str,
        tracker_id: &str,
    ) -> CoreResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT cid FROM tracker_mappings WHERE tracker_name = ? AND tracker_id = ?",
        )
            .bind(tracker_name)
            .bind(tracker_id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(|(cid,)| cid))
    }


    pub async fn update_sync_stats(
        pool: &SqlitePool,
        user_id: i32,
        tracker_name: &str,
        total_entries: i64,
    ) -> CoreResult<()> {
        sqlx::query(
            "UPDATE UserIntegration
         SET total_entries  = ?,
             last_synced_at = strftime('%s', 'now'),
             updated_at     = strftime('%s', 'now')
         WHERE user_id = ? AND tracker_name = ?",
        )
            .bind(total_entries)
            .bind(user_id)
            .bind(tracker_name)
            .execute(pool)
            .await?;

        Ok(())
    }
}