<script lang="ts">
    import type { WatchState } from "@/app/watch.svelte.js";
    import {i18n} from "@/stores/i18n.svelte.js";
    import {appConfig} from "@/stores/config.svelte.js";
    import {onDestroy, untrack} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    let { pageState, buffered = 0 }: { pageState: WatchState; buffered?: number } = $props();

    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

    let trackWidth = $state(0);

    const player = $derived(appConfig.data?.player);
    const thumbInterval = $derived(Math.max(1, player?.thumbnailIntervalSeconds ?? 10));
    const thumbWidth = $derived(player?.thumbnailWidth ?? 240);
    const thumbsOn = $derived(
        !!player?.thumbnailsEnabled && pageState.isStreamLoaded && pageState.currentDuration > 0
    );

    $effect(() => { pageState.isScrubbing = dragging; });
    onDestroy(() => { pageState.isScrubbing = false; });

    function handleSeek(time: number) {
        pageState.seek(time, false);
    }

    const processedChapters = $derived.by(() => {
        const duration = pageState.currentDuration;
        if (duration <= 0) return [];

        const sorted = [...pageState.chapters].sort((a, b) => a.start - b.start);
        let sanitized: { start: number; end: number; title: string }[] = [];
        let currentTimelineTime = 0;

        for (const ch of sorted) {
            if (ch.end <= ch.start) continue;

            if (ch.start > currentTimelineTime) {
                sanitized.push({ start: currentTimelineTime, end: ch.start, title: '' });
                currentTimelineTime = ch.start;
            }

            if (ch.start <= currentTimelineTime) {
                if (ch.end <= currentTimelineTime) continue;
                sanitized.push({ start: currentTimelineTime, end: ch.end, title: ch.title });
                currentTimelineTime = ch.end;
            }
        }

        if (currentTimelineTime < duration) {
            sanitized.push({ start: currentTimelineTime, end: duration, title: '' });
        }

        return sanitized.map(seg => ({
            ...seg,
            width: ((seg.end - seg.start) / duration) * 100,
        }));
    });

    const progress = $derived.by(() => {
        if (dragging && dragFrac !== null) return dragFrac;
        const duration = pageState.currentDuration;
        return duration > 0 ? Math.min(pageState.currentTime / duration, 1) : 0;
    });

    function fracFromX(clientX: number): number {
        if (!trackEl) return 0;
        const rect = trackEl.getBoundingClientRect();
        return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    }

    function onPointerDown(e: PointerEvent) {
        if (e.pointerType === "mouse" && e.button !== 0) return;
        trackEl?.setPointerCapture(e.pointerId); // keep receiving moves outside the bar
        dragging = true;
        hoverFrac = dragFrac = fracFromX(e.clientX);
    }

    function onPointerMove(e: PointerEvent) {
        const frac = fracFromX(e.clientX);
        if (e.pointerType === "mouse" || dragging) hoverFrac = frac;
        if (dragging) dragFrac = frac;
    }

    function onPointerUp(e: PointerEvent) {
        if (!dragging) return;
        if (pageState.currentDuration > 0) handleSeek(fracFromX(e.clientX) * pageState.currentDuration);
        dragging = false;
        dragFrac = null;
        if (e.pointerType !== "mouse") hoverFrac = null; // no hover on touch after release
    }

    function onPointerCancel() {
        // e.g. the system took over the gesture
        dragging = false;
        dragFrac = null;
        hoverFrac = null;
    }

    function onPointerLeave(e: PointerEvent) {
        if (!dragging && e.pointerType === "mouse") hoverFrac = null;
    }

    function formatTime(seconds: number) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
        return `${m}:${s.toString().padStart(2, '0')}`;
    }

    function getSegmentProgress(start: number, end: number, current: number) {
        if (current <= start) return 0;
        if (current >= end) return 100;
        return ((current - start) / (end - start)) * 100;
    }

    const hoverTime = $derived(hoverFrac !== null ? hoverFrac * pageState.currentDuration : null);

    const thumbBucket = $derived(hoverTime !== null ? Math.floor(hoverTime / thumbInterval) : null);

    let thumbUrl = $state<string | null>(null);
    let thumbReq = 0;
    const thumbCache = new Map<number, string>(); // bucket -> blob URL
    const MAX_FRONT_CACHE = 120;

    function clearThumbs() {
        thumbReq++;
        for (const url of thumbCache.values()) URL.revokeObjectURL(url);
        thumbCache.clear();
        thumbUrl = null;
    }

    $effect(() => {
        pageState.cid; pageState.epNumber; pageState.isStreamLoaded;
        untrack(clearThumbs);
    });

    $effect(() => {
        const bucket = thumbBucket;
        const step = thumbInterval;
        const on = thumbsOn;
        const id = ++thumbReq;
        if (!on || bucket === null) return;

        const cached = thumbCache.get(bucket);
        if (cached) { thumbUrl = cached; return; }

        const timer = setTimeout(async () => {
            try {
                const buf = await invoke<ArrayBuffer>("get_thumbnail", { time: bucket * step });
                if (id !== thumbReq || buf.byteLength === 0) return; // stale or "no thumbnail"
                const url = URL.createObjectURL(new Blob([buf], { type: "image/jpeg" }));
                if (thumbCache.size >= MAX_FRONT_CACHE) {
                    const oldest = thumbCache.keys().next().value!;
                    URL.revokeObjectURL(thumbCache.get(oldest)!);
                    thumbCache.delete(oldest);
                }
                thumbCache.set(bucket, url);
                thumbUrl = url;
            } catch { /* ignore: no thumbnail */ }
        }, 150);

        return () => clearTimeout(timer);
    });

    onDestroy(clearThumbs);

    const tooltipLeftPx = $derived.by(() => {
        if (hoverFrac === null) return 0;
        const half = (thumbUrl ? thumbWidth : 40) / 2;
        return Math.max(half, Math.min(trackWidth - half, hoverFrac * trackWidth));
    });

    // We explicitly calculate which chapter is hovered across the entire track hitbox
    const hoverChapter = $derived(
        hoverTime !== null ? processedChapters.find((c, i) =>
            hoverTime >= c.start && (hoverTime < c.end || (i === processedChapters.length - 1 && hoverTime <= c.end))
        ) : null
    );
    const activeChapterStart = $derived(hoverChapter?.start ?? -1);
