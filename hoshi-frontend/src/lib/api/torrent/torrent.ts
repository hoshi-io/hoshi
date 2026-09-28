import { call } from "@/api/client";
import type {
    StopTorrentStreamResponse,
    TorrentFilters,
    TorrentLiveStats,
    TorrentSearchResult,
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

    /** Best match for `query` according to the user's torrent settings, or `null` if nothing fits. */
    autoSelect(id: string, query: string, filters: TorrentFilters = {}, page = 1): Promise<TorrentSearchResult | null> {
        return call<{ torrent: TorrentSearchResult | null }>({
            tauri: { cmd: "auto_select_torrent", args: { id, query, filters, page } },
        }).then(res => res.torrent ?? null);
    },

    /** Adds the torrent and opens a stream session. Pass `magnet` to skip the extension lookup. */
    startStream(id: string, contentId: string, magnet?: string | null): Promise<TorrentStreamInfo> {
        return call<TorrentStreamInfo>({
            tauri: { cmd: "start_torrent_stream", args: { id, contentId, magnet: magnet ?? null } },
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
};