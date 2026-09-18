CREATE TABLE IF NOT EXISTS cache_metadata (
    key TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    query_type TEXT NOT NULL,
    data TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_source ON cache_metadata(source);
CREATE INDEX IF NOT EXISTS idx_cache_type ON cache_metadata(query_type);
CREATE INDEX IF NOT EXISTS idx_cache_expires ON cache_metadata(expires_at);