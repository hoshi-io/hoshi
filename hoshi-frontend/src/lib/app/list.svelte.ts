import { SvelteSet } from "svelte/reactivity";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { listApi } from "@/api/list/list";
import type {BatchStatus, EnrichedListEntry, ListStatus, SyncStatus, UpsertEntryBody, UserStats} from "@/api/list/types";
import type { CoreError } from "@/api/client";
import {type NormalizedCard, normalizeListEntry} from "@/utils/normalize";
import { appConfig } from "@/stores/config.svelte.js";
import type {TrackerInfo} from "@/api/tracker/types";
import {integrationsApi} from "@/api/tracker/tracker";

export type SortOption = "SCORE_DESC" | "TITLE_ASC" | "TITLE_DESC" | "PROGRESS_DESC" | "PROGRESS_ASC";
export type StatusFilter = "ALL" | "CURRENT" | "COMPLETED" | "PLANNING" | "PAUSED" | "DROPPED";
export type TypeFilter = "ALL" | "anime" | "manga" | "novel";
export type MissingTrackerFilter = "ALL" | "anilist" | "mal" | "kitsu" | "simkl";

const TRACKER_LABELS: Record<string, string> = {
    anilist: "AniList",
    mal: "MyAnimeList",
    kitsu: "Kitsu",
    simkl: "Simkl",
};

export const trackerLabel = (tracker: string) => TRACKER_LABELS[tracker] ?? tracker;

/** What started a batch, so the sync panel can label it. */
export type BatchMeta = {
    kind: "edit" | "delete" | "push";
    /** Target tracker for "push". */
    tracker?: string;
};

export type NormalizedListEntry = {
    card: NormalizedCard;
    original: EnrichedListEntry;
};

class ListStore {
    entries = $state<EnrichedListEntry[]>([]);
    stats = $state<UserStats | null>(null);
    isLoading = $state(false);
    error = $state<CoreError | null>(null);
    isInitialized = $state(false);

    showConflicts = $state(false);
    missingOn = $state<MissingTrackerFilter>("ALL");

    activeStatus = $state<StatusFilter>("ALL");
    activeType = $state<TypeFilter>("ALL");
    searchQuery = $state("");
    activeSort = $state<SortOption>("SCORE_DESC");
    isMobileSearchActive = $state(false);
    isDrawerOpen = $state(false);
    selectedEntry = $state<EnrichedListEntry | null>(null);
    isModalOpen = $state(false);
    connectedTrackers = $state<string[]>([]);

    normalized = $state<NormalizedListEntry[]>([]);

    syncStatus = $state<SyncStatus | null>(null);
    batches = $state<Record<string, BatchStatus>>({});
    batchMeta = $state<Record<string, BatchMeta>>({});
    private syncInit: Promise<void> | null = null;
    private unlistenSync: UnlistenFn | null = null;
    private pollTimer: ReturnType<typeof setTimeout> | null = null;
    private reloadTimer: ReturnType<typeof setTimeout> | null = null;
    private syncRefreshing = false;
    private syncDirty = false;

    pendingSyncCount = $derived(
        (this.syncStatus?.trackers ?? []).reduce((n, t) => n + t.pending + t.running, 0)
    );
    failedSyncCount = $derived(
        (this.syncStatus?.trackers ?? []).reduce((n, t) => n + t.failed, 0)
    );
    /** A pause only matters while that tracker still has jobs waiting on it. */
    syncPaused = $derived(
        (this.syncStatus?.trackers ?? []).some(t => t.pausedReason && t.pending + t.running + t.failed > 0)
    );
    /** The sync UI exists only while there is something to show: work in flight, a pause, failures, or a tracked batch. */
    syncVisible = $derived(
        this.pendingSyncCount > 0 ||
        this.failedSyncCount > 0 ||
        this.syncPaused ||
        Object.keys(this.batches).length > 0
    );
    syncPhase = $derived<"syncing" | "paused" | "failed" | "done">(
        this.syncPaused ? "paused"
            : this.pendingSyncCount > 0 ? "syncing"
                : this.failedSyncCount > 0 ? "failed"
                    : "done"
    );
    /** Progress across unfinished batches; null when only un-batched edits are in flight. */
    syncBatchProgress = $derived.by(() => {
        let total = 0;
        let done = 0;
        for (const b of Object.values(this.batches)) {
            if (b.finished) continue;
            total += b.total;
            done += b.done + b.skipped + b.failed;
        }
        return total > 0 ? { done, total } : null;
    });

