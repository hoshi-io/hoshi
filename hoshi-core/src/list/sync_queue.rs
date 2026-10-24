use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use chrono::Utc;
use serde_json::json;
use sqlx::{SqliteConnection, SqlitePool};
use tokio::sync::{broadcast, Notify};
use tracing::{error, info, warn};

use crate::error::{CoreError, CoreResult};
use crate::list::repository::ListRepository;
use crate::list::types::{BatchHandle, BatchIssue, BatchStatus, DeleteTarget, SyncEvent, SyncOp, SyncStatus, TrackerSyncStatus};
use crate::state::AppState;
use crate::tracker::provider::UpdateEntryParams;
use crate::tracker::repository::TrackerRepository;
use crate::tracker::types::TrackerIntegration;

const MAX_ATTEMPTS: i32 = 5;
const OFFLINE_PROBE: Duration = Duration::from_secs(15);
const AUTH_PAUSE: Duration = Duration::from_secs(300);
const RATE_LIMIT_PAUSE: Duration = Duration::from_secs(60);
const DEFAULT_MIN_INTERVAL: Duration = Duration::from_millis(700);
const IDLE_WAKE: Duration = Duration::from_secs(300);
const RETAIN_FINISHED_DAYS: i64 = 7;

fn notifiers() -> &'static Mutex<HashMap<String, Arc<Notify>>> {
    static N: OnceLock<Mutex<HashMap<String, Arc<Notify>>>> = OnceLock::new();
    N.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lane_notify(tracker: &str) -> Arc<Notify> {
    let mut map = notifiers().lock().unwrap();
    map.entry(tracker.to_string())
        .or_insert_with(|| Arc::new(Notify::new()))
        .clone()
}

fn pauses() -> &'static Mutex<HashMap<String, (Instant, &'static str)>> {
    static P: OnceLock<Mutex<HashMap<String, (Instant, &'static str)>>> = OnceLock::new();
    P.get_or_init(|| Mutex::new(HashMap::new()))
}

fn events() -> &'static broadcast::Sender<SyncEvent> {
    static E: OnceLock<broadcast::Sender<SyncEvent>> = OnceLock::new();
    E.get_or_init(|| broadcast::channel(256).0)
}

fn emit(tracker: &str, batch_id: Option<String>) {
    let _ = events().send(SyncEvent { tracker: tracker.to_string(), batch_id });
}


struct Job {
    id: i64,
    user_id: i32,
    cid: String,
    tracker: String,
    op: String,
    remote_id: Option<String>,
    attempts: i32,
    batch_id: Option<String>,
}

enum Outcome {
    Done,
    Skipped(&'static str),
    Offline,
    AuthExpired,
    RateLimited,
    Failed(String),
}

pub struct SyncQueue;

impl SyncQueue {

    /// Call once at app startup (after AppState is built).
    /// Re-queues interrupted jobs, purges old finished ones, spawns one lane per tracker.
    pub async fn start(state: Arc<AppState>) {
        if let Err(e) = Self::recover(&state.pool).await {
            error!(error = ?e, "Sync queue recovery failed");
        }

        let names = state.tracker_registry.names();
        info!(trackers = ?names, "Sync queue: spawning lanes");
        for tracker in names {
            let st = state.clone();
            tokio::spawn(async move { Self::run_lane(st, tracker).await });
        }
        info!("Sync queue workers started");
    }

    /// Subscribe to queue activity (the Tauri layer forwards this to the frontend, throttled).
    pub fn subscribe() -> broadcast::Receiver<SyncEvent> {
        events().subscribe()
    }

    /// Call after the user reconnects / re-authenticates a tracker so its lane resumes immediately.
    pub fn resume_lane(tracker: &str) {
        pauses().lock().unwrap().remove(tracker);
        lane_notify(tracker).notify_one();
    }

    async fn recover(pool: &SqlitePool) -> CoreResult<()> {
        ListRepository::recover_sync_jobs(pool, RETAIN_FINISHED_DAYS).await
    }

