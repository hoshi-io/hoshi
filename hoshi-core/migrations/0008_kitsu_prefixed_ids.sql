-- TrackerSyncJob was added to 0006 in place after v1.2.1 shipped, so databases from
-- v1.2.0/v1.2.1 don't have it; create it here, before the updates below use it.
CREATE TABLE IF NOT EXISTS TrackerSyncJob (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    tracker_name TEXT NOT NULL,
    op TEXT NOT NULL CHECK(op IN ('UPSERT', 'DELETE')),
    status TEXT NOT NULL DEFAULT 'PENDING'
    CHECK(status IN ('PENDING', 'RUNNING', 'DONE', 'FAILED', 'SKIPPED')),
    attempts INTEGER NOT NULL DEFAULT 0,
    next_run_at INTEGER NOT NULL DEFAULT (unixepoch()),   -- unix seconds
    last_error TEXT,                -- failure reason, or skip reason for SKIPPED
    remote_id TEXT,                 -- captured at enqueue time for DELETE (local row/mapping may be gone)
    batch_id TEXT,                  -- groups jobs for progress UI
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sync_job_pending_unique ON TrackerSyncJob(user_id, cid, tracker_name) WHERE status = 'PENDING';
CREATE INDEX IF NOT EXISTS idx_sync_job_claim ON TrackerSyncJob(tracker_name, status, next_run_at);
CREATE INDEX IF NOT EXISTS idx_sync_job_batch ON TrackerSyncJob(batch_id);
CREATE INDEX IF NOT EXISTS idx_sync_job_user ON TrackerSyncJob(user_id, status);

-- Kitsu anime and manga ids share one number space, so Kitsu ids are now stored
-- with a type prefix ("anime:123" / "manga:123"), the same scheme as MAL.
-- Rewrite existing unprefixed Kitsu ids using the type of the content they belong to.
-- Novels live under /manga on Kitsu.
-- OR IGNORE: a row whose prefixed form already exists is left as is instead of
-- failing the migration (and with it, app startup).

UPDATE OR IGNORE tracker_mappings
   SET tracker_id = (SELECT CASE c.type WHEN 'anime' THEN 'anime:' ELSE 'manga:' END
                       FROM content c WHERE c.cid = tracker_mappings.cid) || tracker_id
 WHERE tracker_name = 'kitsu'
   AND instr(tracker_id, ':') = 0
   AND EXISTS (SELECT 1 FROM content c WHERE c.cid = tracker_mappings.cid);

UPDATE metadata
   SET source_id = (SELECT CASE c.type WHEN 'anime' THEN 'anime:' ELSE 'manga:' END
                      FROM content c WHERE c.cid = metadata.cid) || source_id
 WHERE source_name = 'kitsu'
   AND source_id IS NOT NULL
   AND instr(source_id, ':') = 0;

UPDATE ListEntrySource
   SET remote_id = (SELECT CASE c.type WHEN 'anime' THEN 'anime:' ELSE 'manga:' END
                      FROM ListEntry le JOIN content c ON c.cid = le.cid
                     WHERE le.id = ListEntrySource.entry_id) || remote_id
 WHERE tracker = 'kitsu'
   AND instr(remote_id, ':') = 0
   AND EXISTS (SELECT 1 FROM ListEntry le JOIN content c ON c.cid = le.cid
                WHERE le.id = ListEntrySource.entry_id);

-- Pending DELETE jobs carry the remote id captured at enqueue time.
UPDATE TrackerSyncJob
   SET remote_id = (SELECT CASE c.type WHEN 'anime' THEN 'anime:' ELSE 'manga:' END
                      FROM content c WHERE c.cid = TrackerSyncJob.cid) || remote_id
 WHERE tracker_name = 'kitsu'
   AND remote_id IS NOT NULL
   AND instr(remote_id, ':') = 0
   AND EXISTS (SELECT 1 FROM content c WHERE c.cid = TrackerSyncJob.cid);

-- Relations: only rows whose target is already imported have a known type.
-- Unresolved ones keep the legacy unprefixed id (still read as anime, as before).
UPDATE OR IGNORE content_relations
   SET target_tracker_id = (SELECT CASE c.type WHEN 'anime' THEN 'anime:' ELSE 'manga:' END
                              FROM content c WHERE c.cid = content_relations.target_cid) || target_tracker_id
 WHERE target_tracker_name = 'kitsu'
   AND target_cid IS NOT NULL
   AND instr(target_tracker_id, ':') = 0
   AND EXISTS (SELECT 1 FROM content c WHERE c.cid = content_relations.target_cid);

-- Note: Kitsu relations imported before this version were stored with
-- relation_type 'alternative' (the lowercase Kitsu roles weren't recognized).
-- The original role isn't recoverable from the DB, so those rows are left as is.
