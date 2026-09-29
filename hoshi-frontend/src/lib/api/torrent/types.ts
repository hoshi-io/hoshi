export interface TorrentSearchResult {
    id: string;
    title: string;
    magnet?: string | null;
    seeders?: number | null;
}

export interface TorrentStreamInfo {
    sessionId: string;
    url: string;
    totalSize: number;
}

export type TorrentState = "initializing" | "live" | "paused" | "error" | "unknown";

export interface TorrentLiveStats {
    state: TorrentState;
    progressBytes: number;
    totalBytes: number;
    /** Bytes per second. */
    downloadBps: number;
    /** Bytes per second. */
    uploadBps: number;
    peers: number;
}

export interface StopTorrentStreamResponse {
    ok: boolean;
    sessionId: string;
}

export type TorrentFilters = Record<string, unknown>;

export type TorrentCacheState = "active" | "seeding" | "cached";

export interface TorrentCacheFile {
    path: string;
    size: number;
}

export interface TorrentCacheEntry {
    key: string;
    name: string;
    sizeBytes: number;
    lastUsedMs: number;
    state: TorrentCacheState;
    files: TorrentCacheFile[];
}

export interface TorrentStorageStats {
    /** Bytes used by all cached torrents. */
    usedBytes: number;
    /** Free space on the volume holding the cache. */
    freeBytes: number;
    totalDiskBytes: number;
    entryCount: number;
    /** Entries currently being streamed. */
    activeCount: number;
    /** Entries kept running after playback (stopSeedingOnPlaybackEnd = false). */
    seedingCount: number;
}

export interface DeleteTorrentCacheEntryResponse {
    freedBytes: number;
}

export interface ClearTorrentCacheResponse {
    removed: number;
    /** Entries left alone (in use, or deletion failed). */
    skipped: number;
    freedBytes: number;
}