    filtered = $derived(
        this.normalized.filter(item => {
            const titleLang = appConfig.data?.ui?.titleLanguage || "romaji";
            const displayTitle = (item.original.titleI18n?.[titleLang] || item.original.title || "").toLowerCase();
            const baseTitle = (item.original.title || "").toLowerCase();
            const searchString = displayTitle + " " + baseTitle;

            const matchesStatus = this.activeStatus === "ALL" || item.original.status === this.activeStatus;
            const matchesType = this.activeType === "ALL" || item.original.contentType === this.activeType;
            const matchesSearch = this.searchQuery === "" || searchString.includes(this.searchQuery.toLowerCase());

            let matchesConflicts = true;
            if (this.showConflicts) {
                const sources = item.original.sources || [];

                if (sources.length <= 1) {
                    matchesConflicts = false;
                } else {
                    const first = sources[0] as any;

                    matchesConflicts = sources.some((s: any) => {
                        if (s.tracker === first.tracker) return false;

                        const sStatus = String(s.status || "").toUpperCase().trim();
                        const fStatus = String(first.status || "").toUpperCase().trim();

                        const statusConflict = (sStatus !== "" && fStatus !== "") && (sStatus !== fStatus);

                        const sProg = Number(s.progress) || 0;
                        const fProg = Number(first.progress) || 0;
                        const progressConflict = sProg !== fProg;

                        return statusConflict || progressConflict;
                    });
                }
            }

            let matchesMissingTracker = true;

            if (this.missingOn !== "ALL") {
                if (
                    this.missingOn === "simkl" &&
                    (item.card.contentType.toUpperCase() === "MANGA" ||
                        item.card.contentType.toUpperCase() === "NOVEL")
                ) {
                    matchesMissingTracker = false;
                } else {
                    const sources = item.original.sources || [];
                    matchesMissingTracker = !sources.some(
                        s => s.tracker === this.missingOn
                    );
                }
            }

            return matchesStatus && matchesType && matchesSearch && matchesConflicts && matchesMissingTracker;
        })
    );

    sorted = $derived(
        [...this.filtered].sort((a, b) => {
            const titleA = a.original.title || "";
            const titleB = b.original.title || "";
            switch (this.activeSort) {
                case "TITLE_ASC": return titleA.localeCompare(titleB);
                case "TITLE_DESC": return titleB.localeCompare(titleA);
                case "PROGRESS_DESC": return (b.original.progress || 0) - (a.original.progress || 0);
                case "PROGRESS_ASC": return (a.original.progress || 0) - (b.original.progress || 0);
                case "SCORE_DESC": return (b.original.score || 0) - (a.original.score || 0);
                default: return 0;
            }
        })
    );

    selectionMode = $state(false);
    selected = new SvelteSet<string>();

    /**
     * Everything selected, whether or not the current filters show it. The selection is the
     * user's explicit choice, so it survives filter changes and batch actions apply to all of it.
     * (Filtered against `entries` so ids of entries deleted elsewhere don't linger.)
     */
    selectedCids = $derived(
        this.entries.filter(e => this.selected.has(e.cid)).map(e => e.cid)
    );
    allVisibleSelected = $derived(
        this.sorted.length > 0 && this.sorted.every(i => this.selected.has(i.original.cid))
    );

    enterSelection(preselect: string[] = []) {
        this.selected.clear();
        for (const cid of preselect) this.selected.add(cid);
        this.selectionMode = true;
    }

    exitSelection() {
        this.selectionMode = false;
        this.selected.clear();
    }

    toggleSelect(cid: string) {
        if (this.selected.has(cid)) this.selected.delete(cid);
        else this.selected.add(cid);
    }

    /** Everything the filters match, not just the cards rendered so far. */
    selectAllVisible() {
        for (const item of this.sorted) this.selected.add(item.original.cid);
    }

    clearSelection() {
        this.selected.clear();
    }

    /** Shortcut for the "missing on tracker" filter: selection mode with everything that matches. */
    startPushSelection() {
        if (this.missingOn === "ALL") return;
        this.enterSelection(this.sorted.map(i => i.original.cid));
    }

    async pushSelected() {
        const tracker = this.missingOn;
        const cids = this.selectedCids;
        if (tracker === "ALL" || cids.length === 0) return;
        await this.pushToTracker(tracker, cids);
        this.exitSelection();
    }

    /** Bump progress by one for every selected entry (one batch, so it syncs coalesced). */
    async incrementSelected() {
        const bodies: UpsertEntryBody[] = this.entries
            .filter(e => this.selected.has(e.cid))
            .map(e => ({
                cid: e.cid,
                status: e.status,
                progress: (e.progress || 0) + 1,
            }));
        if (bodies.length === 0) return;
        await this.batchUpsert(bodies);
        this.exitSelection();
    }

    async deleteSelected() {
        const cids = this.selectedCids;
        if (cids.length === 0) return;
        await this.batchRemove(cids);
        this.exitSelection();
    }

