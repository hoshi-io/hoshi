<script lang="ts">
    import { onDestroy } from 'svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import {formatTime} from "@/tv/watch/TvPlayerUtils.js";

    let { pageState }: { pageState: WatchState } = $props();

    // Left/Right accumulate into `pending`; the real seek is sent once the user pauses.
    let pending = $state<number | null>(null);
    let commitTimer: ReturnType<typeof setTimeout> | null = null;
    let settleTimer: ReturnType<typeof setTimeout> | null = null;

    const step = $derived(appConfig.data?.player.seekStep ?? 10);
    const duration = $derived(pageState.currentDuration);
    const position = $derived(pending ?? pageState.currentTime);
    const frac = $derived(duration > 0 ? Math.min(1, position / duration) : 0);

    const chapterTitle = $derived(
        pageState.chapters.find((c) => position >= c.start && position < c.end)?.title ?? ''
    );

    // Chapter segments with gaps filled, same idea as the desktop TimeBar.
    const segments = $derived.by(() => {
        if (duration <= 0) return [{ start: 0, end: 1, width: 100 }];
        const sorted = [...pageState.chapters].sort((a, b) => a.start - b.start);
        const out: { start: number; end: number }[] = [];
        let cursor = 0;
        for (const ch of sorted) {
            if (ch.end <= ch.start || ch.end <= cursor) continue;
            if (ch.start > cursor) out.push({ start: cursor, end: ch.start });
            out.push({ start: Math.max(cursor, ch.start), end: ch.end });
            cursor = ch.end;
        }
        if (cursor < duration) out.push({ start: cursor, end: duration });
        return out.map((s) => ({ ...s, width: ((s.end - s.start) / duration) * 100 }));
    });

    function fill(start: number, end: number, t: number): number {
        if (t <= start) return 0;
        if (t >= end) return 100;
        return ((t - start) / (end - start)) * 100;
    }

    $effect(() => {
        pageState.isScrubbing = pending !== null;
    });

    onDestroy(() => {
        if (commitTimer) clearTimeout(commitTimer);
        if (settleTimer) clearTimeout(settleTimer);
        pageState.isScrubbing = false;
    });

    function nudge(dir: 1 | -1) {
        if (duration <= 0) return;
        if (settleTimer) {
            clearTimeout(settleTimer);
            settleTimer = null;
        }
        const base = pending ?? pageState.currentTime;
        pending = Math.min(duration, Math.max(0, base + dir * step));
        if (commitTimer) clearTimeout(commitTimer);
        commitTimer = setTimeout(commit, 700);
    }

    function commit() {
        if (commitTimer) {
            clearTimeout(commitTimer);
            commitTimer = null;
        }
        if (pending === null) return;
        const target = pending;
        // Keep showing the target until mpv reports the new position, so the bar doesn't jump back.
        pageState.seek(target, false).finally(() => {
            settleTimer = setTimeout(() => {
                pending = null;
                settleTimer = null;
            }, 400);
        });
    }

    function onKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowRight') {
            e.preventDefault();
            nudge(1);
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            nudge(-1);
        } else if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            pageState.togglePlay();
        }
    }
</script>

<!-- data-sn-lr: spatialNav leaves Left/Right to this element. OK toggles play/pause. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
        role="slider"
        tabindex="0"
        data-sn
        data-sn-lr
        data-autofocus
        aria-label={i18n.t('watch.player.seek')}
        aria-valuemin={0}
        aria-valuemax={duration}
        aria-valuenow={position}
        onkeydown={onKeydown}
        onblur={commit}
        class="w-full rounded-lg px-3 py-4 outline-none focus:bg-white/10 focus:ring-4 focus:ring-primary"
>
    <div class="relative flex h-3 items-center gap-0.5">
        {#each segments as seg (seg.start)}
            <div class="relative h-2 overflow-hidden rounded-sm bg-white/20" style="width: {seg.width}%">
                <div
                        class="absolute inset-y-0 left-0 bg-white/30"
                        style="width: {fill(seg.start, seg.end, pageState.bufferedTime)}%"
                ></div>
                <div
                        class="absolute inset-y-0 left-0 bg-white"
                        style="width: {fill(seg.start, seg.end, position)}%"
                ></div>
            </div>
        {/each}
        <div
                class="pointer-events-none absolute top-1/2 size-4 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white shadow-md"
                style="left: {frac * 100}%"
        ></div>
    </div>

    <div class="mt-3 flex items-center justify-between text-base font-semibold tabular-nums">
        <span>{formatTime(position)}</span>
        {#if chapterTitle}
            <span class="truncate px-4 text-sm font-medium text-white/70">{chapterTitle}</span>
        {/if}
        <span class="text-white/60">{formatTime(duration)}</span>
    </div>
</div>