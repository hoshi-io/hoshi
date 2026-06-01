use std::sync::Arc;
use tracing::{error, info, warn};
use crate::error::CoreResult;
use crate::list::repository::ListRepository;
use crate::list::types::{ListEntry, UpsertEntryBody};
use crate::list::service::ListService;
use crate::state::AppState;
use crate::tracker::provider::UserListEntry;
use crate::tracker::repository::TrackerRepository;
use crate::users::repository::UserRepo;
use crate::content::services::enrichment::EnrichmentService;

pub struct StartupSyncService;

impl StartupSyncService {
    pub fn run(state: Arc<AppState>) {
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let users = match UserRepo::get_all_users(&state.pool).await {
                Ok(u) => u,
                Err(e) => { error!(error = ?e, "Startup sync: failed to fetch users"); return; }
            };

            info!(users = users.len(), "Starting tracker startup sync");

            for user in users {
                let integrations = match TrackerRepository::get_user_integrations(&state.pool, user.id).await {
                    Ok(i) => i,
                    Err(e) => { warn!(error = ?e, user_id = user.id, "Failed to fetch integrations"); continue; }
                };

                for integration in integrations {
                    if !integration.sync_enabled { continue; }

                    let state = state.clone();
                    tokio::spawn(async move {
                        let provider = match state.tracker_registry.get(&integration.tracker_name) {
                            Some(p) => p,
                            None => { warn!(tracker = %integration.tracker_name, "Not in registry, skipping"); return; }
                        };

                        let entries = match provider.get_user_list(
                            &integration.access_token,
                            &integration.tracker_user_id,
                        ).await {
                            Ok(e) => e,
                            Err(e) => { warn!(error = ?e, tracker = %integration.tracker_name, user_id = integration.user_id, "Failed to fetch remote list"); return; }
                        };

                        let mut imported = 0usize;
                        let mut skipped  = 0usize;

                        for entry in entries {
                            match Self::merge_entry(&state, integration.user_id, &integration.tracker_name, &entry).await {
                                Ok(true)  => imported += 1,
                                Ok(false) => skipped  += 1,
                                Err(e)    => warn!(error = ?e, tracker_id = %entry.tracker_media_id, "Failed to merge entry"),
                            }
                        }

                        info!(
                            tracker = %integration.tracker_name,
                            user_id = integration.user_id,
                            imported, skipped,
                            "Tracker sync complete"
                        );
                    });
                }
            }
        });
    }

    async fn merge_entry(
        state: &Arc<AppState>,
        user_id: i32,
        tracker_name: &str,
        entry: &UserListEntry,
    ) -> CoreResult<bool> {
        let cid = match TrackerRepository::find_cid_by_tracker(
            &state.pool, tracker_name, &entry.tracker_media_id,
        ).await? {
            Some(cid) => cid,
            None => {
                let media = match &entry.media {
                    Some(m) => m,
                    None => return Ok(false),
                };
                let full = EnrichmentService::create_enriched_content(
                    state,
                    &entry.content_type,
                    media,
                    &entry.tracker_media_id,
                    tracker_name,
                    None,
                ).await?;
                full.content.cid
            }
        };

        let local = ListRepository::get_entry(&state.pool, user_id, &cid).await?;
        if let Some(ref local_entry) = local {
            if local_entry.progress >= entry.progress {
                return Ok(false);
            }
        }

        let body = UpsertEntryBody {
            cid:          cid.clone(),
            status:       entry.status.clone().unwrap_or_else(|| "PLANNING".into()),
            progress:     Some(entry.progress),
            score:        entry.score,
            start_date:   entry.start_date.clone(),
            end_date:     entry.end_date.clone(),
            repeat_count: Some(entry.repeat_count),
            notes:        entry.notes.clone(),
            is_private:   Some(entry.is_private),
        };

        let pool = &state.pool;

        ListRepository::upsert_entry(
            pool, user_id, &body,
            &body.status,
            entry.progress,
            entry.start_date.clone(),
            entry.end_date.clone(),
        ).await?;

        if let Ok(Some(saved)) = ListRepository::get_entry(pool, user_id, &cid).await {
            if let Some(entry_id) = saved.id {
                let changes = diff_entry(local.as_ref(), &saved);
                if !changes.is_empty() {
                    if let Err(e) = ListRepository::insert_changes(
                        pool, entry_id, user_id,
                        "REMOTE_SYNC", Some(tracker_name), &changes,
                    ).await {
                        warn!(error = ?e, "Failed to write sync changelog");
                    }
                }

                let snapshot = serde_json::json!({
                    "status": entry.status,
                    "progress": entry.progress,
                    "score": entry.score,
                    "startDate": entry.start_date,
                    "endDate": entry.end_date,
                    "repeatCount": entry.repeat_count,
                });

                if let Err(e) = ListRepository::upsert_entry_source(
                    pool, entry_id, user_id, tracker_name, &entry.tracker_media_id, &snapshot,
                ).await {
                    warn!(error = ?e, "Failed to write entry source on sync");
                }
            }
        }

        Ok(true)
    }
}

fn diff_entry(
    prev: Option<&ListEntry>,
    next: &ListEntry,
) -> Vec<(&'static str, Option<String>, String)> {
    let mut changes = vec![];

    macro_rules! diff_field {
        ($field:expr, $old:expr, $new:expr) => {
            let old_s: Option<String> = $old;
            let new_s: String = $new;
            if prev.is_none() || old_s.as_deref() != Some(new_s.as_str()) {
                changes.push(($field, old_s, new_s));
            }
        };
    }

    diff_field!("status",
        prev.map(|e| e.status.clone()),
        next.status.clone());

    diff_field!("progress",
        prev.map(|e| e.progress.to_string()),
        next.progress.to_string());

    diff_field!("score",
        prev.and_then(|e| e.score).map(|s| s.to_string()),
        next.score.map(|s| s.to_string()).unwrap_or_default());

    diff_field!("repeat_count",
        prev.map(|e| e.repeat_count.to_string()),
        next.repeat_count.to_string());

    diff_field!("start_date",
        prev.and_then(|e| e.start_date.clone()),
        next.start_date.clone().unwrap_or_default());

    diff_field!("end_date",
        prev.and_then(|e| e.end_date.clone()),
        next.end_date.clone().unwrap_or_default());

    diff_field!("notes",
        prev.and_then(|e| e.notes.clone()),
        next.notes.clone().unwrap_or_default());

    diff_field!("is_private",
        prev.map(|e| (e.is_private as i32).to_string()),
        (next.is_private as i32).to_string());

    changes
}