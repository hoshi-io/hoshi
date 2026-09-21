use std::sync::Arc;
use tracing::warn;
use crate::config::model::MergeStrategy;
use crate::config::repository::ConfigRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::diff_field;
use crate::error::CoreResult;
use crate::list::repository::ListRepository;
use crate::list::types::{ListEntry, UpsertEntryBody};
use crate::state::AppState;
use crate::tracker::provider::UserListEntry;
use crate::tracker::repository::TrackerRepository;

pub struct MergeService;

type MergedFields = (i32, String, Option<f64>, Option<String>, Option<String>, Option<String>, i32);

impl MergeService {
    pub fn diff_entry(
        prev: Option<&ListEntry>,
        next: &ListEntry,
    ) -> Vec<(&'static str, Option<String>, String)> {
        let mut changes = vec![];
        let prev_is_none = prev.is_none();

        diff_field!(changes, prev_is_none, "status",
            prev.map(|e| e.status.clone()), next.status.clone());

        diff_field!(changes, prev_is_none, "progress",
            prev.map(|e| e.progress.to_string()), next.progress.to_string());

        diff_field!(changes, prev_is_none, "score",
            prev.and_then(|e| e.score).map(|s| s.to_string()),
            next.score.map(|s| s.to_string()).unwrap_or_default());

        diff_field!(changes, prev_is_none, "repeat_count",
            prev.map(|e| e.repeat_count.to_string()), next.repeat_count.to_string());

        diff_field!(changes, prev_is_none, "start_date",
            prev.and_then(|e| e.start_date.clone()), next.start_date.clone().unwrap_or_default());

        diff_field!(changes, prev_is_none, "end_date",
            prev.and_then(|e| e.end_date.clone()), next.end_date.clone().unwrap_or_default());

        diff_field!(changes, prev_is_none, "notes",
            prev.and_then(|e| e.notes.clone()), next.notes.clone().unwrap_or_default());

        diff_field!(changes, prev_is_none, "is_private",
            prev.map(|e| (e.is_private as i32).to_string()), (next.is_private as i32).to_string());

        changes
    }

    pub fn status_priority(status: &str) -> u8 {
        match status.to_uppercase().as_str() {
            "REPEATING"  => 6,
            "COMPLETED"  => 5,
            "CURRENT"    => 4,
            "PAUSED"     => 3,
            "DROPPED"    => 2,
            "PLANNING"   => 1,
            _            => 0,
        }
    }

    pub fn normalize_tracker_status(status: &str) -> String {
        match status.to_lowercase().as_str() {
            "watching"       | "current"       => "CURRENT".to_string(),
            "plan_to_watch"  | "planning"      => "PLANNING".to_string(),
            "completed"                        => "COMPLETED".to_string(),
            "on_hold"        | "paused"        => "PAUSED".to_string(),
            "dropped"                          => "DROPPED".to_string(),
            "repeating"      | "rewatching"    => "REPEATING".to_string(),
            _                                  => "PLANNING".to_string(), // Safe fallback
        }
    }
    
    fn resolve_merge_strategy(
        strategy: &MergeStrategy,
        tracker_name: &str,
        local: Option<&ListEntry>,
        entry: &UserListEntry,
    ) -> MergedFields {
        let Some(l) = local else {
            return (
                entry.progress,
                Self::normalize_tracker_status(&entry.status.clone().unwrap_or_else(|| "PLANNING".into())),
                entry.score,
                entry.start_date.clone(),
                entry.end_date.clone(),
                entry.notes.clone(),
                entry.repeat_count,
            );
        };

        let remote_status = Self::normalize_tracker_status(&entry.status.clone().unwrap_or_else(|| "PLANNING".into()));
        let remote_wins = |name: &str| tracker_name == name
            || (name == "myanimelist" && tracker_name == "mal");

        match strategy {
            MergeStrategy::KeepLocal => (
                l.progress, l.status.clone(), l.score,
                l.start_date.clone(), l.end_date.clone(),
                l.notes.clone(), l.repeat_count,
            ),
            MergeStrategy::KeepRemote | MergeStrategy::KeepLatest => (
                entry.progress, remote_status, entry.score,
                entry.start_date.clone(), entry.end_date.clone(),
                entry.notes.clone(), entry.repeat_count,
            ),
            MergeStrategy::KeepHighest => (
                entry.progress.max(l.progress),
                if Self::status_priority(&remote_status) >= Self::status_priority(&l.status) { remote_status } else { l.status.clone() },
                entry.score.or(l.score),
                l.start_date.clone().or(entry.start_date.clone()),
                l.end_date.clone().or(entry.end_date.clone()),
                entry.notes.clone().or_else(|| l.notes.clone()),
                entry.repeat_count.max(l.repeat_count),
            ),
            MergeStrategy::AnilistFirst if remote_wins("anilist") => (
                entry.progress, remote_status, entry.score,
                entry.start_date.clone(), entry.end_date.clone(),
                entry.notes.clone(), entry.repeat_count,
            ),
            MergeStrategy::MalFirst if remote_wins("myanimelist") => (
                entry.progress, remote_status, entry.score,
                entry.start_date.clone(), entry.end_date.clone(),
                entry.notes.clone(), entry.repeat_count,
            ),
            MergeStrategy::KitsuFirst if remote_wins("kitsu") => (
                entry.progress, remote_status, entry.score,
                entry.start_date.clone(), entry.end_date.clone(),
                entry.notes.clone(), entry.repeat_count,
            ),
            MergeStrategy::SimklFirst if remote_wins("simkl") => (
                entry.progress, remote_status, entry.score,
                entry.start_date.clone(), entry.end_date.clone(),
                entry.notes.clone(), entry.repeat_count,
            ),
            // Any *First strategy where this tracker isn't the preferred one
            MergeStrategy::AnilistFirst | MergeStrategy::MalFirst
            | MergeStrategy::KitsuFirst | MergeStrategy::SimklFirst => (
                l.progress, l.status.clone(), l.score,
                l.start_date.clone(), l.end_date.clone(),
                l.notes.clone(), l.repeat_count,
            ),
        }
    }