    updateConnectedTrackers(trackers: TrackerInfo[]) {
        this.connectedTrackers = trackers
            .filter(t => t.connected)
            .map(t => t.name.toLowerCase());
    }

    /** `silent` reloads without flipping `isLoading` (no skeleton/spinner flash). */
    async loadData(forceRefresh = false, silent = false) {
        if (this.isInitialized && !forceRefresh) return;
        listStore.updateConnectedTrackers(await integrationsApi.getAll());

        if (!silent) this.isLoading = true;
        this.error = null;

        try {
            const [listRes, statsRes] = await Promise.all([
                listApi.getList({}),
                listApi.getStats()
            ]);

            this.entries = listRes.results;
            this.normalized = listRes.results.map(entry => ({
                card: normalizeListEntry(entry),
                original: entry,
            }));
            this.stats = statsRes;
            this.isInitialized = true;
        } catch (err) {
            console.error("Failed to load collection data:", err);
            this.error = err as CoreError;
        } finally {
            this.isLoading = false;
        }
    }

    upsertLocal(body: UpsertEntryBody, result: EnrichedListEntry) {
        const idx = this.entries.findIndex(e => e.cid === body.cid);
        if (idx !== -1) {
            this.entries[idx] = result;
            this.normalized[idx] = { card: normalizeListEntry(result), original: result };
        } else {
            this.entries = [result, ...this.entries];
            this.normalized = [{ card: normalizeListEntry(result), original: result }, ...this.normalized];
        }
    }

    deleteLocal(cid: string) {
        this.entries = this.entries.filter(e => e.cid !== cid);
        this.normalized = this.normalized.filter(e => e.original.cid !== cid);
    }

    async refresh() {
        await this.loadData(true);
    }

    hasCid(cid: string): boolean {
        return this.entries.some(e => e.cid === cid);
    }

    async incrementProgress(entry: EnrichedListEntry) {
        const nextProgress = (entry.progress || 0) + 1;
        const body: UpsertEntryBody = {
            cid: entry.cid,
            status: entry.status,
            progress: nextProgress,
        };

        await listApi.upsert(body);
        this.kickSync();
        const res = await listApi.getEntry(entry.cid);
        if (res.found && res.entry) {
            this.upsertLocal(body, res.entry);
        }
    }

    async removeEntry(cid: string) {
        await listApi.delete(cid);
        this.deleteLocal(cid);
        this.kickSync();
    }

    initSync(): Promise<void> {
        return (this.syncInit ??= this.startSync());
    }

    destroySync() {
        this.unlistenSync?.();
        this.unlistenSync = null;
        this.syncInit = null;
        if (this.pollTimer) clearTimeout(this.pollTimer);
        if (this.reloadTimer) clearTimeout(this.reloadTimer);
        this.pollTimer = null;
        this.reloadTimer = null;
    }

    private async startSync() {
        await this.refreshSyncStatus();
        this.schedulePoll();
        try {
            this.unlistenSync = await listen("sync-progress", () => {
                void this.onSyncEvent();
            });
        } catch (err) {
            console.warn("sync-progress listener unavailable:", err);
        }
    }

    kickSync() {
        void this.onSyncEvent();
    }

    private async onSyncEvent() {
        if (this.syncRefreshing) {
            this.syncDirty = true;
            return;
        }
        this.syncRefreshing = true;
        try {
            do {
                this.syncDirty = false;
                await Promise.all([this.refreshSyncStatus(), this.refreshBatches()]);
            } while (this.syncDirty);
        } finally {
            this.syncRefreshing = false;
        }
        this.schedulePoll();
    }

    /**
     * Safety net for missed events: the backend forwards them throttled and says nothing when a
     * pause expires, so the last update (the one that says "done") can get lost. While there is
     * work, re-ask on a timer; it stops by itself once the queue is empty.
     */
    private schedulePoll() {
        if (this.pollTimer) return;
        const openBatch = Object.values(this.batches).some(b => !b.finished);
        if (this.pendingSyncCount === 0 && !openBatch) return;

        const live = (this.syncStatus?.trackers ?? []).some(t => t.pending + t.running > 0 && !t.pausedReason);
        this.pollTimer = setTimeout(() => {
            this.pollTimer = null;
            void this.onSyncEvent();
        }, live ? 1500 : 5000);
    }

    /** Coalesced silent reload, so several "finished" signals in a row cost one fetch. */
    private scheduleReload() {
        if (this.reloadTimer) return;
        this.reloadTimer = setTimeout(() => {
            this.reloadTimer = null;
            void this.loadData(true, true);
        }, 200);
    }