    pub fn new_batch_id(user_id: i32) -> String {
        format!("batch-{}-{}", user_id, Utc::now().timestamp_micros())
    }

    /// Enqueue an UPSERT for each cid to each sync-enabled tracker.
    ///
    /// * `only_tracker = None`  → normal edits: only trackers where the entry has a mapping
    ///   (no mapping = nothing to push, same as the old silent skip).
    /// * `only_tracker = Some`  → explicit push ("on X but not Y"): enqueued even without a
    ///   mapping so the job ends as SKIPPED("no_mapping") and the user sees why.
    ///
    /// Returns the number of jobs written (after coalescing they may overlap existing ones).
    pub async fn enqueue_entries(
        pool: &SqlitePool,
        user_id: i32,
        cids: &[String],
        batch_id: Option<&str>,
        only_tracker: Option<&str>,
    ) -> CoreResult<usize> {
        if cids.is_empty() {
            return Ok(0);
        }

        let integrations: Vec<TrackerIntegration> =
            TrackerRepository::get_user_integrations(pool, user_id)
                .await?
                .into_iter()
                .filter(|i| i.sync_enabled)
                .filter(|i| only_tracker.map_or(true, |t| t == i.tracker_name))
                .collect();

        info!(
            user_id,
            ?cids,
            ?only_tracker,
            integrations = ?integrations.iter().map(|i| i.tracker_name.as_str()).collect::<Vec<_>>(),
            "Sync enqueue: sync-enabled integrations"
        );

        if integrations.is_empty() {
            warn!(user_id, "Sync enqueue: no sync-enabled integrations for this user, nothing queued");
            return match only_tracker {
                Some(_) => Err(CoreError::NotFound("error.sync.tracker_not_connected".into())),
                None => Ok(0),
            };
        }

        let mut jobs: Vec<(&String, &str)> = vec![];
        for cid in cids {
            let mappings = TrackerRepository::get_mappings_by_cid(pool, cid).await?;
            for integ in &integrations {
                let has_mapping = mappings.iter().any(|m| m.tracker_name == integ.tracker_name);
                if has_mapping || only_tracker.is_some() {
                    jobs.push((cid, integ.tracker_name.as_str()));
                } else {
                    warn!(
                        %cid,
                        tracker = %integ.tracker_name,
                        mappings = ?mappings.iter().map(|m| m.tracker_name.as_str()).collect::<Vec<_>>(),
                        "Sync enqueue: no mapping for this tracker, skipping"
                    );
                }
            }
        }
        info!(jobs = jobs.len(), "Sync enqueue: jobs to write");

        let mut tx = pool.begin().await?;
        for (cid, tracker) in &jobs {
            Self::enqueue_one(&mut *tx, user_id, cid, tracker, SyncOp::Upsert, None, batch_id).await?;
        }
        tx.commit().await?;

        Self::wake(jobs.iter().map(|(_, t)| *t), batch_id);
        Ok(jobs.len())
    }

    /// Read mappings for entries that are about to be deleted locally.
    /// Must run BEFORE the local delete.
    pub async fn capture_delete_targets(
        pool: &SqlitePool,
        user_id: i32,
        cids: &[String],
    ) -> CoreResult<Vec<DeleteTarget>> {
        let integrations: Vec<TrackerIntegration> =
            TrackerRepository::get_user_integrations(pool, user_id)
                .await?
                .into_iter()
                .filter(|i| i.sync_enabled)
                .collect();

        info!(
            user_id,
            ?cids,
            integrations = ?integrations.iter().map(|i| i.tracker_name.as_str()).collect::<Vec<_>>(),
            "Sync delete capture: sync-enabled integrations"
        );

        let mut targets = vec![];
        if integrations.is_empty() {
            warn!(user_id, "Sync delete capture: no sync-enabled integrations, nothing to delete remotely");
            return Ok(targets);
        }
        for cid in cids {
            let mappings = TrackerRepository::get_mappings_by_cid(pool, cid).await?;
            for integ in &integrations {
                if let Some(m) = mappings.iter().find(|m| m.tracker_name == integ.tracker_name) {
                    targets.push(DeleteTarget {
                        cid: cid.clone(),
                        tracker: integ.tracker_name.clone(),
                        remote_id: m.tracker_id.clone(),
                    });
                } else {
                    warn!(
                        %cid,
                        tracker = %integ.tracker_name,
                        mappings = ?mappings.iter().map(|m| m.tracker_name.as_str()).collect::<Vec<_>>(),
                        "Sync delete capture: no mapping for this tracker, skipping"
                    );
                }
            }
        }
        info!(targets = targets.len(), "Sync delete capture: targets captured");
        Ok(targets)
    }

