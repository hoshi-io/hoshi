import { onDestroy, untrack } from 'svelte';
import { invoke } from '@tauri-apps/api/core';
import type { WatchState } from '@/app/watch.svelte.js';
import { appConfig } from '@/stores/config.svelte.js';

export type Chapter = { start: number; end: number; title: string };
export type Segment = Chapter & { width: number };

/**
 * Chapters sorted, sanitized and with gaps filled by untitled segments.
 * Always returns at least one segment, so callers don't need a "no chapters" branch.
 */
export function buildSegments(chapters: readonly Chapter[], duration: number): Segment[] {
    if (duration <= 0) return [{ start: 0, end: 1, title: '', width: 100 }];

    const sorted = [...chapters].sort((a, b) => a.start - b.start);
    const out: Chapter[] = [];
    let cursor = 0;

    for (const ch of sorted) {
        if (ch.end <= ch.start || ch.end <= cursor) continue;
        if (ch.start > cursor) out.push({ start: cursor, end: ch.start, title: '' });
        out.push({ start: Math.max(cursor, ch.start), end: ch.end, title: ch.title });
        cursor = ch.end;
    }
    if (cursor < duration) out.push({ start: cursor, end: duration, title: '' });

    return out.map((s) => ({ ...s, width: ((s.end - s.start) / duration) * 100 }));
}

/** How much (0-100) of the [start, end] segment is covered by time `t`. */
export function segmentFill(start: number, end: number, t: number): number {
    if (t <= start) return 0;
    if (t >= end) return 100;
    return ((t - start) / (end - start)) * 100;
}

export function formatTime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    return `${m}:${s.toString().padStart(2, '0')}`;
}

const MAX_FRONT_CACHE = 120;

/**
 * Loads and caches preview thumbnails. Must be called during component init
 * (it registers $effects and onDestroy).
 *
 * @param getTime time to preview in seconds, or null when no preview is wanted
 *                (hover time on desktop, pending seek position on TV).
 */
export function createThumbnails(pageState: WatchState, getTime: () => number | null) {
    const player = $derived(appConfig.data?.player);
    const interval = $derived(Math.max(1, player?.thumbnailIntervalSeconds ?? 10));
    const width = $derived(player?.thumbnailWidth ?? 240);
    const enabled = $derived(
        !!player?.thumbnailsEnabled && pageState.isStreamLoaded && pageState.currentDuration > 0
    );
    const bucket = $derived.by(() => {
        const t = getTime();
        return t === null ? null : Math.floor(t / interval);
    });

    let url = $state<string | null>(null);
    let req = 0;
    const cache = new Map<number, string>(); // bucket -> blob URL

    function clear() {
        req++;
        for (const u of cache.values()) URL.revokeObjectURL(u);
        cache.clear();
        url = null;
    }

    // New episode / stream: drop everything.
    $effect(() => {
        pageState.cid; pageState.epNumber; pageState.isStreamLoaded;
        untrack(clear);
    });

    $effect(() => {
        const b = bucket;
        const step = interval;
        const on = enabled;
        const id = ++req;
        if (!on || b === null) return;

        const cached = cache.get(b);
        if (cached) { url = cached; return; }

        const timer = setTimeout(async () => {
            try {
                const buf = await invoke<ArrayBuffer>('get_thumbnail', { time: b * step });
                if (id !== req || buf.byteLength === 0) return; // stale or "no thumbnail"
                const blobUrl = URL.createObjectURL(new Blob([buf], { type: 'image/jpeg' }));
                if (cache.size >= MAX_FRONT_CACHE) {
                    const oldest = cache.keys().next().value!;
                    URL.revokeObjectURL(cache.get(oldest)!);
                    cache.delete(oldest);
                }
                cache.set(b, blobUrl);
                url = blobUrl;
            } catch { /* ignore: no thumbnail */ }
        }, 150);

        return () => clearTimeout(timer);
    });

    onDestroy(clear);

    return {
        get url() { return url; },
        get width() { return width; },
        get enabled() { return enabled; },
    };
}