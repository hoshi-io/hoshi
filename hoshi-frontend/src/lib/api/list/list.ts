import { call } from "@/api/client";
import type {
    ListResponse,
    SingleEntryResponse,
    UpsertEntryResponse,
    UpsertEntryBody,
    FilterQuery,
    UserStats, EntryHistoryResponse, BatchWriteResponse, BatchHandle, BatchStatus, SyncStatus, BatchIssue,
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

    batchUpsert(entries: UpsertEntryBody[]) {
        return call<BatchWriteResponse>({
            tauri: { cmd: "batch_upsert_entries", args: { entries } },
        });
    },

    batchDelete(cids: string[]) {
        return call<BatchWriteResponse>({
            tauri: { cmd: "batch_delete_entries", args: { cids } },
        });
    },

    /** Push entries to one specific tracker (the "on X but not Y" button). */
    pushToTracker(tracker: string, cids: string[]) {
        return call<BatchHandle>({
            tauri: { cmd: "push_to_tracker", args: { tracker, cids } },
        });
    },

    getBatchStatus(batchId: string) {
        return call<BatchStatus>({
            tauri: { cmd: "get_batch_status", args: { batchId } },
        });
    },

    getBatchIssues(batchId: string) {
        return call<BatchIssue[]>({
            tauri: { cmd: "get_batch_issues", args: { batchId } },
        });
    },

    resumeSyncLane(tracker: string) {
        return call<void>({
            tauri: { cmd: "resume_sync_lane", args: { tracker } },
        });
    },

    getSyncStatus() {
        return call<SyncStatus>({
            tauri: { cmd: "get_sync_status", args: {} },
        });
    },

    /** Omit batchId to retry every failed job. Resolves to the number of jobs re-queued. */
    retryFailedSyncs(batchId?: string) {
        return call<number>({
            tauri: { cmd: "retry_failed_syncs", args: { batchId: batchId ?? null } },
        });
    },

    /** Drops still-pending jobs of the batch. Resolves to the number removed. */
    cancelBatch(batchId: string) {
        return call<number>({
            tauri: { cmd: "cancel_batch", args: { batchId } },
        });
    },
};