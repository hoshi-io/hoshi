import { listApi } from "@/api/list/list";
import type {EnrichedListEntry, UpsertEntryBody, UserStats} from "@/api/list/types";
import type { CoreError } from "@/api/client";
import {type NormalizedCard, normalizeListEntry} from "@/utils/normalize";
import { appConfig } from "@/stores/config.svelte.js";

export type SortOption = "SCORE_DESC" | "TITLE_ASC" | "TITLE_DESC" | "PROGRESS_DESC" | "PROGRESS_ASC";
export type StatusFilter = "ALL" | "CURRENT" | "COMPLETED" | "PLANNING" | "PAUSED" | "DROPPED";
export type TypeFilter = "ALL" | "anime" | "manga" | "novel";

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

    activeStatus = $state<StatusFilter>("ALL");
    activeType = $state<TypeFilter>("ALL");
    searchQuery = $state("");
    activeSort = $state<SortOption>("SCORE_DESC");
    isMobileSearchActive = $state(false);
    isDrawerOpen = $state(false);
    selectedEntry = $state<EnrichedListEntry | null>(null);
    isModalOpen = $state(false);

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
            return matchesStatus && matchesType && matchesSearch;
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

    async loadData(forceRefresh = false) {
        if (this.isInitialized && !forceRefresh) return;

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

    resetFilters() {
        this.activeStatus = "ALL";
        this.activeType = "ALL";
        this.searchQuery = "";
        this.activeSort = "SCORE_DESC";
    }

    openEdit(entry: EnrichedListEntry) {
        this.selectedEntry = entry;
        this.isModalOpen = true;
    }
}

export const listStore = new ListStore();