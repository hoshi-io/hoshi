/**
 * Frontend mirrors of the Rust torrent types
 * (`hoshi_core::torrent::types` and `hoshi_core::extensions::types`).
 * Field names are camelCase because the Rust structs use `rename_all = "camelCase"`.
 */

/**
 * A search result returned by a torrent extension.
 *
 * Only the fields the frontend reads today are listed. Copy the remaining
 * fields from Rust's `TorrentSearchResult` when a component needs them.
 */
export interface TorrentSearchResult {
    id: string;
    title: string;
    /** Direct magnet link, when the extension provides one. Otherwise resolve it with `getMagnet`. */
    magnet?: string | null;
    seeders?: number | null;
}

/** Returned by `start_torrent_stream`. `url` is the `torrent://<sessionId>` URL handed to mpv. */
export interface TorrentStreamInfo {
    sessionId: string;
    url: string;
    totalSize: number;
}

/** librqbit's torrent state; `unknown` is the backend's fallback. */
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

/** Extension-specific search filters, forwarded to the extension untouched. */
export type TorrentFilters = Record<string, unknown>;