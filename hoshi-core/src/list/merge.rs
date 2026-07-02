use std::sync::Arc;
use tracing::warn;
use crate::config::model::MergeStrategy;
use crate::config::repository::ConfigRepository;
use crate::content::services::enrichment::EnrichmentService;
use crate::error::CoreResult;
use crate::list::repository::ListRepository;
use crate::list::types::{ListEntry, UpsertEntryBody};
use crate::state::AppState;
use crate::tracker::provider::UserListEntry;
use crate::tracker::repository::TrackerRepository;

pub struct MergeService;

impl MergeService {
    pub fn diff_entry(
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
                let media = match &entry.media {
                    Some(m) => m,
                    None => return Ok(false),
                };
                let full = EnrichmentService::create_enriched_content(
                    state, &entry.content_type, media,
                    &entry.tracker_media_id, tracker_name, None,
                ).await?;
                full.content.cid
            }
        };

        let config = ConfigRepository::get_config(&state.pool, user_id).await?;
        let strategy = &config.list.merge_strategy;

        let local = ListRepository::get_entry(&state.pool, user_id, &cid).await?;

        let (final_progress, final_status, final_score, final_start, final_end) = match &local {
            None => (
                entry.progress,
                Self::normalize_tracker_status(&entry.status.clone().unwrap_or_else(|| "PLANNING".into())),
                entry.score,
                entry.start_date.clone(),
                entry.end_date.clone(),
            ),
            Some(l) => {
                let remote_status = Self::normalize_tracker_status(&entry.status.clone().unwrap_or_else(|| "PLANNING".into()));
                match strategy {
                    MergeStrategy::KeepLocal => (
                        l.progress, l.status.clone(), l.score,
                        l.start_date.clone(), l.end_date.clone(),
                    ),
                    MergeStrategy::KeepRemote => (
                        entry.progress, remote_status, entry.score,
                        entry.start_date.clone(), entry.end_date.clone(),
                    ),
                    MergeStrategy::KeepHighest => (
                        entry.progress.max(l.progress),
                        if Self::status_priority(&remote_status) >= Self::status_priority(&l.status) { remote_status } else { l.status.clone() },
                        entry.score.or(l.score),
                        l.start_date.clone().or(entry.start_date.clone()),
                        l.end_date.clone().or(entry.end_date.clone()),
                    ),
                    MergeStrategy::KeepLatest => (
                        entry.progress, remote_status, entry.score,
                        entry.start_date.clone(), entry.end_date.clone(),
                    ),
                    MergeStrategy::AnilistFirst => {
                        if tracker_name == "anilist" {
                            (entry.progress, remote_status, entry.score,
                             entry.start_date.clone(), entry.end_date.clone())
                        } else {
                            (l.progress, l.status.clone(), l.score,
                             l.start_date.clone(), l.end_date.clone())
                        }
                    },
                    MergeStrategy::MalFirst => {
                        if tracker_name == "myanimelist" || tracker_name == "mal" {
                            (entry.progress, remote_status, entry.score,
                             entry.start_date.clone(), entry.end_date.clone())
                        } else {
                            (l.progress, l.status.clone(), l.score,
                             l.start_date.clone(), l.end_date.clone())
                        }
                    },
                    MergeStrategy::KitsuFirst => {
                        if tracker_name == "kitsu" {
                            (entry.progress, remote_status, entry.score,
                             entry.start_date.clone(), entry.end_date.clone())
                        } else {
                            (l.progress, l.status.clone(), l.score,
                             l.start_date.clone(), l.end_date.clone())
                        }
                    },

                    MergeStrategy::SimklFirst => {
                        if tracker_name == "simkl" {
                            (entry.progress, remote_status, entry.score,
                             entry.start_date.clone(), entry.end_date.clone())
                        } else {
                            (l.progress, l.status.clone(), l.score,
                             l.start_date.clone(), l.end_date.clone())
                        }
                    },
                }
            }
        };

        let needs_update = match &local {
            None => true,
            Some(l) => l.progress != final_progress || l.status != final_status,
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
                repeat_count: Some(entry.repeat_count),
                notes:        entry.notes.clone(),
                is_private:   Some(config.list.private_by_default || entry.is_private),
            };

            ListRepository::upsert_entry(
                pool, user_id, &body,
                &final_status, final_progress, final_start, final_end,
            ).await?;
        }

        if let Ok(Some(saved)) = ListRepository::get_entry(pool, user_id, &cid).await {
            if let Some(entry_id) = saved.id {
                if needs_update {
                    let changes = Self::diff_entry(local.as_ref(), &saved);
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

        Ok(needs_update)
    }
}