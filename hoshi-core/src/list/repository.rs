use sqlx::{SqliteConnection, SqlitePool};
use crate::error::CoreResult;
use crate::list::types::{EntrySource, EntrySourceRow, ListEntry, ListEntryChange, ScoreDistribution, UpsertEntryBody, UserStats};

pub struct ListRepository;

pub type SyncJobRow = (i64, i32, String, String, String, Option<String>, i32, Option<String>);

impl ListRepository {
    pub async fn get_entry(pool: &SqlitePool, user_id: i32, cid: &str) -> CoreResult<Option<ListEntry>> {
        let mut conn = pool.acquire().await?;
        Self::get_entry_conn(&mut conn, user_id, cid).await
    }

    /// Same as `get_entry` but on an existing connection/transaction (used by batch writes).
    pub async fn get_entry_conn(conn: &mut SqliteConnection, user_id: i32, cid: &str) -> CoreResult<Option<ListEntry>> {
        let row = sqlx::query_as::<_, ListEntry>(
            "SELECT * FROM ListEntry WHERE user_id = ? AND cid = ?",
        )
            .bind(user_id)
            .bind(cid)
            .fetch_optional(&mut *conn)
            .await?;
        Ok(row)
    }

    pub async fn get_entries(
        pool: &SqlitePool,
        user_id: i32,
        status: Option<&str>,
    ) -> CoreResult<Vec<ListEntry>> {
        let rows = match status {
            Some(s) => sqlx::query_as::<_, ListEntry>(
                "SELECT * FROM ListEntry WHERE user_id = ? AND status = ? ORDER BY updated_at DESC",
            )
                .bind(user_id)
                .bind(s)
                .fetch_all(pool)
                .await?,
            None => sqlx::query_as::<_, ListEntry>(
                "SELECT * FROM ListEntry WHERE user_id = ? ORDER BY updated_at DESC",
            )
                .bind(user_id)
                .fetch_all(pool)
                .await?,
        };
        Ok(rows)
    }

    pub async fn upsert_entry(
        pool: &SqlitePool,
        user_id: i32,
        body: &UpsertEntryBody,
        final_status: &str,
        new_progress: i32,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> CoreResult<usize> {
        let mut conn = pool.acquire().await?;
        Self::upsert_entry_conn(&mut conn, user_id, body, final_status, new_progress, start_date, end_date).await
    }

    pub async fn upsert_entry_conn(
        conn: &mut SqliteConnection,
        user_id: i32,
        body: &UpsertEntryBody,
        final_status: &str,
        new_progress: i32,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> CoreResult<usize> {
        let rows = sqlx::query(
            r#"
            INSERT INTO ListEntry
                (user_id, cid, status, progress, score, start_date, end_date,
                 repeat_count, notes, is_private, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(user_id, cid) DO UPDATE SET
                status       = excluded.status,
                progress     = excluded.progress,
                score        = excluded.score,
                start_date   = excluded.start_date,
                end_date     = excluded.end_date,
                repeat_count = excluded.repeat_count,
                notes        = excluded.notes,
                is_private   = excluded.is_private,
                updated_at   = CURRENT_TIMESTAMP
            "#,
        )
            .bind(user_id)
            .bind(&body.cid)
            .bind(final_status)
            .bind(new_progress)
            .bind(body.score)
            .bind(start_date)
            .bind(end_date)
            .bind(body.repeat_count.unwrap_or(0))
            .bind(&body.notes)
            .bind(body.is_private.unwrap_or(false))
            .execute(&mut *conn)
            .await?
            .rows_affected() as usize;
        Ok(rows)
    }

    pub async fn delete_entry(pool: &SqlitePool, user_id: i32, cid: &str) -> CoreResult<bool> {
        let mut conn = pool.acquire().await?;
        Self::delete_entry_conn(&mut conn, user_id, cid).await
    }

    pub async fn delete_entry_conn(conn: &mut SqliteConnection, user_id: i32, cid: &str) -> CoreResult<bool> {
        let rows = sqlx::query(
            "DELETE FROM ListEntry WHERE user_id = ? AND cid = ?",
        )
            .bind(user_id)
            .bind(cid)
            .execute(&mut *conn)
            .await?
            .rows_affected();
        Ok(rows > 0)
    }

    pub async fn get_user_stats(pool: &SqlitePool, user_id: i32) -> CoreResult<UserStats> {
        // All counts in a single query
        let row: (i32, i32, i32, i32, i32, i32, i32) = sqlx::query_as(
            "SELECT
                COUNT(*),
                SUM(status = 'CURRENT'),
                SUM(status = 'COMPLETED'),
                SUM(status = 'PLANNING'),
                SUM(status = 'PAUSED'),
                SUM(status = 'DROPPED'),
                SUM(status = 'REPEATING')
             FROM ListEntry WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let (total_entries, watching, completed, planning, paused, dropped, repeating) = row;

        let mean_score: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(score) FROM ListEntry WHERE user_id = ? AND score IS NOT NULL",
        )
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .flatten();

        let score_distribution: Vec<ScoreDistribution> = sqlx::query_as(
            "SELECT CAST(ROUND(score) AS INTEGER) AS score, COUNT(*) AS count
             FROM ListEntry
             WHERE user_id = ? AND score IS NOT NULL AND score >= 1 AND score <= 10
             GROUP BY score ORDER BY score ASC",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        let days_since_last_activity: Option<i64> = sqlx::query_scalar(
            "SELECT CAST(julianday('now') - julianday(MAX(updated_at)) AS INTEGER)
             FROM ListEntry WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .flatten();

        let total_rewatches: i32 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(repeat_count), 0) FROM ListEntry WHERE user_id = ?",
        )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let entries_with_notes: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND notes IS NOT NULL AND notes != ''",
        )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let private_entries: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ListEntry WHERE user_id = ? AND is_private = 1",
        )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        let started = total_entries - planning;
        let completion_rate = (started > 0).then(|| completed as f64 / started as f64);

        Ok(UserStats {
            total_entries,
            watching,
            completed,
            planning,
            paused,
            dropped,
            repeating,
            total_episodes: 0,
            total_chapters: 0,
            mean_score,
            score_distribution,
            days_since_last_activity,
            completion_rate,
            total_rewatches,
            entries_with_notes,
            private_entries,
        })
    }

