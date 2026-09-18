CREATE TABLE IF NOT EXISTS tracker_mappings (
    cid TEXT NOT NULL,
    tracker_name TEXT NOT NULL,
    tracker_id TEXT NOT NULL,
    tracker_url TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (cid, tracker_name),
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tracker_name ON tracker_mappings(tracker_name);
CREATE INDEX IF NOT EXISTS idx_tracker_id ON tracker_mappings(tracker_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracker_name_id ON tracker_mappings(tracker_name, tracker_id);

CREATE TABLE IF NOT EXISTS extension_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    extension_name TEXT NOT NULL,
    extension_id TEXT NOT NULL,
    nsfw INTEGER NOT NULL DEFAULT 0,
    language TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(extension_name, extension_id),
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ext_cid ON extension_sources(cid);
CREATE INDEX IF NOT EXISTS idx_ext_name ON extension_sources(extension_name);
CREATE INDEX IF NOT EXISTS idx_ext_nsfw ON extension_sources(nsfw);