CREATE TABLE IF NOT EXISTS User (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    avatar_data BLOB,
    avatar_mime TEXT,
    password_hash TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS auth_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    active_user_id INTEGER,
    FOREIGN KEY (active_user_id) REFERENCES User(id) ON DELETE SET NULL
);
INSERT OR IGNORE INTO auth_state (id, active_user_id) VALUES (1, NULL);

CREATE TABLE IF NOT EXISTS UserConfig (
    user_id INTEGER PRIMARY KEY,
    config TEXT NOT NULL DEFAULT '{}',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS UserIntegration (
    user_id INTEGER NOT NULL,
    tracker_name TEXT NOT NULL,
    tracker_user_id TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    expires_at DATETIME NOT NULL,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    display_name TEXT,
    avatar_url TEXT,
    profile_url TEXT,
    total_entries INTEGER,
    last_synced_at DATETIME,
    score_format TEXT,
    sync_interval INTEGER NOT NULL DEFAULT 3600,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, tracker_name),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE
);