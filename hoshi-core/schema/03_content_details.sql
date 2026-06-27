CREATE TABLE IF NOT EXISTS content_units (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    unit_number REAL NOT NULL,
    type TEXT NOT NULL,
    title TEXT,
    description TEXT,
    thumbnail_url TEXT,
    released_at TEXT,
    duration INTEGER,
    absolute_number INTEGER,
    created_at INTEGER NOT NULL,
    UNIQUE(cid, type, unit_number),
    FOREIGN KEY(cid) REFERENCES content(cid) ON DELETE CASCADE
);

ALTER TABLE content_units ADD COLUMN updated_at INTEGER;

CREATE INDEX IF NOT EXISTS idx_units_cid ON content_units(cid);
CREATE INDEX IF NOT EXISTS idx_units_type ON content_units(type);

CREATE TABLE IF NOT EXISTS content_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_cid TEXT NOT NULL,
    target_cid TEXT NOT NULL,
    relation_type TEXT NOT NULL CHECK(relation_type IN (
        'sequel', 'prequel', 'side_story', 'spinoff',
        'adaptation', 'alternative', 'parent', 'summary'
    )),
    source_name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(source_cid, target_cid, relation_type),
    FOREIGN KEY (source_cid) REFERENCES content(cid) ON DELETE CASCADE,
    FOREIGN KEY (target_cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_rel_source ON content_relations(source_cid);
CREATE INDEX IF NOT EXISTS idx_rel_target ON content_relations(target_cid);

CREATE TABLE IF NOT EXISTS pending_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_cid TEXT NOT NULL,
    target_tracker_name TEXT NOT NULL,
    target_tracker_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(source_cid, target_tracker_name, target_tracker_id, relation_type),
    FOREIGN KEY (source_cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_pending_rel_source ON pending_relations(source_cid);