    /// Enqueue DELETE jobs (call AFTER the local delete succeeded).
    /// A pending UPSERT for the same (cid, tracker) is superseded by the DELETE.
    pub async fn enqueue_deletes(
        pool: &SqlitePool,
        user_id: i32,
        targets: &[DeleteTarget],
        batch_id: Option<&str>,
    ) -> CoreResult<usize> {
        if targets.is_empty() {
            info!("Sync enqueue_deletes: no targets, nothing queued");
            return Ok(0);
        }
        info!(count = targets.len(), "Sync enqueue_deletes: queueing deletes");
        let mut tx = pool.begin().await?;
        for t in targets {
            Self::enqueue_one(
                &mut *tx,
                user_id,
                &t.cid,
                &t.tracker,
                SyncOp::Delete,
                Some(&t.remote_id),
                batch_id,
            )
                .await?;
        }
        tx.commit().await?;

        Self::wake(targets.iter().map(|t| t.tracker.as_str()), batch_id);
        Ok(targets.len())
    }

    /// "On X but not Y" push button: UPSERT these cids to one specific tracker.
    pub async fn push_to_tracker(
        pool: &SqlitePool,
        user_id: i32,
        tracker: &str,
        cids: &[String],
    ) -> CoreResult<BatchHandle> {
        let batch_id = Self::new_batch_id(user_id);
        let enqueued = Self::enqueue_entries(pool, user_id, cids, Some(&batch_id), Some(tracker)).await?;
        Ok(BatchHandle { batch_id, enqueued })
    }

    /// Latest op wins. Re-enqueueing resets attempts/backoff and takes the new batch_id
    /// (so a job re-touched by a newer batch moves to that batch's progress).
    async fn enqueue_one(
        conn: &mut SqliteConnection,
        user_id: i32,
        cid: &str,
        tracker: &str,
        op: SyncOp,
        remote_id: Option<&str>,
        batch_id: Option<&str>,
    ) -> CoreResult<()> {
        ListRepository::enqueue_sync_job_conn(conn, user_id, cid, tracker, op.as_str(), remote_id, batch_id).await
    }

    fn wake<'a>(trackers: impl Iterator<Item = &'a str>, batch_id: Option<&str>) {
        let mut seen: Vec<&str> = vec![];
        for t in trackers {
            if !seen.contains(&t) {
                seen.push(t);
                lane_notify(t).notify_one();
                emit(t, batch_id.map(str::to_string));
            }
        }
    }


    pub async fn batch_status(pool: &SqlitePool, user_id: i32, batch_id: &str) -> CoreResult<BatchStatus> {
        let rows = ListRepository::sync_batch_status_counts(pool, user_id, batch_id).await?;

        let mut s = BatchStatus { batch_id: batch_id.to_string(), ..Default::default() };
        for (status, n) in rows {
            s.total += n;
            match status.as_str() {
                "PENDING" => s.pending = n,
                "RUNNING" => s.running = n,
                "DONE" => s.done = n,
                "FAILED" => s.failed = n,
                "SKIPPED" => s.skipped = n,
                _ => {}
            }
        }
        s.finished = s.pending == 0 && s.running == 0;
        Ok(s)
    }