    pub async fn merge_entry(
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
                let owned_media;
                let media = match &entry.media {
                    Some(m) => m,
                    None => {
                        let provider = state.tracker_registry.get(tracker_name).ok_or_else(|| {
                            crate::error::CoreError::Internal("error.tracker.not_in_registry".into())
                        })?;
                        match provider.get_by_id(&entry.tracker_media_id).await {
                            Ok(Some(m)) => { owned_media = m; &owned_media }
                            Ok(None) => {
                                warn!(tracker = %tracker_name, id = %entry.tracker_media_id, "No inline media and get_by_id returned nothing, skipping entry");
                                return Ok(false);
                            }
                            Err(e) => {
                                warn!(error = ?e, tracker = %tracker_name, id = %entry.tracker_media_id, "No inline media and get_by_id failed, skipping entry");
                                return Ok(false);
                            }
                        }
                    }
                };
                let full = EnrichmentService::create_enriched_content(
                    state, &entry.content_type, media,
                    &entry.tracker_media_id, tracker_name, None,
                ).await?;
                full.content.cid
            }
        };

        let config = ConfigRepository::get_config(&state.pool, user_id).await?;
        let local = ListRepository::get_entry(&state.pool, user_id, &cid).await?;

        let (final_progress, final_status, final_score, final_start, final_end, final_notes, final_repeat_count) =
            Self::resolve_merge_strategy(&config.list.merge_strategy, tracker_name, local.as_ref(), entry);

        let needs_update = match &local {
            None => true,
            Some(l) => {
                l.progress != final_progress
                    || l.status != final_status
                    || l.score != final_score
                    || l.start_date != final_start
                    || l.end_date != final_end
                    || l.notes != final_notes
                    || l.repeat_count != final_repeat_count
            }
        };

        let pool = &state.pool;

        if needs_update {
            let body = UpsertEntryBody {
                cid:          cid.clone(),
                status:       final_status.clone(),
                progress:     Some(final_progress),
                score:        final_score,
                start_date:   final_start.clone(),
                end_date:     final_end.clone(),
                repeat_count: Some(final_repeat_count),
                notes:        final_notes.clone(),
                is_private:   Some(config.list.private_by_default || entry.is_private),
            };

            ListRepository::upsert_entry(
                pool, user_id, &body,
                &final_status, final_progress, final_start, final_end,
            ).await?;
        }

        Self::record_sync_side_effects(pool, user_id, &cid, tracker_name, entry, local.as_ref(), needs_update).await;

        Ok(needs_update)
    }

    async fn record_sync_side_effects(
        pool: &sqlx::SqlitePool,
        user_id: i32,
        cid: &str,
        tracker_name: &str,
        entry: &UserListEntry,
        local: Option<&ListEntry>,
        needs_update: bool,
    ) {
        match ListRepository::get_entry(pool, user_id, cid).await {
            Ok(Some(saved)) => {
                if let Some(entry_id) = saved.id {
                    if needs_update {
                        let changes = Self::diff_entry(local, &saved);
                        if !changes.is_empty() {
                            if let Err(e) = ListRepository::insert_changes(
                                pool, entry_id, user_id,
                                "REMOTE_SYNC", Some(tracker_name), &changes,
                            ).await {
                                warn!(error = ?e, "Failed to write sync changelog");
                            }
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
            Ok(None) => {
                warn!(cid = %cid, user_id, "Entry vanished immediately after upsert -- changelog and entry-source snapshot were skipped");
            }
            Err(e) => {
                warn!(error = ?e, cid = %cid, user_id, "Failed to re-fetch entry after upsert -- changelog and entry-source snapshot were skipped");
            }
        }
    }
}