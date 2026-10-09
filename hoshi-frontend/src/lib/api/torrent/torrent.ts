import { call } from "@/api/client";
import type {
    ClearTorrentCacheResponse,
    DeleteTorrentCacheEntryResponse,
    StopTorrentStreamResponse,
    TorrentCacheEntry,
    TorrentFilters,
    TorrentLiveStats,
    TorrentSearchResult,
    RankedTorrent,
    TorrentTargetSummary,
    TorrentStorageStats,
    TorrentStreamInfo,
} from "./types";

export const torrentApi = {
    search(id: string, query: string, filters: TorrentFilters = {}, page = 1): Promise<TorrentSearchResult[]> {
        return call<{ results: TorrentSearchResult[] }>({
            tauri: { cmd: "search_torrents", args: { id, query, filters, page } },
        }).then(res => res.results ?? []);
    },

    getMagnet(id: string, contentId: string): Promise<string> {
        return call<{ magnet: string }>({
            tauri: { cmd: "get_magnet", args: { id, contentId } },
        }).then(res => res.magnet);
    },

    /**
     * Best match for an episode of `cid` according to the user's torrent settings,
     * or `null` if nothing fits. The backend derives title, season and absolute
     * episode number from the content's metadata and relations.
     */
    autoSelect(id: string, cid: string, episode: number, filters: TorrentFilters = {}, page = 1): Promise<TorrentSearchResult | null> {
        return call<{ torrent: TorrentSearchResult | null }>({
            tauri: { cmd: "auto_select_torrent", args: { id, cid, episode, filters, page } },
        }).then(res => res.torrent ?? null);
    },

    /** Adds the torrent and opens a stream session. Pass `magnet` to skip the extension lookup. */
    startStream(id: string, contentId: string, cid: string, episode: number, magnet?: string | null): Promise<TorrentStreamInfo> {
        return call<TorrentStreamInfo>({
            tauri: { cmd: "start_torrent_stream", args: { id, contentId, cid: cid, episode: episode, magnet: magnet ?? null } },
        });
    },

    stopStream(sessionId: string): Promise<StopTorrentStreamResponse> {
        return call<StopTorrentStreamResponse>({
            tauri: { cmd: "stop_torrent_stream", args: { sessionId } },
        });
    },

    /** `null` when the backend has no live session with this id. */
    getStats(sessionId: string): Promise<TorrentLiveStats | null> {
        return call<TorrentLiveStats | null>({
            tauri: { cmd: "get_torrent_stats", args: { sessionId } },
        });
    },

    // ------------------------------------------------------------ cache / storage

    /** Disk usage of the torrent cache plus free space on its volume. */
    getStorageStats(): Promise<TorrentStorageStats> {
        return call<TorrentStorageStats>({
            tauri: { cmd: "get_torrent_storage_stats", args: {} },
        });
    },

    /** Every cached torrent with its files, state (active / seeding / cached) and last-used time. */
    listCache(): Promise<TorrentCacheEntry[]> {
        return call<TorrentCacheEntry[]>({
            tauri: { cmd: "list_torrent_cache", args: {} },
        });
    },

    /** Deletes one cached torrent by `TorrentCacheEntry.key`. Fails if it is currently being streamed. */
    deleteCacheEntry(key: string): Promise<DeleteTorrentCacheEntryResponse> {
        return call<DeleteTorrentCacheEntryResponse>({
            tauri: { cmd: "delete_torrent_cache_entry", args: { key } },
        });
    },

    /** Deletes everything that isn't currently being streamed. */
    clearCache(): Promise<ClearTorrentCacheResponse> {
        return call<ClearTorrentCacheResponse>({
            tauri: { cmd: "clear_torrent_cache", args: {} },
        });
    },

    /** Absolute path of the cache folder (for "open in file manager"). */
    getCacheDir(): Promise<string> {
        return call<{ path: string }>({
            tauri: { cmd: "get_torrent_cache_dir", args: {} },
        }).then(res => res.path);
    },
};