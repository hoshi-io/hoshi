import { listApi } from "@/api/list/list";
import type {EnrichedListEntry, ListStatus, UpsertEntryBody, UserStats} from "@/api/list/types";
import type { CoreError } from "@/api/client";
import {type NormalizedCard, normalizeListEntry} from "@/utils/normalize";
import { appConfig } from "@/stores/config.svelte.js";
import type {TrackerInfo} from "@/api/tracker/types";
import {integrationsApi} from "@/api/tracker/tracker";

export type SortOption = "SCORE_DESC" | "TITLE_ASC" | "TITLE_DESC" | "PROGRESS_DESC" | "PROGRESS_ASC";
export type StatusFilter = "ALL" | "CURRENT" | "COMPLETED" | "PLANNING" | "PAUSED" | "DROPPED";
export type TypeFilter = "ALL" | "anime" | "manga" | "novel";
export type MissingTrackerFilter = "ALL" | "anilist" | "mal" | "kitsu" | "simkl";

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

    updateConnectedTrackers(trackers: TrackerInfo[]) {
        this.connectedTrackers = trackers
            .filter(t => t.connected)
            .map(t => t.name.toLowerCase());
    }

    async loadData(forceRefresh = false) {
        if (this.isInitialized && !forceRefresh) return;
        listStore.updateConnectedTrackers(await integrationsApi.getAll());

        this.isLoading = true;
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
        const res = await listApi.getEntry(entry.cid);
        if (res.found && res.entry) {
            this.upsertLocal(body, res.entry);
        }
    }

    async removeEntry(cid: string) {
        await listApi.delete(cid);
        this.deleteLocal(cid);
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