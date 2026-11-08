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
