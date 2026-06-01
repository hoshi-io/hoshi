CREATE TABLE IF NOT EXISTS ListEntry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('CURRENT', 'PLANNING', 'COMPLETED', 'PAUSED', 'DROPPED', 'REPEATING')),
    progress INTEGER NOT NULL DEFAULT 0,
    score REAL,
    start_date DATE,
    end_date DATE,
    repeat_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    is_private INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, cid),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_list_user ON ListEntry(user_id);
CREATE INDEX IF NOT EXISTS idx_list_cid ON ListEntry(cid);
CREATE INDEX IF NOT EXISTS idx_list_status ON ListEntry(status);

-- Which remotes know about each entry, and their sync state
CREATE TABLE IF NOT EXISTS ListEntrySource (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    tracker TEXT NOT NULL,              -- 'anilist', 'myanimelist', etc.
    remote_id TEXT NOT NULL,            -- the entry's ID on that tracker
    remote_updated_at DATETIME,         -- when the remote last reported a change
    synced_at DATETIME,                 -- when you last successfully pushed/pulled
    remote_snapshot TEXT,
    UNIQUE (entry_id, tracker),
    FOREIGN KEY (entry_id) REFERENCES ListEntry(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_source_entry ON ListEntrySource(entry_id);
CREATE INDEX IF NOT EXISTS idx_source_user_tracker ON ListEntrySource(user_id, tracker);

-- Field-level history of every change ever made to an entry
CREATE TABLE IF NOT EXISTS ListEntryChange (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    source TEXT NOT NULL CHECK(source IN ('LOCAL', 'REMOTE_SYNC', 'IMPORT')),
    tracker TEXT,
    field TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    FOREIGN KEY (entry_id) REFERENCES ListEntry(id) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_change_entry ON ListEntryChange(entry_id, changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_change_user ON ListEntryChange(user_id, changed_at DESC);

CREATE TABLE IF NOT EXISTS AnimeProgress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    episode INTEGER NOT NULL,
    timestamp_seconds INTEGER NOT NULL DEFAULT 0,
    episode_duration_seconds INTEGER,
    completed INTEGER NOT NULL DEFAULT 0,
    last_accessed INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(user_id, cid, episode),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_anime_progress_user_accessed ON AnimeProgress(user_id, last_accessed DESC);
CREATE INDEX IF NOT EXISTS idx_anime_progress_user_cid ON AnimeProgress(user_id, cid);

CREATE TABLE IF NOT EXISTS ChapterProgress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    chapter INTEGER NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    last_accessed INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(user_id, cid, chapter),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_chapter_progress_user_accessed ON ChapterProgress(user_id, last_accessed DESC);
CREATE INDEX IF NOT EXISTS idx_chapter_progress_user_cid ON ChapterProgress(user_id, cid);