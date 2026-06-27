import type { FullContent, ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';
import { contentApi } from '@/api/content/content';
import { progressApi } from '@/api/progress/progress';
import type { CoreError } from '@/api/client';
import { type NormalizedCard, normalizeFullContent } from "@/utils/normalize";

export type NormalizedSection = {
    trending: NormalizedCard[];
    popular: NormalizedCard[];
    topRated: NormalizedCard[];
    seasonal: NormalizedCard[];
    recentlyFinished: NormalizedCard[];
    upcoming: NormalizedCard[];
    topAction: NormalizedCard[];
};

function normalizeSection(section: Record<string, FullContent[]>): NormalizedSection {
    const norm = (arr: FullContent[]) => arr.map(normalizeFullContent);
    return {
        trending: norm(section.trending ?? []),
        popular: norm(section.popular ?? []),
        topRated: norm(section.topRated ?? []),
        seasonal: norm(section.seasonal ?? []),
        recentlyFinished: norm(section.recentlyFinished ?? []),
        upcoming: norm(section.upcoming ?? []),
        topAction: norm(section.topAction ?? []),
    };
}

class HomeState {
    normalized = $state<Record<ContentType, NormalizedSection | null>>({
        anime: null,
        manga: null,
        novel: null,
    });

    continueItems = $state<ContinueItem[]>([]);
    loading = $state(false);
    error = $state<CoreError | null>(null);

    hasData = $derived(this.normalized.anime !== null);

    async load() {
        if (!this.hasData) this.loading = true;
        this.error = null;

        try {
            const [res, progRes] = await Promise.all([
                contentApi.getHome(),
                progressApi.getContinueWatching(20)
            ]);

            this.normalized = {
                anime: normalizeSection(res.anime),
                manga: normalizeSection(res.manga),
                novel: normalizeSection(res.novel),
            };
            this.continueItems = progRes.items || [];
        } catch (err) {
            console.error('Failed to load home content', err);
            if (!this.hasData) this.error = err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async refreshContinueWatching() {
        try {
            const progRes = await progressApi.getContinueWatching(20);
            this.continueItems = progRes.items || [];
        } catch (err) {
            console.error('Failed to refresh continue watching items', err);
        }
    }

    getContinueItems(mode: ContentType) {
        return this.continueItems.filter(item => item.contentType === mode);
    }

    getSection(mode: ContentType) {
        return this.normalized[mode];
    }
}

export const homeState = new HomeState();