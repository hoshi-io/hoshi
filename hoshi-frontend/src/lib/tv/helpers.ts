import { appConfig } from '@/stores/config.svelte.js';

/** Thumbnails come as `_m.` (medium); `_w.` is the wide variant that suits 16:9 cards. */
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