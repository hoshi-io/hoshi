use std::sync::Arc;
use serde_json::{json, Value};
use chrono::Utc;
use futures::future::join_all;
use sqlx::SqlitePool;
use tracing::{debug, error, info, instrument, warn};
use crate::content::models::EpisodeData;
use crate::content::repositories::content::ContentRepository;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::TrackerIntegration;
use crate::list::types::{ChangeSource, EnrichedListEntry, EntryHistoryResponse, FilterQuery, ListEntry, ListResponse, SingleEntryResponse, SuccessResponse, UpsertEntryBody, UpsertEntryResponse, UserStats};
use crate::tracker::provider::UpdateEntryParams;
use crate::state::AppState;

pub struct ListService;

impl ListService {

    #[instrument(skip(state))]
    pub async fn get_list(
        state: &AppState,
        user_id: i32,
        filter: FilterQuery,
    ) -> CoreResult<ListResponse> {
        let entries = ListRepository::get_entries(&state.pool, user_id, filter.status.as_deref()).await?;

        let mut enriched = Self::enrich_entries(&state, &state.pool, entries).await?;

        if let Some(ct) = filter.content_type {
            enriched.retain(|e| e.content_type == ct.to_lowercase());
        }

        Ok(ListResponse { results: enriched })
    }

    #[instrument(skip(state))]
    pub async fn get_single_entry(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<SingleEntryResponse> {
        let entry = ListRepository::get_entry(&state.pool, user_id, &cid).await?;

        if let Some(e) = entry {
            let enriched = Self::enrich_entries(&state, &state.pool, vec![e]).await?;
            Ok(SingleEntryResponse {
                found: true,
                entry: enriched.into_iter().next(),
            })
        } else {
            Ok(SingleEntryResponse { found: false, entry: None })
        }
    }

    #[instrument(skip(state))]
    pub async fn get_user_stats(
        state: &AppState,
        user_id: i32,
    ) -> CoreResult<UserStats> {
        let mut stats = ListRepository::get_user_stats(&state.pool, user_id).await?;

        let completed_progress = ListRepository::get_completed_entries_progress(&state.pool, user_id).await?;
        let mut total_episodes = 0i32;
        let mut total_chapters = 0i32;

        for (cid, progress) in completed_progress {
            if let Ok(Some(content)) = ContentRepository::get_content_by_cid(&state.pool, &cid).await {
                match content.content_type.as_str() {
                    "anime" => total_episodes += progress,
                    "manga" | "novel" => total_chapters += progress,
                    _ => {}
                }
            }
        }

        stats.total_episodes = total_episodes;
        stats.total_chapters = total_chapters;

        Ok(stats)
    }

    #[instrument(skip(state, body), fields(cid = %body.cid, status = %body.status))]
    pub async fn upsert_entry(
        state: Arc<AppState>,
        user_id: i32,
        body: UpsertEntryBody,
    ) -> CoreResult<UpsertEntryResponse> {
        let total_units = {
            ContentRepository::get_content_by_cid(&state.pool, &body.cid).await?
                .ok_or_else(|| {
                    warn!("Attempted to upsert entry for non-existent content");
                    CoreError::NotFound("error.list.content_not_found".into())
                })?;

            ContentRepository::get_by_cid(&state.pool, &body.cid).await?
                .map(|m| match m.eps_or_chapters {
                    EpisodeData::Count(n) => n,
                    EpisodeData::List(ref l)  => l.len() as i32,
                })
        };

        let prev_entry = ListRepository::get_entry(&state.pool, user_id, &body.cid).await?;

        let is_new        = prev_entry.is_none();
        let prev_progress = prev_entry.as_ref().map(|e| e.progress).unwrap_or(0);
        let new_progress  = body.progress.unwrap_or(prev_progress);

        if !is_new && body.progress.is_some() && new_progress < prev_progress {
            warn!(prev = prev_progress, new = new_progress, "Ignoring progress downgrade for user");
            return Ok(UpsertEntryResponse { success: true, changes: 0, is_new: false });
        }

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut final_start_date = body.start_date.clone();
        let mut final_end_date   = body.end_date.clone();
        let mut final_status     = body.status.clone();

        if let Some(ref prev) = prev_entry {
            if final_start_date.is_none() && prev.start_date.is_some() {
                final_start_date = prev.start_date.clone();
            }
        }

        if final_start_date.is_none() && new_progress == 1 {
            final_start_date = Some(today.clone());
        }

        if let Some(total) = total_units {
            if new_progress >= total && total > 0 {
                final_status = "COMPLETED".to_string();
                if final_end_date.is_none() {
                    final_end_date = Some(today);
                }
            }
        }

        let changes = ListRepository::upsert_entry(
            &state.pool,
            user_id,
            &body,
            &final_status,
            new_progress,
            final_start_date.clone(),
            final_end_date.clone(),
        ).await?;

        let changes_rows = Self::diff_entry(
            prev_entry.as_ref(),
            &final_status,
            new_progress,
            &body,
            &final_start_date,
            &final_end_date,
        );

        if !changes_rows.is_empty() {
            if let Ok(Some(saved)) = ListRepository::get_entry(&state.pool, user_id, &body.cid).await {
                if let Some(entry_id) = saved.id {
                    if let Err(e) = ListRepository::insert_changes(
                        &state.pool,
                        entry_id,
                        user_id,
                        ChangeSource::Local.as_str(),
                        None,
                        &changes_rows,
                    ).await {
                        error!(error = ?e, "Failed to write changelog");
                    }
                }
            }
        }

        info!(is_new = is_new, "List entry successfully saved");

        let cid_clone   = body.cid.clone();
        let state_clone = state.clone();
        tokio::task::spawn(async move {
            if let Err(e) = Self::sync_entry_to_all_trackers(&state_clone, user_id, &cid_clone).await {
                error!(error = ?e, cid = %cid_clone, "Background sync failed for entry");
            } else {
                debug!(cid = %cid_clone, "Background sync completed successfully");
            }
        });

        Ok(UpsertEntryResponse { success: true, changes, is_new })
    }

    #[instrument(skip(state))]
    pub async fn delete_entry(
        state: Arc<AppState>,
        user_id: i32,
        cid: String,
    ) -> CoreResult<SuccessResponse> {
        let state_clone = state.clone();
        let cid_clone   = cid.clone();
        tokio::task::spawn(async move {
            if let Err(e) = Self::delete_from_trackers(&state_clone, user_id, &cid_clone).await {
                warn!(error = ?e, "Failed to delete entry from external trackers");
            }
        });

        let deleted = ListRepository::delete_entry(&state.pool, user_id, &cid).await?;

        if deleted {
            info!("Entry deleted successfully from local database");
            Ok(SuccessResponse { success: true })
        } else {
            Err(CoreError::NotFound("error.list.entry_not_found".into()))
        }
    }

    async fn enrich_entries(
        state: &AppState,
        pool: &SqlitePool,
        entries: Vec<ListEntry>,
    ) -> CoreResult<Vec<EnrichedListEntry>> {
        let futures = entries.into_iter().map(|entry| {
            let pool_clone = pool.clone();
            async move {
                let full_content = ContentRepository::get_full_content(&pool_clone, &entry.cid).await.ok().flatten();

                match full_content {
                    Some(full) => {
                        let content_type = full.content.content_type.as_str().to_string();
                        let nsfw         = full.content.nsfw;

                        let meta = full.primary_metadata();

                        let title       = meta.map(|m| m.title.clone()).unwrap_or_else(|| "Unknown".into());
                        let title_i18n = meta.as_ref().map(|m| m.title_i18n.clone()).unwrap_or_default();
                        let cover_image = meta.and_then(|m| m.cover_image.clone());
                        let external_ids = meta.map(|m| m.external_ids.clone()).unwrap_or(json!({}));

                        let total = meta.map(|m| match &m.eps_or_chapters {
                            EpisodeData::Count(n) => *n,
                            EpisodeData::List(l)  => l.len() as i32,
                        });

                        let mut tracker_ids = serde_json::Map::new();
                        for mapping in &full.tracker_mappings {
                            tracker_ids.insert(
                                mapping.tracker_name.clone(),
                                Value::String(mapping.tracker_id.clone()),
                            );
                        }

                        let sources = if let Some(entry_id) = entry.id {
                            ListRepository::get_entry_sources(&state.pool, entry_id)
                                .await
                                .unwrap_or_default()
                        } else {
                            vec![]
                        };

                        EnrichedListEntry {
                            entry,
                            title,
                            title_i18n,
                            cover_image,
                            content_type,
                            nsfw,
                            total_units: total,
                            tracker_ids: Value::Object(tracker_ids),
                            external_ids,
                            has_extension_source: !full.extension_sources.is_empty(),
                            sources
                        }
                    }
                    None => EnrichedListEntry {
                        entry,
                        title: "Unknown Content".into(),
                        title_i18n: Default::default(),
                        cover_image: None,
                        content_type: "unknown".into(),
                        nsfw: false,
                        total_units: None,
                        tracker_ids: json!({}),
                        external_ids: json!({}),
                        has_extension_source: false,
                        sources: vec![],
                    },
                }
            }
        });

        Ok(join_all(futures).await)
    }

    #[instrument(skip(state))]
    async fn sync_entry_to_all_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = TrackerRepository::get_user_integrations(&state.pool, user_id).await?;

        for integration in integrations {
            if !integration.sync_enabled { continue; }
            if let Err(e) = Self::sync_entry_to_single_tracker(state, user_id, cid, &integration).await {
                error!(tracker = %integration.tracker_name, error = ?e, "Sync error for tracker");
            }
        }
        Ok(())
    }

