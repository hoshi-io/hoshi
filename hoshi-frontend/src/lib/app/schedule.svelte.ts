import { scheduleApi } from "@/api/schedule/schedule";
import type { AiringEntry } from "@/api/schedule/types";
import type { CoreError } from "@/api/client";
import { i18n } from "@/stores/i18n.svelte.js";
import type { NormalizedCard } from "@/utils/normalize";
import { normalizeFullContent } from "@/utils/normalize";

export type ScheduleGroup = {
    key: string;
    header: string;
    isToday: boolean;
    items: Array<{
        card: NormalizedCard;
        episode: number;
        airingAt: number;
        trackerId: string;
    }>;
};

export type RecentlyAiredItem = {
    card: NormalizedCard;
    episode: number;
    airingAt: number;
    trackerId: string;
    hasUnit: boolean;
};

function getMs(ts: number) {
    return ts > 1e11 ? ts : ts * 1000;
}

function toCard(entry: AiringEntry): NormalizedCard {
    return normalizeFullContent(entry.fullContent);
}

function hasSyncedUnit(entry: AiringEntry): boolean {
    return entry.fullContent.contentUnits?.some(
        u => u.unitNumber === entry.episode && u.contentType === "episode"
    ) ?? false;
}

function buildGroups(entries: AiringEntry[]): ScheduleGroup[] {
    const todayStr    = new Date().toDateString();
    const tomorrowStr = new Date(Date.now() + 86_400_000).toDateString();

    function getDayLabel(d: Date) {
        const s = d.toDateString();
        if (s === todayStr)    return i18n.t("schedule.today");
        if (s === tomorrowStr) return i18n.t("schedule.tomorrow");
        return d.toLocaleDateString(i18n.locale, { weekday: "long", month: "long", day: "numeric" });
    }

    const groups: Record<string, AiringEntry[]> = {};
    for (const e of entries) {
        const d   = new Date(getMs(e.airingAt));
        const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
        (groups[key] ??= []).push(e);
    }

    return Object.keys(groups).sort().map(key => {
        const sorted = groups[key].sort((a, b) => a.airingAt - b.airingAt);
        const d      = new Date(getMs(sorted[0].airingAt));
        return {
            key,
            header:  getDayLabel(d),
            isToday: d.toDateString() === todayStr,
            items:   sorted.map(e => ({
                card:      toCard(e),
                episode:   e.episode,
                airingAt:  e.airingAt,
                trackerId: e.trackerId,
            })),
        };
    });
}

class ScheduleStore {
    entries  = $state<AiringEntry[]>([]);
    isLoading = $state(false);
    error     = $state<CoreError | null>(null);
    myListOnly = $state(false);

    filteredEntries = $derived(
        this.myListOnly
            ? this.entries.filter(e => e.userStatus === "CURRENT" || e.userStatus === "PLANNING")
            : this.entries
    );

    upcoming = $derived(this.filteredEntries.filter(e => getMs(e.airingAt) > Date.now()));
    aired    = $derived(this.filteredEntries.filter(e => getMs(e.airingAt) <= Date.now()));

    groups = $derived(buildGroups(this.upcoming));

    recentlyAired = $derived<RecentlyAiredItem[]>(
        [...this.aired]
            .sort((a, b) => b.airingAt - a.airingAt)
            .map(e => ({
                card:      toCard(e),
                episode:   e.episode,
                airingAt:  e.airingAt,
                trackerId: e.trackerId,
                hasUnit:   hasSyncedUnit(e),
            }))
    );

    async load(force = false) {
        if (!force && this.entries.length > 0) return;

        this.isLoading = true;
        this.error = null;

        try {
            this.entries = await scheduleApi.get({ daysBack: 2, daysAhead: 7 });
        } catch (err) {
            console.error("Failed to load schedule:", err);
            this.error = err as CoreError;
            this.entries = [];
        } finally {
            this.isLoading = false;
        }
    }

    toggleMyList() { this.myListOnly = !this.myListOnly; }
}

export const scheduleStore = new ScheduleStore();