    pub async fn batch_issues(pool: &SqlitePool, user_id: i32, batch_id: &str) -> CoreResult<Vec<BatchIssue>> {
        let rows = ListRepository::sync_batch_issue_rows(pool, user_id, batch_id).await?;

        Ok(rows
            .into_iter()
            .map(|(cid, tracker, op, status, reason)| BatchIssue { cid, tracker, op, status, reason })
            .collect())
    }

    pub async fn sync_status(pool: &SqlitePool, user_id: i32) -> CoreResult<SyncStatus> {
        let rows = ListRepository::sync_job_counts(pool, user_id).await?;

        let mut map: HashMap<String, TrackerSyncStatus> = HashMap::new();
        for (tracker, status, n) in rows {
            let e = map.entry(tracker.clone()).or_insert_with(|| TrackerSyncStatus {
                tracker,
                ..Default::default()
            });
            match status.as_str() {
                "PENDING" => e.pending = n,
                "RUNNING" => e.running = n,
                "FAILED" => e.failed = n,
                _ => {}
            }
        }

        let now = Instant::now();
        for (tracker, (until, reason)) in pauses().lock().unwrap().iter() {
            if *until > now {
                map.entry(tracker.clone())
                    .or_insert_with(|| TrackerSyncStatus { tracker: tracker.clone(), ..Default::default() })
                    .paused_reason = Some((*reason).to_string());
            }
        }

        let mut trackers: Vec<_> = map.into_values().collect();
        trackers.sort_by(|a, b| a.tracker.cmp(&b.tracker));
        Ok(SyncStatus { trackers })
    }

    /// Re-queue FAILED jobs (of one batch, or all of them when `batch_id` is None).
    pub async fn retry_failed(pool: &SqlitePool, user_id: i32, batch_id: Option<&str>) -> CoreResult<usize> {
        let n = ListRepository::requeue_failed_sync_jobs(pool, user_id, batch_id).await?;

        if n > 0 {
            for t in Self::pending_trackers(pool, user_id).await? {
                lane_notify(&t).notify_one();
                emit(&t, batch_id.map(str::to_string));
            }
        }
        Ok(n)
    }

    /// Drop PENDING jobs of a batch. Already-running/finished jobs are untouched.
    pub async fn cancel_batch(pool: &SqlitePool, user_id: i32, batch_id: &str) -> CoreResult<usize> {
        ListRepository::cancel_pending_sync_batch(pool, user_id, batch_id).await
    }

    async fn pending_trackers(pool: &SqlitePool, user_id: i32) -> CoreResult<Vec<String>> {
        ListRepository::pending_sync_trackers(pool, user_id).await
    }

