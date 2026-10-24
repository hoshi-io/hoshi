use std::sync::Arc;
use serde_json::{json, Value};
use chrono::Utc;
use futures::future::join_all;
use sqlx::{SqliteConnection, SqlitePool};
use tracing::{error, info, instrument, warn};
use crate::content::models::EpisodeData;
use crate::content::repositories::content::ContentRepository;
use crate::diff_field;
use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::list::sync_queue::SyncQueue;
use crate::list::types::{BatchItemError, BatchWriteResponse, ChangeSource, EnrichedListEntry, EntryHistoryResponse, FilterQuery, FinalFields, ListEntry, ListResponse, SingleEntryResponse, SuccessResponse, UpsertEntryBody, UpsertEntryResponse, UserStats};
use crate::state::AppState;

pub struct ListService;

struct UpsertOutcome {
    changes: usize,
    is_new: bool,
    written: bool,
}

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
    
    fn compute_final_fields(
        prev_entry: Option<&ListEntry>,
        body: &UpsertEntryBody,
        total_units: Option<i32>,
        new_progress: i32,
    ) -> FinalFields {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut final_start_date = body.start_date.clone();
        let mut final_end_date   = body.end_date.clone();
        let mut final_status     = body.status.clone();

        if let Some(prev) = prev_entry {
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

        FinalFields { status: final_status, progress: new_progress, start_date: final_start_date, end_date: final_end_date }
    }

    /// Validates that the content exists and returns its total episodes/chapters.
    async fn resolve_total_units(pool: &SqlitePool, cid: &str) -> CoreResult<Option<i32>> {
        ContentRepository::get_content_by_cid(pool, cid).await?
            .ok_or_else(|| {
                warn!("Attempted to upsert entry for non-existent content");
                CoreError::NotFound("error.list.content_not_found".into())
            })?;

        Ok(ContentRepository::get_by_cid(pool, cid).await?
            .map(|m| match m.eps_or_chapters {
                EpisodeData::Count(n) => n,
                EpisodeData::List(ref l)  => l.len() as i32,
            }))
    }
    
    async fn apply_upsert(
        conn: &mut SqliteConnection,
        user_id: i32,
        body: &UpsertEntryBody,
        total_units: Option<i32>,
    ) -> CoreResult<UpsertOutcome> {
        let prev_entry = ListRepository::get_entry_conn(conn, user_id, &body.cid).await?;

        let is_new        = prev_entry.is_none();
        let prev_progress = prev_entry.as_ref().map(|e| e.progress).unwrap_or(0);
        let new_progress  = body.progress.unwrap_or(prev_progress);

        if !is_new && body.progress.is_some() && new_progress < prev_progress {
            warn!(prev = prev_progress, new = new_progress, "Ignoring progress downgrade for user");
            return Ok(UpsertOutcome { changes: 0, is_new: false, written: false });
        }

        let fields = Self::compute_final_fields(prev_entry.as_ref(), body, total_units, new_progress);
        let changes = Self::persist_and_diff(conn, user_id, body, prev_entry.as_ref(), &fields).await?;

        Ok(UpsertOutcome { changes, is_new, written: true })
    }

    #[instrument(skip(state, body), fields(cid = %body.cid, status = %body.status))]
    pub async fn upsert_entry(
        state: Arc<AppState>,
        user_id: i32,
        body: UpsertEntryBody,
    ) -> CoreResult<UpsertEntryResponse> {
        let total_units = Self::resolve_total_units(&state.pool, &body.cid).await?;

        let outcome = {
            let mut conn = state.pool.acquire().await?;
            Self::apply_upsert(&mut conn, user_id, &body, total_units).await?
        };

        if outcome.written {
            info!(is_new = outcome.is_new, "List entry successfully saved");

            // Tracker sync is eventual: record the intent, the worker does the rest.
            // A queue failure must not fail the (already committed) local save.
            if let Err(e) = SyncQueue::enqueue_entries(
                &state.pool, user_id, std::slice::from_ref(&body.cid), None, None,
            ).await {
                error!(error = ?e, cid = %body.cid, "Failed to enqueue tracker sync");
            }
        }

        Ok(UpsertEntryResponse { success: true, changes: outcome.changes, is_new: outcome.is_new })
    }
    
    #[instrument(skip(state, bodies), fields(count = bodies.len()))]
    pub async fn batch_upsert_entries(
        state: Arc<AppState>,
        user_id: i32,
        bodies: Vec<UpsertEntryBody>,
    ) -> CoreResult<BatchWriteResponse> {
        let batch_id = SyncQueue::new_batch_id(user_id);
        let mut errors: Vec<BatchItemError> = vec![];

        // Reads first, outside the write transaction.
        let mut prepared = Vec::with_capacity(bodies.len());
        for body in bodies {
            match Self::resolve_total_units(&state.pool, &body.cid).await {
                Ok(total) => prepared.push((body, total)),
                Err(e) => errors.push(BatchItemError { cid: body.cid, error: e.to_string() }),
            }
        }

        let mut written: Vec<String> = vec![];
        let mut tx = state.pool.begin().await?;
        for (body, total) in &prepared {
            match Self::apply_upsert(&mut *tx, user_id, body, *total).await {
                Ok(o) if o.written => written.push(body.cid.clone()),
                Ok(_) => {}
                Err(e) => errors.push(BatchItemError { cid: body.cid.clone(), error: e.to_string() }),
            }
        }
        tx.commit().await?;

        written.sort();
        written.dedup();

        let enqueued = SyncQueue::enqueue_entries(&state.pool, user_id, &written, Some(&batch_id), None)
            .await
            .unwrap_or_else(|e| {
                error!(error = ?e, "Failed to enqueue batch sync");
                0
            });

        info!(applied = written.len(), enqueued, failed = errors.len(), "Batch upsert finished");
        Ok(BatchWriteResponse { batch_id, applied: written.len(), enqueued, errors })
    }

    async fn persist_and_diff(
        conn: &mut SqliteConnection,
        user_id: i32,
        body: &UpsertEntryBody,
        prev_entry: Option<&ListEntry>,
        fields: &FinalFields,
    ) -> CoreResult<usize> {
        let changes = ListRepository::upsert_entry_conn(
            conn,
            user_id,
            body,
            &fields.status,
            fields.progress,
            fields.start_date.clone(),
            fields.end_date.clone(),
        ).await?;

        let changes_rows = Self::diff_entry(
            prev_entry,
            &fields.status,
            fields.progress,
            body,
            &fields.start_date,
            &fields.end_date,
        );

        if !changes_rows.is_empty() {
            if let Ok(Some(saved)) = ListRepository::get_entry_conn(conn, user_id, &body.cid).await {
                if let Some(entry_id) = saved.id {
                    if let Err(e) = ListRepository::insert_changes_conn(
                        conn,
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

        Ok(changes)
    }

    #[instrument(skip(state))]
    pub async fn delete_entry(
        state: Arc<AppState>,
        user_id: i32,
        cid: String,
    ) -> CoreResult<SuccessResponse> {
        let targets = SyncQueue::capture_delete_targets(&state.pool, user_id, std::slice::from_ref(&cid))
            .await
            .unwrap_or_else(|e| {
                warn!(error = ?e, "Could not read tracker mappings; remote delete will not be queued");
                vec![]
            });

        let deleted = ListRepository::delete_entry(&state.pool, user_id, &cid).await?;

        if deleted {
            info!("Entry deleted successfully from local database");
            if let Err(e) = SyncQueue::enqueue_deletes(&state.pool, user_id, &targets, None).await {
                error!(error = ?e, "Failed to enqueue tracker deletes");
            }
            Ok(SuccessResponse { success: true })
        } else {
            Err(CoreError::NotFound("error.list.entry_not_found".into()))
        }
    }

    #[instrument(skip(state, cids), fields(count = cids.len()))]
    pub async fn batch_delete_entries(
        state: Arc<AppState>,
        user_id: i32,
        cids: Vec<String>,
    ) -> CoreResult<BatchWriteResponse> {
        let batch_id = SyncQueue::new_batch_id(user_id);

        let targets = SyncQueue::capture_delete_targets(&state.pool, user_id, &cids).await?;

        let mut deleted: Vec<String> = vec![];
        let mut tx = state.pool.begin().await?;
        for cid in &cids {
            if ListRepository::delete_entry_conn(&mut *tx, user_id, cid).await? {
                deleted.push(cid.clone());
            }
        }
        tx.commit().await?;

        let targets: Vec<_> = targets.into_iter().filter(|t| deleted.contains(&t.cid)).collect();
        let enqueued = SyncQueue::enqueue_deletes(&state.pool, user_id, &targets, Some(&batch_id))
            .await
            .unwrap_or_else(|e| {
                error!(error = ?e, "Failed to enqueue batch deletes");
                0
            });

        let errors = cids.iter()
            .filter(|c| !deleted.contains(c))
            .map(|c| BatchItemError { cid: c.clone(), error: "error.list.entry_not_found".into() })
            .collect();

        Ok(BatchWriteResponse { batch_id, applied: deleted.len(), enqueued, errors })
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

    pub fn diff_entry(
        prev: Option<&ListEntry>,
        final_status: &str,
        new_progress: i32,
        body: &UpsertEntryBody,
        final_start_date: &Option<String>,
        final_end_date: &Option<String>,
    ) -> Vec<(&'static str, Option<String>, String)> {
        let mut changes = vec![];
        let prev_is_none = prev.is_none();

        diff_field!(changes, prev_is_none, "status",
            prev.map(|e| e.status.clone()), final_status.to_string());

        diff_field!(changes, prev_is_none, "progress",
            prev.map(|e| e.progress.to_string()), new_progress.to_string());

        if let Some(score) = body.score {
            diff_field!(changes, prev_is_none, "score",
                prev.and_then(|e| e.score).map(|s| s.to_string()), score.to_string());
        }

        diff_field!(changes, prev_is_none, "start_date",
            prev.and_then(|e| e.start_date.clone()), final_start_date.clone().unwrap_or_default());

        if let Some(ed) = final_end_date {
            diff_field!(changes, prev_is_none, "end_date",
                prev.and_then(|e| e.end_date.clone()), ed.clone());
        }

        if let Some(notes) = &body.notes {
            diff_field!(changes, prev_is_none, "notes",
                prev.and_then(|e| e.notes.clone()), notes.clone());
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