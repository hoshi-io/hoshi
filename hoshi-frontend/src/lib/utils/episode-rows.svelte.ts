import type { ContinueItem } from '@/api/progress/types.js';
import { scheduleStore, type RecentlyAiredItem } from '@/app/schedule.svelte.js';
import { appConfig } from '@/stores/config.svelte.js';
import { i18n } from '@/stores/i18n.svelte.js';

export function wideImage(url: string | null | undefined): string | null {
    if (!url) return null;
    return url.replace('_m.', '_w.');
}

/** Title in the user's configured language, falling back to romaji and then the default. */
export function displayTitle(
    titleI18n: Record<string, string> | null | undefined,
    fallback: string
): string {
    if (!appConfig.data) return fallback;
    const lang = appConfig.data.ui.titleLanguage || 'romaji';
    return titleI18n?.[lang] || titleI18n?.['romaji'] || fallback;
}

export function shouldHideNsfw(nsfw: boolean | null | undefined): boolean {
    return !!nsfw && !appConfig.data?.general?.showAdultContent;
}

export function shouldBlurNsfw(nsfw: boolean | null | undefined): boolean {
    return !!nsfw && !!appConfig.data?.general?.blurAdultContent;
}

export function continueUrl(item: ContinueItem, base = ''): string {
    if (item.contentType === 'anime' && item.episode) {
        const url = `${base}/watch/${item.cid}/${item.episode}`;
        return item.timestampSeconds && item.timestampSeconds > 0
            ? `${url}?t=${item.timestampSeconds}`
            : url;
    }
    return `${base}/c/${item.cid}`;
}

export function watchUrl(item: RecentlyAiredItem, base = ''): string {
    return `${base}/watch/${item.card.cid}/${item.episode}`;
}

/** 0-100 */
export function continueProgress(item: ContinueItem): number {
    if (!item.timestampSeconds || !item.episodeDurationSeconds) return 0;
    return Math.min((item.timestampSeconds / item.episodeDurationSeconds) * 100, 100);
}

/**
 * "12" when the episode has its own title (shown next to it), otherwise "Episode 12".
 */
export function episodeLabel(
    episode: number | string | null | undefined,
    unitTitle: string | null | undefined
): { number: string; title: string | null } {
    return {
        number: unitTitle
            ? String(episode)
            : i18n.t('watch.episode_number', { num: episode ?? '?' }),
        title: unitTitle ?? null,
    };
}

const toMs = (ts: number) => (ts > 1e11 ? ts : ts * 1000);

export function relativeAired(item: RecentlyAiredItem): string {
    const hours = Math.floor((Date.now() - toMs(item.airingAt)) / 3_600_000);
    if (hours < 1) return i18n.t('home.recently_aired.just_aired');
    if (hours < 24) return i18n.t('home.recently_aired.hours_ago', { count: hours });
    return i18n.t('home.recently_aired.days_ago', { count: Math.floor(hours / 24) });
}

const MIN_RECENT_SCORE = 6;

/** Continue-watching/reading items with hidden NSFW removed. Call during component init. */
export function useContinueItems(getItems: () => ContinueItem[], limit = Infinity) {
    const items = $derived(
        getItems()
            .filter((item) => !shouldHideNsfw(item.nsfw))
            .slice(0, limit)
    );
    return {
        get items() { return items; },
    };
}

/** Recently aired episodes (loads the schedule, filters NSFW and low scores). Call during component init. */
export function useRecentlyAired(limit = Infinity) {
    $effect(() => {
        scheduleStore.load();
    });

    const items = $derived(
        scheduleStore.recentlyAired
            .filter(
                (item) =>
                    !shouldHideNsfw(item.card.nsfw) &&
                    item.card.score != null &&
                    item.card.score >= MIN_RECENT_SCORE
            )
            .slice(0, limit)
    );
    return {
        get items() { return items; },
    };
}