    async fn sync_entry_to_single_tracker(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
        integration: &TrackerIntegration,
    ) -> CoreResult<()> {
        let provider = match state.tracker_registry.get(&integration.tracker_name) {
            Some(p) => p,
            None => {
                warn!("Tracker '{}' not found in registry, skipping sync", integration.tracker_name);
                return Ok(());
            }
        };

        let remote_id = {
            let mappings  = TrackerRepository::get_mappings_by_cid(&state.pool, cid).await?;
            mappings.into_iter()
                .find(|m| m.tracker_name == integration.tracker_name)
                .map(|m| m.tracker_id)
        };

        let remote_id = match remote_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let entry = ListRepository::get_entry(&state.pool, user_id, cid).await?;

        let entry = match entry {
            Some(e) => e,
            None => return Ok(()),
        };

        let params = UpdateEntryParams {
            media_id:     remote_id,
            status:       Some(entry.status),
            progress:     Some(entry.progress),
            score:        entry.score,
            start_date:   entry.start_date,
            end_date:     entry.end_date,
            repeat_count: Some(entry.repeat_count),
            notes:        entry.notes,
            is_private:   Some(entry.is_private),
            score_format: integration.score_format.clone(),
        };

        provider.update_entry(&integration.access_token, params).await?;
        Ok(())
    }

