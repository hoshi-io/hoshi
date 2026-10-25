import type { ContentUnit } from '@/api/content/types.js';
import type { AnimeProgress } from '@/api/progress/types.js';
import { i18n } from '@/stores/i18n.svelte.js';
import {wideImage} from "@/utils/episode-rows.svelte.js";


export function isInProgress(p?: AnimeProgress | null): p is AnimeProgress {
    return !!p && !p.completed && (p.timestampSeconds ?? 0) > 0;
}

export function formatMinutes(minutes: number | null | undefined): string {
    if (!minutes || minutes <= 0) return '';
    if (minutes < 60) return `${minutes}m`;
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function formatTimestamp(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
}

/** Svelte action: scrolls the node into view when `isResume` is true. */
export function scrollIfResume(node: HTMLElement, isResume: boolean) {
    const scroll = (on: boolean) => {
        if (on) requestAnimationFrame(() => node.scrollIntoView({ behavior: 'smooth', block: 'nearest' }));
    };
    scroll(isResume);
    return { update: scroll };
}

type BaseEpisode = {
    number: number;
    title: string | null;
    thumbnail: string | null;
    duration: number | null; // minutes
};

/** Everything a card needs, already formatted. */
export type EpisodeRow = {
    number: number;
    /** "Episode 5" */
    label: string;
    title: string | null;
    thumbnail: string | null;
    /** "24m" or null */
    duration: string | null;
    completed: boolean;
    inProgress: boolean;
    isResume: boolean;
    /** 0-100, only while in progress */
    pct: number | null;
    /** "Resume" / "Watch" on the resume episode, otherwise null */
    tag: string | null;
    /** "12:34 · 45%" while in progress */
    timestamp: string | null;
    href: string;
};

type Options = {
    cid: () => string;
    units: () => ContentUnit[] | null | undefined;
    /** Total episode count, used to fake the list when there is no per-episode data. */
    total: () => number | null | undefined;
    /** Episode duration in minutes (only shown for episodes with real data). */
    duration: () => number | null | undefined;
    progress: () => AnimeProgress[];
    /** Return false to get an empty list (e.g. movies, manga). Default: always on. */
    enabled?: () => boolean;
    /** Episodes per arc chip. Default 12. */
    arcSize?: number;
    /** Route prefix: '' on desktop, '/tv' on TV. */
    watchBase?: string;
};

/** Builds the episode grid state. Call during component init (registers an $effect). */
export function useEpisodeList(opts: Options) {
    const arcSize = opts.arcSize ?? 12;
    const base = opts.watchBase ?? '';

    const progressMap = $derived(new Map(opts.progress().map((p) => [p.episode, p])));

    const episodes = $derived.by((): BaseEpisode[] => {
        if (opts.enabled && !opts.enabled()) return [];

        const enriched = (opts.units() ?? [])
            .filter((u) => u.contentType === 'episode' && u.title)
            .sort((a, b) => a.unitNumber - b.unitNumber);

        if (enriched.length > 0) {
            const duration = opts.duration() ?? null;
            return enriched.map((u) => ({
                number: u.unitNumber,
                title: u.title ?? null,
                thumbnail: wideImage(u.thumbnailUrl),
                duration
            }));
        }

        const total = opts.total();
        const count = total && total > 0 ? total : 12;
        return Array.from({ length: count }, (_, i) => ({
            number: i + 1,
            title: null,
            thumbnail: null,
            duration: null
        }));
    });

    const resumeEpisode = $derived.by(() => {
        const progress = opts.progress();
        const inProgress = progress
            .filter((p) => !p.completed && p.timestampSeconds && p.timestampSeconds > 0)
            .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];
        if (inProgress) return inProgress.episode;

        const done = new Set(progress.filter((p) => p.completed).map((p) => p.episode));
        return episodes.find((ep) => !done.has(ep.number))?.number ?? episodes[0]?.number ?? 1;
    });

    const arcs = $derived.by(() => {
        if (episodes.length <= arcSize) return null;
        const out: { start: number; end: number; label: string }[] = [];
        for (let i = 0; i < episodes.length; i += arcSize) {
            const slice = episodes.slice(i, i + arcSize);
            const start = slice[0].number;
            const end = slice[slice.length - 1].number;
            out.push({ start, end, label: `${start}–${end}` });
        }
        return out;
    });

    const resumeArcIndex = $derived.by(() => {
        if (!arcs) return 0;
        const idx = arcs.findIndex((a) => resumeEpisode >= a.start && resumeEpisode <= a.end);
        return idx >= 0 ? idx : 0;
    });

    let selectedArc = $state(0);
    $effect(() => {
        selectedArc = resumeArcIndex;
    });

    function watchHref(ep: number, prog?: AnimeProgress | null): string {
        const url = `${base}/watch/${opts.cid()}/${ep}`;
        return isInProgress(prog) ? `${url}?t=${prog.timestampSeconds}` : url;
    }

    const visible = $derived.by((): EpisodeRow[] => {
        const arc = arcs ? (arcs[selectedArc] ?? arcs[0]) : null;
        const list = arc
            ? episodes.filter((ep) => ep.number >= arc.start && ep.number <= arc.end)
            : episodes;

        return list.map((ep) => {
            const prog = progressMap.get(ep.number);
            const completed = prog?.completed ?? false;
            const inProgress = isInProgress(prog);
            const isResume = ep.number === resumeEpisode;
            const pct =
                inProgress && prog.episodeDurationSeconds && prog.episodeDurationSeconds > 0
                    ? Math.min(100, Math.round((prog.timestampSeconds! / prog.episodeDurationSeconds) * 100))
                    : null;

            return {
                number: ep.number,
                label: i18n.t('content.episode_title', { num: ep.number }),
                title: ep.title,
                thumbnail: ep.thumbnail,
                duration: formatMinutes(ep.duration) || null,
                completed,
                inProgress,
                isResume,
                pct,
                tag:
                    isResume && !completed
                        ? inProgress
                            ? i18n.t('content.resume')
                            : i18n.t('home.hero.watch')
                        : null,
                timestamp: inProgress
                    ? formatTimestamp(prog.timestampSeconds!) + (pct !== null ? ` · ${pct}%` : '')
                    : null,
                href: watchHref(ep.number, prog)
            };
        });
    });

    return {
        get episodes() { return episodes; },
        /** Rows for the current arc, ready to render. */
        get visible() { return visible; },
        /** null when the list is short enough to show at once. */
        get arcs() { return arcs; },
        get selectedArc() { return selectedArc; },
        set selectedArc(i: number) { selectedArc = i; },
        get resumeEpisode() { return resumeEpisode; },
        progressFor: (ep: number) => progressMap.get(ep),
        watchHref
    };
}