    pub async fn get_completed_entries_progress(
        pool: &SqlitePool,
        user_id: i32,
    ) -> CoreResult<Vec<(String, i32)>> {
        let rows: Vec<(String, i32)> = sqlx::query_as(
            "SELECT cid, progress FROM ListEntry WHERE user_id = ? AND status = 'COMPLETED'",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_entries_by_cids(
        pool: &SqlitePool,
        user_id: i32,
        cids: &[String],
    ) -> CoreResult<Vec<ListEntry>> {
        if cids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders = cids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT * FROM ListEntry WHERE user_id = ? AND cid IN ({})",
            placeholders
        );
        let mut query = sqlx::query_as::<_, ListEntry>(&sql).bind(user_id);
        for cid in cids {
            query = query.bind(cid);
        }
        Ok(query.fetch_all(pool).await?)
    }

    pub async fn insert_changes(
        pool: &SqlitePool,
        entry_id: i64,
        user_id: i32,
        source: &str,
        tracker: Option<&str>,
        changes: &[(&str, Option<String>, String)], // (field, old, new)
    ) -> CoreResult<()> {
        let mut conn = pool.acquire().await?;
        Self::insert_changes_conn(&mut conn, entry_id, user_id, source, tracker, changes).await
    }

    pub async fn insert_changes_conn(
        conn: &mut SqliteConnection,
        entry_id: i64,
        user_id: i32,
        source: &str,
        tracker: Option<&str>,
        changes: &[(&str, Option<String>, String)], // (field, old, new)
    ) -> CoreResult<()> {
        for (field, old_value, new_value) in changes {
            sqlx::query(
                r#"INSERT INTO ListEntryChange
               (entry_id, user_id, source, tracker, field, old_value, new_value)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            )
                .bind(entry_id)
                .bind(user_id)
                .bind(source)
                .bind(tracker)
                .bind(field)
                .bind(old_value.as_deref())
                .bind(new_value)
                .execute(&mut *conn)
                .await?;
        }
        Ok(())
    }

    pub async fn insert_deletion_change(
        pool: &SqlitePool,
        entry_id: i64,
        user_id: i32,
        source: &str,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"INSERT INTO ListEntryChange
           (entry_id, user_id, source, field, old_value, new_value)
           VALUES (?, ?, ?, 'deleted', NULL, 'true')"#,
        )
            .bind(entry_id)
            .bind(user_id)
            .bind(source)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_entry_history(
        pool: &SqlitePool,
        entry_id: i64,
        user_id: i32,
    ) -> CoreResult<Vec<ListEntryChange>> {
        let rows = sqlx::query_as::<_, ListEntryChange>(
            "SELECT * FROM ListEntryChange
         WHERE entry_id = ? AND user_id = ?
         ORDER BY changed_at DESC",
        )
            .bind(entry_id)
            .bind(user_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_user_history(
        pool: &SqlitePool,
        user_id: i32,
        limit: i64,
    ) -> CoreResult<Vec<ListEntryChange>> {
        let rows = sqlx::query_as::<_, ListEntryChange>(
            "SELECT * FROM ListEntryChange
         WHERE user_id = ?
         ORDER BY changed_at DESC
         LIMIT ?",
        )
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    pub async fn upsert_entry_source(
        pool: &SqlitePool,
        entry_id: i64,
        user_id: i32,
        tracker: &str,
        remote_id: &str,
        snapshot: &serde_json::Value,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"INSERT INTO ListEntrySource (entry_id, user_id, tracker, remote_id, remote_snapshot, synced_at)
           VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
           ON CONFLICT(entry_id, tracker) DO UPDATE SET
               remote_id       = excluded.remote_id,
               remote_snapshot = excluded.remote_snapshot,
               synced_at       = CURRENT_TIMESTAMP"#,
        )
            .bind(entry_id)
            .bind(user_id)
            .bind(tracker)
            .bind(remote_id)
            .bind(snapshot.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_entry_sources(
        pool: &SqlitePool,
        entry_id: i64,
    ) -> CoreResult<Vec<EntrySource>> {
        let rows = sqlx::query_as::<_, EntrySourceRow>(
            "SELECT tracker, remote_id, synced_at, remote_snapshot FROM ListEntrySource WHERE entry_id = ?",
        )
            .bind(entry_id)
            .fetch_all(pool)
            .await?;
        Ok(rows.into_iter().map(EntrySource::from).collect())
    }

    /// Raw last-synced snapshot for one tracker (used by the KeepLatest merge).
    pub async fn get_entry_source_snapshot(
        pool: &SqlitePool,
        entry_id: i64,
        tracker: &str,
    ) -> CoreResult<Option<serde_json::Value>> {
        let raw: Option<String> = sqlx::query_scalar(
            "SELECT remote_snapshot FROM ListEntrySource WHERE entry_id = ? AND tracker = ?",
        )
            .bind(entry_id)
            .bind(tracker)
            .fetch_optional(pool)
            .await?;
        Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
    }


    /// Startup recovery: interrupted jobs go back to PENDING
    pub async fn recover_sync_jobs(pool: &SqlitePool, retain_days: i64) -> CoreResult<()> {
        sqlx::query("UPDATE OR IGNORE TrackerSyncJob SET status = 'PENDING' WHERE status = 'RUNNING'")
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM TrackerSyncJob WHERE status = 'RUNNING'")
            .execute(pool)
            .await?;
        sqlx::query(
            "DELETE FROM TrackerSyncJob
             WHERE status IN ('DONE', 'SKIPPED')
               AND updated_at < datetime('now', '-' || ? || ' days')",
        )
            .bind(retain_days)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Latest op wins. Re-enqueueing resets attempts/backoff and takes the new batch_id.
    /// Runs on a connection/transaction so a whole set of jobs can be written atomically.
    pub async fn enqueue_sync_job_conn(
        conn: &mut SqliteConnection,
        user_id: i32,
        cid: &str,
        tracker: &str,
        op: &str,
        remote_id: Option<&str>,
        batch_id: Option<&str>,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO TrackerSyncJob
                (user_id, cid, tracker_name, op, status, remote_id, batch_id, next_run_at)
            VALUES (?, ?, ?, ?, 'PENDING', ?, ?, unixepoch())
            ON CONFLICT(user_id, cid, tracker_name) WHERE status = 'PENDING' DO UPDATE SET
                op          = excluded.op,
                remote_id   = COALESCE(excluded.remote_id, remote_id),
                batch_id    = COALESCE(excluded.batch_id, batch_id),
                attempts    = 0,
                next_run_at = unixepoch(),
                last_error  = NULL,
                updated_at  = CURRENT_TIMESTAMP
            "#,
        )
            .bind(user_id)
            .bind(cid)
            .bind(tracker)
            .bind(op)
            .bind(remote_id)
            .bind(batch_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    /// `(status, count)` per status for one batch.
    pub async fn sync_batch_status_counts(
        pool: &SqlitePool,
        user_id: i32,
        batch_id: &str,
    ) -> CoreResult<Vec<(String, i64)>> {
        let rows = sqlx::query_as(
            "SELECT status, COUNT(*) FROM TrackerSyncJob
             WHERE user_id = ? AND batch_id = ? GROUP BY status",
        )
            .bind(user_id)
            .bind(batch_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    /// `(cid, tracker_name, op, status, last_error)` of a batch's FAILED/SKIPPED jobs.
    pub async fn sync_batch_issue_rows(
        pool: &SqlitePool,
        user_id: i32,
        batch_id: &str,
    ) -> CoreResult<Vec<(String, String, String, String, Option<String>)>> {
        let rows = sqlx::query_as(
            "SELECT cid, tracker_name, op, status, last_error FROM TrackerSyncJob
             WHERE user_id = ? AND batch_id = ? AND status IN ('FAILED', 'SKIPPED')
             ORDER BY status, tracker_name, cid
             LIMIT 500",
        )
            .bind(user_id)
            .bind(batch_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    /// `(tracker_name, status, count)` for PENDING/RUNNING/FAILED jobs of a user.
    pub async fn sync_job_counts(
        pool: &SqlitePool,
        user_id: i32,
    ) -> CoreResult<Vec<(String, String, i64)>> {
        let rows = sqlx::query_as(
            "SELECT tracker_name, status, COUNT(*) FROM TrackerSyncJob
             WHERE user_id = ? AND status IN ('PENDING', 'RUNNING', 'FAILED')
             GROUP BY tracker_name, status",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    /// FAILED → PENDING (one batch, or all when `batch_id` is None)
    pub async fn requeue_failed_sync_jobs(
        pool: &SqlitePool,
        user_id: i32,
        batch_id: Option<&str>,
    ) -> CoreResult<usize> {
        let n = sqlx::query(
            "UPDATE OR IGNORE TrackerSyncJob
                SET status = 'PENDING', attempts = 0, next_run_at = unixepoch(),
                    last_error = NULL, updated_at = CURRENT_TIMESTAMP
              WHERE user_id = ? AND status = 'FAILED' AND (? IS NULL OR batch_id = ?)",
        )
            .bind(user_id)
            .bind(batch_id)
            .bind(batch_id)
            .execute(pool)
            .await?
            .rows_affected() as usize;
        Ok(n)
    }

    /// Delete the PENDING jobs of a batch. Running/finished jobs are untouched.
    pub async fn cancel_pending_sync_batch(
        pool: &SqlitePool,
        user_id: i32,
        batch_id: &str,
    ) -> CoreResult<usize> {
        let n = sqlx::query(
            "DELETE FROM TrackerSyncJob WHERE user_id = ? AND batch_id = ? AND status = 'PENDING'",
        )
            .bind(user_id)
            .bind(batch_id)
            .execute(pool)
            .await?
            .rows_affected() as usize;
        Ok(n)
    }

    pub async fn pending_sync_trackers(pool: &SqlitePool, user_id: i32) -> CoreResult<Vec<String>> {
        Ok(sqlx::query_scalar(
            "SELECT DISTINCT tracker_name FROM TrackerSyncJob WHERE user_id = ? AND status = 'PENDING'",
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?)
    }

    /// Atomically claim the next due PENDING job of a tracker (→ RUNNING, attempts + 1).
    /// Interactive edits (no batch_id) are claimed before batch jobs.
    pub async fn claim_next_sync_job(
        pool: &SqlitePool,
        tracker: &str,
        now: i64,
    ) -> CoreResult<Option<SyncJobRow>> {
        let row = sqlx::query_as(
            r#"
            UPDATE TrackerSyncJob
               SET status = 'RUNNING', attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = (
                SELECT id FROM TrackerSyncJob
                 WHERE tracker_name = ? AND status = 'PENDING' AND next_run_at <= ?
                 ORDER BY (batch_id IS NOT NULL) ASC, next_run_at ASC, id ASC
                 LIMIT 1
             )
            RETURNING id, user_id, cid, tracker_name, op, remote_id, attempts, batch_id
            "#,
        )
            .bind(tracker)
            .bind(now)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }

    /// Earliest `next_run_at` among a tracker's PENDING jobs, if any.
    pub async fn next_pending_sync_run_at(pool: &SqlitePool, tracker: &str) -> CoreResult<Option<i64>> {
        let next: Option<i64> = sqlx::query_scalar(
            "SELECT MIN(next_run_at) FROM TrackerSyncJob WHERE tracker_name = ? AND status = 'PENDING'",
        )
            .bind(tracker)
            .fetch_one(pool)
            .await?;
        Ok(next)
    }

    pub async fn complete_sync_job(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        reason: Option<&str>,
    ) -> CoreResult<()> {
        sqlx::query(
            "UPDATE TrackerSyncJob SET status = ?, last_error = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
            .bind(status)
            .bind(reason)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// RUNNING → PENDING for a later run. `burn_attempt = false` refunds the attempt
    /// taken at claim time. If a newer PENDING job exists for the same entry/tracker the
    /// unique index rejects the update; this job is superseded, so it is dropped.
    pub async fn release_sync_job(
        pool: &SqlitePool,
        id: i64,
        next_run_at: i64,
        burn_attempt: bool,
        reason: Option<&str>,
    ) -> CoreResult<()> {
        let refund = if burn_attempt { 0 } else { 1 };
        let n = sqlx::query(
            "UPDATE OR IGNORE TrackerSyncJob
                SET status = 'PENDING', next_run_at = ?, last_error = ?,
                    attempts = MAX(attempts - ?, 0), updated_at = CURRENT_TIMESTAMP
              WHERE id = ?",
        )
            .bind(next_run_at)
            .bind(reason)
            .bind(refund)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if n == 0 {
            sqlx::query("DELETE FROM TrackerSyncJob WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }
}