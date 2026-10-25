<script lang="ts">
    import { onDestroy } from 'svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { buildSegments, createThumbnails, formatTime, segmentFill } from '@/utils/SeekBar.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';

    let { pageState }: { pageState: WatchState } = $props();

    let pending = $state<number | null>(null);
    let commitTimer: ReturnType<typeof setTimeout> | null = null;
    let settleTimer: ReturnType<typeof setTimeout> | null = null;

    const step = $derived(appConfig.data?.player.seekStep ?? 10);
    const duration = $derived(pageState.currentDuration);
    const position = $derived(pending ?? pageState.currentTime);
    const frac = $derived(duration > 0 ? Math.min(1, position / duration) : 0);

    const segments = $derived(buildSegments(pageState.chapters, duration));
    const chapterTitle = $derived(
        segments.find((s) => position >= s.start && position < s.end)?.title ?? ''
    );

    // Preview thumbnails only while a seek is pending.
    const thumbs = createThumbnails(pageState, () => (pending !== null ? position : null));

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
        class="relative w-full rounded-lg px-3 py-4 outline-none focus:bg-white/10 focus:ring-4 focus:ring-primary"
>
    <!-- Thumbnail Preview Tooltip (Shown when scrubbing via TV remote Left/Right) -->
    {#if pending !== null && thumbs.enabled}
        <div
                class="absolute bottom-full mb-4 flex flex-col items-center gap-1.5 pointer-events-none z-[100] -translate-x-1/2"
                style="left: {frac * 100}%"
        >
            {#if thumbs.url}
                <img
                        src={thumbs.url}
                        alt=""
                        draggable="false"
                        class="rounded-lg object-cover shadow-2xl ring-2 ring-white/20"
                        style="width: {thumbs.width}px"
                />
            {/if}
            <div class="flex flex-col items-center px-3 py-1 rounded-md bg-black/80 backdrop-blur-md text-white whitespace-nowrap shadow-lg">
                {#if chapterTitle}
                    <span class="text-xs font-medium text-white/70 leading-tight">{chapterTitle}</span>
                {/if}
                <span class="text-sm font-bold tabular-nums leading-tight">{formatTime(position)}</span>
            </div>
        </div>
    {/if}

    <div class="relative flex h-3 items-center gap-0.5">
        {#each segments as seg (seg.start)}
            <div class="relative h-2 overflow-hidden rounded-sm bg-white/20" style="width: {seg.width}%">
                <div
                        class="absolute inset-y-0 left-0 bg-white/30"
                        style="width: {segmentFill(seg.start, seg.end, pageState.bufferedTime)}%"
                ></div>
                <div
                        class="absolute inset-y-0 left-0 bg-white"
                        style="width: {segmentFill(seg.start, seg.end, position)}%"
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