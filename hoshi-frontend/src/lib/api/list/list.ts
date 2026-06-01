import { call } from "@/api/client";
import type {
    ListResponse,
    SingleEntryResponse,
    UpsertEntryResponse,
    UpsertEntryBody,
    FilterQuery,
    UserStats, EntryHistoryResponse,
} from "./types";

export const listApi = {
    getList(query?: FilterQuery) {
        return call<ListResponse>({
            http:  { path: "list", method: "GET", params: query as Record<string, unknown> },
            tauri: { cmd: "get_list", args: { query: query ?? {} } },
        });
    },

    getStats() {
        return call<UserStats>({
            tauri: { cmd: "get_stats", args: {} },
        });
    },

    getEntry(cid: string) {
        return call<SingleEntryResponse>({
            tauri: { cmd: "get_single_entry", args: { cid } },
        });
    },

    upsert(body: UpsertEntryBody) {
        return call<UpsertEntryResponse>({
            tauri: { cmd: "upsert_entry", args: { body } },
        });
    },

    delete(cid: string) {
        return call<void>({
            tauri: { cmd: "delete_entry", args: { cid } },
        });
    },

    getEntryHistory(cid: string) {
        return call<EntryHistoryResponse>({
            tauri: { cmd: "get_entry_history", args: { cid } },
        });
    },

    getActivityFeed(limit?: number) {
        return call<EntryHistoryResponse>({
            tauri: { cmd: "get_activity_feed", args: { limit: limit ?? 50 } },
        });
    },
};