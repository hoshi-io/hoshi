-- Was added to 0004 after v1.2.1 shipped; moved here so released migrations keep their checksums.
CREATE INDEX IF NOT EXISTS idx_rel_target_tracker ON content_relations(target_tracker_name, target_tracker_id);
