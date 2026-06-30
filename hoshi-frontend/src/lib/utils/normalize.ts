import type { ContentType, ExtensionSearchResult, FullContent, TrackerMedia } from "@/api/content/types";
import { primaryMetadata } from "@/api/content/types";
import { appConfig } from "@/stores/config.svelte.js";
import { i18n } from "@/stores/i18n.svelte.js";
import type { EnrichedListEntry } from "@/api/list/types";
import {page} from "$app/state";

export type NormalizedCard = {
    cid: string;
    titleI18n: Record<string, string>;
    titleDefault: string;
    cover: string;
    score: number | null;
    year: string | null;
    nsfw: boolean;
    hasAdultGenre: boolean;
    contentTypeLabel: string | null;
    synopsis: string | null;
    status: string | null;
    bannerImage: string | null;
    trailerUrlRaw: string | null;
    episodeCount: number | null;
    contentType: string;
    href: string;
    imageHeaders?: Record<string, string>;
};

export function getCardTitle(card: NormalizedCard): string {
    const lang = appConfig.data?.ui?.titleLanguage || "romaji";
    return card.titleI18n?.[lang] || card.titleDefault || "";
}

export function getCardShouldBlur(card: NormalizedCard): boolean {
    return (
        (card.nsfw || card.hasAdultGenre) &&
        (appConfig.data?.general?.blurAdultContent ?? false)
    );
}

export function getCardTrailerUrl(card: NormalizedCard): string | null {
    if (appConfig.data?.ui?.disableCardTrailers) return null;
    return card.trailerUrlRaw;
}

export function getCardScore(card: NormalizedCard): string | null {
    if (card.score === null) return null;
    const format = appConfig.data?.content?.scoreFormat;

    switch (format) {
        case 'point100':
            return String(Math.round(card.score * 10));
        case 'point10Decimal':
            return card.score.toFixed(1);
        case 'point5Stars':
            return '★'.repeat(Math.round(card.score / 2));
        case 'point10':
        default:
            return String(Math.round(card.score));
    }
}

export function formatScore(rating: number | null): string | null {
    if (rating === null || rating === undefined) return null;
    const format = appConfig.data?.content?.scoreFormat;

    switch (format) {
        case 'point100':      return String(Math.round(rating * 10));
        case 'point10Decimal': return rating.toFixed(1);
        case 'point5Stars':   return '★'.repeat(Math.round(rating / 2));
        case 'point10':
        default:              return String(Math.round(rating));
    }
}

export function getCardScoreIsStars(card: NormalizedCard): boolean {
    return appConfig.data?.content?.scoreFormat === 'point5Stars';
}

export function normalizeFullContent(item: FullContent): NormalizedCard {
    const meta = primaryMetadata(item, appConfig.data?.content?.preferredMetadataProvider);
    const formatKey = meta?.subtype?.toUpperCase();
    const score = meta?.rating ?? null;
    const hasAdultGenre = meta?.genres?.some(
        g => g.toLowerCase() === "hentai" || g.toLowerCase() === "adult"
    ) ?? false;

    return {
        cid: item.content.cid,
        titleI18n: meta?.titleI18n ?? {},
        titleDefault: meta?.title ?? "",
        cover: meta?.coverImage ?? "",
        score,
        year: meta?.releaseDate ? meta.releaseDate.split("-")[0] : null,
        nsfw: item.content.nsfw,
        hasAdultGenre,
        contentTypeLabel: formatKey ? (i18n.t(`card.${formatKey}`) || formatKey) : null,
        synopsis: meta?.synopsis?.replace(/<[^>]*>?/gm, "") ?? null,
        status: meta?.status ?? null,
        bannerImage: meta?.bannerImage ?? null,
        trailerUrlRaw: meta?.trailerUrl ?? null,
        episodeCount: meta?.epsOrChapters ?? null,
        contentType: item.content.contentType,
        href: `/c/${item.content.cid}`,
    };
}

export function normalizeTrackerMedia(item: TrackerMedia, tracker: string): NormalizedCard {
    const formatKey = item.format?.toUpperCase();
    const score = item?.rating ?? null;
    const hasAdultGenre = item.genres?.some(
        g => g.toLowerCase() === "hentai" || g.toLowerCase() === "adult"
    ) ?? false;

    return {
        cid: `${item.trackerId}`,
        titleI18n: item.titleI18n ?? {},
        titleDefault: item.title ?? "",
        cover: item.coverImage ?? "",
        score,
        year: item.releaseDate ? item.releaseDate.split("-")[0] : null,
        nsfw: item.nsfw,
        hasAdultGenre,
        contentTypeLabel: formatKey ?? null,
        synopsis: item.synopsis?.replace(/<[^>]*>?/gm, "") ?? null,
        status: item.status ?? null,
        bannerImage: item.bannerImage ?? null,
        trailerUrlRaw: item.trailerUrl ?? null,
        episodeCount: item.episodeCount ?? item.chapterCount ?? null,
        contentType: item.contentType,
        href: `/c/${tracker}/${btoa(item.trackerId)}`,
    };
}

export function getCardContentTypeLabel(card: NormalizedCard): string | null {
    if (!card.contentTypeLabel) return null;
    return i18n.t(`card.${card.contentTypeLabel}`) || card.contentTypeLabel;
}

export function normalizeExtensionResult(
    item: ExtensionSearchResult,
    extensionId: string,
    contentType: ContentType
): NormalizedCard {
    return {
        cid: `${item.id}`,
        titleI18n: {},
        titleDefault: item.title ?? "",
        cover: item.image ?? "",
        score: null,
        year: null,
        nsfw: item.nsfw ?? false,
        hasAdultGenre: false,
        contentTypeLabel: null,
        synopsis: null,
        status: null,
        bannerImage: null,
        trailerUrlRaw: null,
        episodeCount: null,
        contentType,
        href: `/c/${extensionId}/${btoa(item.id)}`,
        imageHeaders: undefined,
    };
}

export function normalizeListEntry(entry: EnrichedListEntry): NormalizedCard {
    return {
        cid: entry.cid,
        titleI18n: entry.titleI18n ?? {},
        titleDefault: entry.title ?? "",
        cover: entry.coverImage ?? "",
        score: entry.score ?? null,
        year: null,
        nsfw: entry.nsfw ?? false,
        hasAdultGenre: false,
        contentTypeLabel: null,
        synopsis: null,
        status: entry.status,
        bannerImage: null,
        trailerUrlRaw: null,
        episodeCount: entry.totalUnits ?? null,
        contentType: entry.contentType,
        href: `/c/${entry.cid}`,
    };
}

export type NormalizedRelation = {
    targetCid: string | null;
    targetTrackerName: string;
    targetTrackerId: string;
    relationType: string;
    card: NormalizedCard;
};

export function normalizeRelationCard(relation: {
    targetCid: string | null;
    targetTrackerName: string;
    targetTrackerId: string;
    targetTitle: string;
    targetCoverImage?: string | null;
}): NormalizedCard {
    const href = relation.targetCid
        ? `/c/${relation.targetCid}`
        : `/c/${relation.targetTrackerName}/${btoa(relation.targetTrackerId)}`;

    return {
        cid: relation.targetCid ?? "",
        titleI18n: {},
        titleDefault: relation.targetTitle,
        cover: relation.targetCoverImage ?? "",
        score: null,
        year: null,
        nsfw: false,
        hasAdultGenre: false,
        contentTypeLabel: null,
        synopsis: null,
        status: null,
        bannerImage: null,
        trailerUrlRaw: null,
        episodeCount: null,
        contentType: "",
        href,
    };
}