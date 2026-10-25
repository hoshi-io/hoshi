<script lang="ts">
    import type { WatchState } from "@/app/watch.svelte.js";
    import { buildSegments, createThumbnails, formatTime, segmentFill } from "@/utils/SeekBar.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { onDestroy } from "svelte";

    let { pageState, buffered = 0 }: { pageState: WatchState; buffered?: number } = $props();

    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

    let trackWidth = $state(0);

    $effect(() => { pageState.isScrubbing = dragging; });
    onDestroy(() => { pageState.isScrubbing = false; });

    function handleSeek(time: number) {
        pageState.seek(time, false);
    }

    const segments = $derived(buildSegments(pageState.chapters, pageState.currentDuration));

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

    const hoverTime = $derived(hoverFrac !== null ? hoverFrac * pageState.currentDuration : null);

    const thumbs = createThumbnails(pageState, () => hoverTime);

    const tooltipLeftPx = $derived.by(() => {
        if (hoverFrac === null) return 0;
        const half = (thumbs.url ? thumbs.width : 40) / 2;
        return Math.max(half, Math.min(trackWidth - half, hoverFrac * trackWidth));
    });

    const hoverChapter = $derived(
        hoverTime !== null ? segments.find((c, i) =>
            hoverTime >= c.start && (hoverTime < c.end || (i === segments.length - 1 && hoverTime <= c.end))
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
    <div class="relative flex items-center w-full h-[12px] gap-0.5">
        {#each segments as segment (segment.start)}
            <div
                    class="relative flex items-center h-full"
                    style="width: {segment.width}%"
            >
                <div
                        class="relative w-full rounded-sm overflow-hidden transition-all duration-200 ease-out {activeChapterStart === segment.start ? 'h-[8px] bg-white/30' : 'h-[6px] bg-white/20'}"
                >
                    <div
                            class="absolute inset-y-0 left-0 bg-white/30 pointer-events-none"
                            style="width: {segmentFill(segment.start, segment.end, buffered * pageState.currentDuration)}%"
                    ></div>
                    <div
                            class="absolute inset-y-0 left-0 bg-white pointer-events-none"
                            style="width: {segmentFill(segment.start, segment.end, progress * pageState.currentDuration)}%"
                    ></div>
                </div>
            </div>
        {/each}

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
            {#if thumbs.url}
                <img
                        src={thumbs.url}
                        alt=""
                        draggable="false"
                        class="rounded-sm object-cover shadow-2xl ring-1 ring-white/20"
                        style="width: {thumbs.width}px"
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