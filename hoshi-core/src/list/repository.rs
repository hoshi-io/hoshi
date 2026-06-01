use sqlx::SqlitePool;
use crate::error::CoreResult;
use crate::list::types::{EntrySource, EntrySourceRow, ListEntry, ListEntryChange, ScoreDistribution, UpsertEntryBody, UserStats};

pub struct ListRepository;

impl ListRepository {
    pub async fn get_entry(pool: &SqlitePool, user_id: i32, cid: &str) -> CoreResult<Option<ListEntry>> {
        let row = sqlx::query_as::<_, ListEntry>(
            "SELECT * FROM ListEntry WHERE user_id = ? AND cid = ?",
        )
            .bind(user_id)
            .bind(cid)
            .fetch_optional(pool)
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
            .execute(pool)
            .await?
            .rows_affected() as usize;
        Ok(rows)
    }

    pub async fn delete_entry(pool: &SqlitePool, user_id: i32, cid: &str) -> CoreResult<bool> {
        let rows = sqlx::query(
            "DELETE FROM ListEntry WHERE user_id = ? AND cid = ?",
        )
            .bind(user_id)
            .bind(cid)
            .execute(pool)
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
                .execute(pool)
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
}