    async fn delete_from_trackers(
        state: &Arc<AppState>,
        user_id: i32,
        cid: &str,
    ) -> CoreResult<()> {
        let integrations = TrackerRepository::get_user_integrations(&state.pool, user_id).await?;

        for integration in integrations {
            if !integration.sync_enabled { continue; }

            let provider = match state.tracker_registry.get(&integration.tracker_name) {
                Some(p) => p,
                None => {
                    warn!("Tracker '{}' not found in registry, skipping delete", integration.tracker_name);
                    continue;
                }
            };

            let remote_id = {
                let mappings  = TrackerRepository::get_mappings_by_cid(&state.pool, cid).await?;
                mappings.into_iter()
                    .find(|m| m.tracker_name == integration.tracker_name)
                    .map(|m| m.tracker_id)
            };

            if let Some(id) = remote_id {
                if let Err(e) = provider.delete_entry(&integration.access_token, &id).await {
                    error!("Failed to delete from tracker '{}': {}", integration.tracker_name, e);
                }
            }
        }

        Ok(())
    }

    pub fn diff_entry(
        prev: Option<&ListEntry>,
        final_status: &str,
        new_progress: i32,
        body: &UpsertEntryBody,
        final_start_date: &Option<String>,
        final_end_date: &Option<String>,
    ) -> Vec<(&'static str, Option<String>, String)> {
        let mut changes = vec![];

        macro_rules! diff {
        ($field:expr, $old:expr, $new:expr) => {
            let old_s = $old.as_ref().map(|v: &String| v.clone());
            let new_s = $new.clone();
            if prev.is_none() || old_s.as_deref() != Some(new_s.as_str()) {
                changes.push(($field, old_s, new_s));
            }
        };
    }

        diff!("status",
        prev.map(|e| e.status.clone()),
        final_status.to_string());

        diff!("progress",
        prev.map(|e| e.progress.to_string()),
        new_progress.to_string());

        if let Some(score) = body.score {
            diff!("score",
            prev.and_then(|e| e.score).map(|s| s.to_string()),
            score.to_string());
        }

        diff!("start_date",
        prev.and_then(|e| e.start_date.clone()),
        final_start_date.clone().unwrap_or_default());

        if let Some(ed) = final_end_date {
            diff!("end_date",
            prev.and_then(|e| e.end_date.clone()),
            ed.clone());
        }

        if let Some(notes) = &body.notes {
            diff!("notes",
            prev.and_then(|e| e.notes.clone()),
            notes.clone());
        }

        changes
    }

    pub async fn get_entry_history(
        state: &AppState,
        user_id: i32,
        cid: String,
    ) -> CoreResult<EntryHistoryResponse> {
        let entry = ListRepository::get_entry(&state.pool, user_id, &cid)
            .await?
            .ok_or_else(|| CoreError::NotFound("error.list.entry_not_found".into()))?;

        let entry_id = entry.id.ok_or_else(|| CoreError::Internal("error.list.missing_id".into()))?;

        let changes = ListRepository::get_entry_history(&state.pool, entry_id, user_id).await?;
        Ok(EntryHistoryResponse { changes })
    }

    pub async fn get_activity_feed(
        state: &AppState,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<EntryHistoryResponse> {
        let changes = ListRepository::get_user_history(&state.pool, user_id, limit).await?;
        Ok(EntryHistoryResponse { changes })
    }
}