CREATE TABLE IF NOT EXISTS content (
    cid TEXT PRIMARY KEY,
    type TEXT NOT NULL CHECK(type IN ('anime', 'manga', 'novel')),
    nsfw INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL,
    source_name TEXT NOT NULL,
    source_id TEXT,
    subtype TEXT,
    title TEXT NOT NULL,
    alt_titles TEXT NOT NULL DEFAULT '[]',
    title_i18n TEXT NOT NULL DEFAULT '{}',
    synopsis TEXT,
    cover_image TEXT,
    banner_image TEXT,
    eps_or_chapters TEXT,
    status TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    genres TEXT NOT NULL DEFAULT '[]',
    release_date TEXT,
    end_date TEXT,
    rating REAL,
    trailer_url TEXT,
    characters TEXT NOT NULL DEFAULT '[]',
    studio TEXT,
    staff TEXT NOT NULL DEFAULT '[]',
    external_ids TEXT NOT NULL DEFAULT '{}',
    episode_duration INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(cid, source_name),
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_metadata_cid ON metadata(cid);
CREATE INDEX IF NOT EXISTS idx_metadata_source ON metadata(source_name);
CREATE INDEX IF NOT EXISTS idx_metadata_title ON metadata(title);
CREATE INDEX IF NOT EXISTS idx_metadata_title_japanese ON metadata(json_extract(title_i18n, '$.japanese'));
CREATE INDEX IF NOT EXISTS idx_metadata_title_romaji ON metadata(json_extract(title_i18n, '$.romaji'));
CREATE INDEX IF NOT EXISTS idx_metadata_title_english ON metadata(json_extract(title_i18n, '$.english'));