    async refreshSyncStatus() {
        const hadPending = this.pendingSyncCount > 0;
        try {
            this.syncStatus = await listApi.getSyncStatus();
        } catch (err) {
            console.error("Failed to load sync status:", err);
            return;
        }
        // Pushes record which trackers know an entry (list `sources`): reload once the queue drains,
        // so un-batched edits show their synced state too.
        if (hadPending && this.pendingSyncCount === 0) this.scheduleReload();
    }

    /** Re-query unfinished batches; when one completes, reload the list since tracker sources changed. */
    private async refreshBatches() {
        let anyFinished = false;
        const open = Object.values(this.batches).filter(b => !b.finished);

        await Promise.all(open.map(async b => {
            try {
                const next = await listApi.getBatchStatus(b.batchId);
                // Dismissed while the request was in flight.
                if (!(b.batchId in this.batches)) return;
                this.batches[b.batchId] = next;
                if (next.finished) {
                    anyFinished = true;
                    this.settleBatch(next);
                }
            } catch (err) {
                console.error("Failed to load batch status:", err);
            }
        }));

        if (anyFinished) this.scheduleReload();
    }

    /** A batch that finished cleanly clears itself after a moment; one with issues stays until dismissed. */
    private settleBatch(b: BatchStatus) {
        if (b.failed + b.skipped > 0) return;
        setTimeout(() => {
            const cur = this.batches[b.batchId];
            if (cur?.finished && cur.failed + cur.skipped === 0) this.dismissBatch(b.batchId);
        }, 4000);
    }

    /** Start following a batch. Zero-job batches (nothing to sync) are not tracked. */
    async trackBatch(batchId: string, enqueued: number, meta: BatchMeta) {
        if (enqueued === 0) return;
        this.batchMeta[batchId] = meta;
        try {
            const status = await listApi.getBatchStatus(batchId);
            this.batches[batchId] = status;
            // Already done (fast batch): no later event will trigger the reload.
            if (status.finished) {
                this.settleBatch(status);
                this.scheduleReload();
            } else {
                void this.onSyncEvent(); // refresh status now and start following it
            }
        } catch (err) {
            console.error("Failed to track batch:", err);
        }
    }

    dismissBatch(batchId: string) {
        delete this.batches[batchId];
        delete this.batchMeta[batchId];
    }

    /** "Retry now" for a paused lane (offline / auth expired / rate limited). */
    async resumeTracker(tracker: string) {
        await listApi.resumeSyncLane(tracker);
        await this.onSyncEvent();
    }

    async batchUpsert(bodies: UpsertEntryBody[]) {
        const res = await listApi.batchUpsert(bodies);
        // Local writes are committed already; show them now, tracker sync trails behind.
        await this.loadData(true, true);
        await this.trackBatch(res.batchId, res.enqueued, { kind: "edit" });
        return res;
    }

    async batchRemove(cids: string[]) {
        const res = await listApi.batchDelete(cids);
        const gone = new Set(cids.filter(c => !res.errors.some(e => e.cid === c)));
        this.entries = this.entries.filter(e => !gone.has(e.cid));
        this.normalized = this.normalized.filter(e => !gone.has(e.original.cid));
        await this.trackBatch(res.batchId, res.enqueued, { kind: "delete" });
        return res;
    }

    /** "On X but not Y": push these entries to `tracker` only. */
    async pushToTracker(tracker: string, cids: string[]) {
        const res = await listApi.pushToTracker(tracker, cids);
        await this.trackBatch(res.batchId, res.enqueued, { kind: "push", tracker });
        return res;
    }

    async retryFailed(batchId?: string) {
        const n = await listApi.retryFailedSyncs(batchId);
        if (batchId && n > 0 && this.batches[batchId]) {
            // Reopen it so refreshBatches follows it again.
            this.batches[batchId] = { ...this.batches[batchId], finished: false };
        }
        await this.onSyncEvent();
        return n;
    }

    async cancelBatch(batchId: string) {
        const n = await listApi.cancelBatch(batchId);
        await this.onSyncEvent();
        return n;
    }

    resetFilters() {
        this.activeStatus = "ALL";
        this.activeType = "ALL";
        this.searchQuery = "";
        this.activeSort = "SCORE_DESC";
        this.showConflicts = false;
        this.missingOn = "ALL";
    }

    openEdit(entry: EnrichedListEntry) {
        this.selectedEntry = entry;
        this.isModalOpen = true;
    }

    updateEntryProgressLocal(cid: string, progress: number, status: ListStatus) {
        const idx = this.entries.findIndex(e => e.cid === cid);
        if (idx !== -1) {
            const updatedEntry = {
                ...this.entries[idx],
                progress,
                status
            };

            this.entries[idx] = updatedEntry;
            this.normalized[idx] = {
                card: normalizeListEntry(updatedEntry),
                original: updatedEntry
            };
        }
    }
}

export const listStore = new ListStore();