</script>

<div
        class="relative flex items-center w-full py-3 cursor-pointer touch-none"
        bind:this={trackEl}
        bind:clientWidth={trackWidth}
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        onpointercancel={onPointerCancel}
        onpointerleave={onPointerLeave}
        role="slider"
        tabindex="0"
        aria-label={i18n.t('watch.player.seek')}
        aria-valuemin={0}
        aria-valuemax={pageState.currentDuration}
        aria-valuenow={pageState.currentTime}
>
    <!-- Increased internal container height to h-[12px] to give the 8px bar room to transition cleanly without jumping -->
    <div class="relative flex items-center w-full h-[12px] gap-0.5">
        {#if processedChapters.length > 0}
            {#each processedChapters as segment}
                <div
                        class="relative flex items-center h-full"
                        style="width: {segment.width}%"
                >
                    <!-- Instead of CSS group-hover, we apply height/color based on our JS logic -->
                    <div
                            class="relative w-full rounded-sm overflow-hidden transition-all duration-200 ease-out {activeChapterStart === segment.start ? 'h-[8px] bg-white/30' : 'h-[6px] bg-white/20'}"
                    >
                        <div
                                class="absolute inset-y-0 left-0 bg-white/30 pointer-events-none"
                                style="width: {getSegmentProgress(segment.start, segment.end, buffered * pageState.currentDuration)}%"
                        ></div>
                        <div
                                class="absolute inset-y-0 left-0 bg-white pointer-events-none"
                                style="width: {getSegmentProgress(segment.start, segment.end, dragging && dragFrac !== null ? dragFrac * pageState.currentDuration : pageState.currentTime)}%"
                        ></div>
                    </div>
                </div>
            {/each}
        {:else}
            <div class="relative flex items-center w-full h-full">
                <div
                        class="relative w-full rounded-sm overflow-hidden transition-all duration-200 ease-out {hoverFrac !== null ? 'h-[8px] bg-white/30' : 'h-[6px] bg-white/20'}"
                >
                    <div
                            class="absolute inset-y-0 left-0 bg-white/30 pointer-events-none"
                            style="width: {buffered * 100}%"
                    ></div>
                    <div
                            class="absolute inset-y-0 left-0 bg-white pointer-events-none"
                            style="width: {progress * 100}%"
                    ></div>
                </div>
            </div>
        {/if}

        <div
                class="absolute top-1/2 w-3.5 h-3.5 bg-white rounded-full pointer-events-none shadow-md transition-transform duration-200 ease-out z-10 origin-center"
                style="left: {progress * 100}%; transform: translate(-50%, -50%) scale({dragging || hoverFrac !== null ? '1' : '0'});"
        ></div>
    </div>

    {#if hoverFrac !== null && hoverTime !== null}
        <div
                class="absolute bottom-full mb-3 flex flex-col items-center gap-1.5 pointer-events-none z-[100] -translate-x-1/2"
                style="left: {tooltipLeftPx}px"
        >
            {#if thumbUrl}
                <img
                        src={thumbUrl}
                        alt=""
                        draggable="false"
                        class="rounded-lg object-cover shadow-2xl ring-1 ring-white/20"
                        style="width: {thumbWidth}px"
                />
            {/if}
            <div class="flex flex-col items-center px-2 py-0.5 rounded-md bg-black/80 backdrop-blur-sm text-white whitespace-nowrap">
                {#if hoverChapter?.title}
                    <span class="text-[10px] font-medium text-white/70 leading-tight">{hoverChapter.title}</span>
                {/if}
                <span class="text-xs font-bold tabular-nums leading-tight">{formatTime(hoverTime)}</span>
            </div>
        </div>
    {/if}
</div>