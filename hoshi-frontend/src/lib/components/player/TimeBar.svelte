<script lang="ts">
    import type { WatchState } from "@/app/watch.svelte.js";

    let { pageState, buffered = 0 }: { pageState: WatchState; buffered?: number } = $props();

    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

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

    function fracFromEvent(e: MouseEvent | TouchEvent): number {
        if (!trackEl) return 0;
        const rect = trackEl.getBoundingClientRect();
        const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
        return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    }

    function onMouseDown(e: MouseEvent) {
        dragging = true;
        dragFrac = fracFromEvent(e);
    }

    function onMouseMove(e: MouseEvent) {
        hoverFrac = fracFromEvent(e);
        if (dragging) dragFrac = hoverFrac;
    }

    function onMouseUp(e: MouseEvent) {
        if (dragging) {
            const frac = fracFromEvent(e);
            if (pageState.currentDuration > 0) handleSeek(frac * pageState.currentDuration);
            dragging = false;
            dragFrac = null;
        }
    }

    function onMouseLeave() {
        if (!dragging) hoverFrac = null;
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

    // We explicitly calculate which chapter is hovered across the entire track hitbox
    const hoverChapter = $derived(
        hoverTime !== null ? processedChapters.find((c, i) =>
            hoverTime >= c.start && (hoverTime < c.end || (i === processedChapters.length - 1 && hoverTime <= c.end))
        ) : null
    );
    const activeChapterStart = $derived(hoverChapter?.start ?? -1);
</script>

<svelte:window
        onmousemove={dragging ? onMouseMove : undefined}
        onmouseup={dragging ? onMouseUp : undefined}
/>

<div
        class="relative flex items-center w-full py-2 cursor-pointer touch-none"
        bind:this={trackEl}
        onmousedown={onMouseDown}
        onmousemove={onMouseMove}
        onmouseleave={onMouseLeave}
        role="slider"
        tabindex="0"
        aria-label="Seek"
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
                class="absolute bottom-full mb-3 px-2.5 py-1.5 bg-black/90 text-white rounded-lg shadow-xl pointer-events-none flex flex-col items-center gap-0.5 z-[100] transform -translate-x-1/2 whitespace-nowrap border border-white/10"
                style="left: {hoverFrac * 100}%"
        >
            {#if hoverChapter?.title}
                <span class="text-[11px] font-medium text-white/70">{hoverChapter.title}</span>
            {/if}
            <span class="text-xs font-bold tabular-nums">{formatTime(hoverTime)}</span>
        </div>
    {/if}
</div>