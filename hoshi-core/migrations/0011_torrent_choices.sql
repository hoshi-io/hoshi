-- The torrent that last played each episode, per user and source. Packs (is_pack = 1)
-- are also tried for the show's other episodes.
CREATE TABLE IF NOT EXISTS torrent_choices (
    user_id INTEGER NOT NULL,
    cid TEXT NOT NULL,
    extension_id TEXT NOT NULL,
    episode INTEGER NOT NULL,
    torrent_id TEXT NOT NULL,
    title TEXT NOT NULL,
    magnet TEXT NOT NULL,
    -- Same as the torrent cache's folder name, to show what each cached torrent was used for.
    info_hash TEXT NOT NULL,
    is_pack INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, cid, extension_id, episode),
    FOREIGN KEY (user_id) REFERENCES User(id) ON DELETE CASCADE,
    FOREIGN KEY (cid) REFERENCES content(cid) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_torrent_choices_hash ON torrent_choices(user_id, info_hash);
CREATE INDEX IF NOT EXISTS idx_torrent_choices_pack ON torrent_choices(user_id, cid, extension_id, is_pack, updated_at);