    async fn run_lane(state: Arc<AppState>, tracker: String) {
        let notify = lane_notify(&tracker);
        let min_interval = state
            .tracker_registry
            .get(&tracker)
            .map(|p| p.rate_limit().min_interval)
            .unwrap_or(DEFAULT_MIN_INTERVAL);

        info!(tracker = %tracker, min_interval_ms = min_interval.as_millis() as u64, "Sync lane running");

        loop {
            if let Some(wait) = Self::pause_remaining(&tracker) {
                info!(tracker = %tracker, wait_secs = wait.as_secs(), "Sync lane paused");
                // A wake (new job / resume_lane) re-evaluates; pause stays unless cleared.
                tokio::select! {
                    _ = notify.notified() => {}
                    _ = tokio::time::sleep(wait) => {}
                }
                continue;
            }

            let now = Utc::now().timestamp();
            match Self::claim_next(&state.pool, &tracker, now).await {
                Ok(Some(job)) => {
                    info!(
                        job_id = job.id, tracker = %job.tracker, cid = %job.cid,
                        op = %job.op, attempts = job.attempts, batch = ?job.batch_id,
                        "Sync job claimed"
                    );
                    let started = Instant::now();
                    Self::process(&state, &job).await;
                    if let Some(rest) = min_interval.checked_sub(started.elapsed()) {
                        tokio::time::sleep(rest).await;
                    }
                }
                Ok(None) => {
                    let wait = Self::time_until_next_due(&state.pool, &tracker, now).await;
                    tokio::select! {
                        _ = notify.notified() => {}
                        _ = tokio::time::sleep(wait) => {}
                    }
                }
                Err(e) => {
                    error!(tracker = %tracker, error = ?e, "Sync lane DB error");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn claim_next(pool: &SqlitePool, tracker: &str, now: i64) -> CoreResult<Option<Job>> {
        let row = ListRepository::claim_next_sync_job(pool, tracker, now).await?;

        Ok(row.map(|(id, user_id, cid, tracker, op, remote_id, attempts, batch_id)| Job {
            id,
            user_id,
            cid,
            tracker,
            op,
            remote_id,
            attempts,
            batch_id,
        }))
    }

    async fn time_until_next_due(pool: &SqlitePool, tracker: &str, now: i64) -> Duration {
        let next = ListRepository::next_pending_sync_run_at(pool, tracker).await.ok().flatten();

        match next {
            Some(t) => Duration::from_secs((t - now).clamp(1, IDLE_WAKE.as_secs() as i64) as u64),
            None => IDLE_WAKE,
        }
    }

    fn pause(tracker: &str, dur: Duration, reason: &'static str) {
        pauses().lock().unwrap().insert(tracker.to_string(), (Instant::now() + dur, reason));
    }

    fn pause_remaining(tracker: &str) -> Option<Duration> {
        let mut map = pauses().lock().unwrap();
        match map.get(tracker) {
            Some((until, _)) => {
                let now = Instant::now();
                if *until > now {
                    Some(*until - now)
                } else {
                    map.remove(tracker);
                    None
                }
            }
            None => None,
        }
    }

    async fn process(state: &Arc<AppState>, job: &Job) {
        let outcome = match Self::execute(state, job).await {
            Ok(o) => o,
            Err(e) => {
                let outcome = Self::classify(&e);
                // Log the raw error: Offline/AuthExpired/RateLimited are otherwise silent.
                warn!(
                    job_id = job.id, tracker = %job.tracker, cid = %job.cid, op = %job.op,
                    error = %e,
                    classified = match &outcome {
                        Outcome::Offline => "offline",
                        Outcome::AuthExpired => "auth_expired",
                        Outcome::RateLimited => "rate_limited",
                        Outcome::Failed(_) => "failed",
                        _ => "other",
                    },
                    "Sync job errored"
                );
                outcome
            }
        };
        let pool = &state.pool;
        let now = Utc::now().timestamp();

        let res = match outcome {
            Outcome::Done => {
                info!(job_id = job.id, tracker = %job.tracker, cid = %job.cid, op = %job.op, "Sync job done");
                Self::complete(pool, job.id, "DONE", None).await
            }
            Outcome::Skipped(reason) => {
                info!(tracker = %job.tracker, cid = %job.cid, reason, "Sync job skipped");
                Self::complete(pool, job.id, "SKIPPED", Some(reason)).await
            }
            // Not the job's fault: don't burn an attempt, pause the lane instead.
            Outcome::Offline => {
                Self::pause(&job.tracker, OFFLINE_PROBE, "offline");
                Self::release(pool, job.id, now + OFFLINE_PROBE.as_secs() as i64, false, Some("offline")).await
            }
            Outcome::AuthExpired => {
                Self::pause(&job.tracker, AUTH_PAUSE, "auth_expired");
                Self::release(pool, job.id, now + AUTH_PAUSE.as_secs() as i64, false, Some("auth_expired")).await
            }
            Outcome::RateLimited => {
                Self::pause(&job.tracker, RATE_LIMIT_PAUSE, "rate_limited");
                Self::release(pool, job.id, now + RATE_LIMIT_PAUSE.as_secs() as i64, false, Some("rate_limited")).await
            }
            Outcome::Failed(msg) => {
                warn!(tracker = %job.tracker, cid = %job.cid, attempts = job.attempts, error = %msg, "Sync job failed");
                if job.attempts >= MAX_ATTEMPTS {
                    Self::complete(pool, job.id, "FAILED", Some(msg.as_str())).await
                } else {
                    let backoff = (15i64 << (job.attempts - 1).clamp(0, 8)).min(3600);
                    Self::release(pool, job.id, now + backoff, true, Some(msg.as_str())).await
                }
            }
        };

        if let Err(e) = res {
            error!(job_id = job.id, error = ?e, "Failed to record sync job result");
        }
        emit(&job.tracker, job.batch_id.clone());
    }

    async fn complete(pool: &SqlitePool, id: i64, status: &str, reason: Option<&str>) -> CoreResult<()> {
        ListRepository::complete_sync_job(pool, id, status, reason).await
    }

    async fn release(
        pool: &SqlitePool,
        id: i64,
        next_run_at: i64,
        burn_attempt: bool,
        reason: Option<&str>,
    ) -> CoreResult<()> {
        ListRepository::release_sync_job(pool, id, next_run_at, burn_attempt, reason).await
    }

    fn classify(e: &CoreError) -> Outcome {
        let msg = e.to_string();
        let l = msg.to_lowercase();

        if l.contains("429") || l.contains("rate limit") || l.contains("too many requests") {
            Outcome::RateLimited
        } else if l.contains("401") || l.contains("unauthorized") || l.contains("invalid token") || l.contains("token expired") {
            Outcome::AuthExpired
        } else if l.contains("error sending request")
            || l.contains("dns")
            || l.contains("connect")
            || l.contains("timed out")
            || l.contains("timeout")
            || l.contains("network")
            || l.contains("offline")
        {
            Outcome::Offline
        } else {
            Outcome::Failed(msg)
        }
    }

    async fn execute(state: &Arc<AppState>, job: &Job) -> CoreResult<Outcome> {
        let integration = TrackerRepository::get_user_integrations(&state.pool, job.user_id)
            .await?
            .into_iter()
            .find(|i| i.tracker_name == job.tracker);

        let integration = match integration {
            Some(i) => i,
            None => return Ok(Outcome::Skipped("not_connected")),
        };
        if !integration.sync_enabled {
            return Ok(Outcome::Skipped("sync_disabled"));
        }

        let provider = match state.tracker_registry.get(&job.tracker) {
            Some(p) => p,
            None => return Ok(Outcome::Skipped("unknown_tracker")),
        };

        if job.op == SyncOp::Delete.as_str() {
            let remote_id = match job.remote_id.as_deref() {
                Some(id) => id,
                None => return Ok(Outcome::Skipped("no_remote_id")),
            };
            // `false` = already gone remotely; that's the desired end state.
            provider.delete_entry(&integration.access_token, remote_id).await?;
            return Ok(Outcome::Done);
        }

        // UPSERT: read the *current* entry; this is what makes coalescing safe.
        let remote_id = TrackerRepository::get_mappings_by_cid(&state.pool, &job.cid)
            .await?
            .into_iter()
            .find(|m| m.tracker_name == job.tracker)
            .map(|m| m.tracker_id);
        let remote_id = match remote_id {
            Some(id) => id,
            None => return Ok(Outcome::Skipped("no_mapping")),
        };

        let entry = match ListRepository::get_entry(&state.pool, job.user_id, &job.cid).await? {
            Some(e) => e,
            None => return Ok(Outcome::Skipped("entry_missing")),
        };

        let snapshot = json!({
            "status":      entry.status,
            "progress":    entry.progress,
            "score":       entry.score,
            "startDate":   entry.start_date,
            "endDate":     entry.end_date,
            "repeatCount": entry.repeat_count,
        });
        let entry_id = entry.id;

        let params = UpdateEntryParams {
            media_id:     remote_id.clone(),
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

        if let Some(entry_id) = entry_id {
            if let Err(e) = ListRepository::upsert_entry_source(
                &state.pool,
                entry_id,
                job.user_id,
                &job.tracker,
                &remote_id,
                &snapshot,
            )
                .await
            {
                warn!(error = ?e, cid = %job.cid, "Pushed, but failed to record entry source");
            }
        }

        Ok(Outcome::Done)
    }
}