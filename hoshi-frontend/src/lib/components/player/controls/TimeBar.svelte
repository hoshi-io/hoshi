<script lang="ts">
    import type { Chapter } from '../types.js';

    interface Props {
        currentTime: number;
        duration: number;
        buffered: number;
        chapters: Chapter[];
        onSeek: (time: number) => void;
    }

    let { currentTime, duration, buffered, chapters, onSeek }: Props = $props();

    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

    const progress = $derived.by(() => {
        if (dragging && dragFrac !== null) return dragFrac;
        return duration > 0 ? Math.min(currentTime / duration, 1) : 0;
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
            if (duration > 0) onSeek(frac * duration);
            dragging = false;
            dragFrac = null;
        }
    }

    function onMouseLeave() {
        if (!dragging) hoverFrac = null;
    }

    function onTouchStart(e: TouchEvent) {
        dragging = true;
        dragFrac = fracFromEvent(e);
    }

    function onTouchMove(e: TouchEvent) {
        e.preventDefault();
        dragFrac = fracFromEvent(e);
        hoverFrac = dragFrac;
    }

    function onTouchEnd(e: TouchEvent) {
        if (dragging && dragFrac !== null) {
            if (duration > 0) onSeek(dragFrac * duration);
        }
        dragging = false;
        dragFrac = null;
        hoverFrac = null;
    }

    function formatTime(seconds: number) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
        return `${m}:${s.toString().padStart(2, '0')}`;
    }

    const processedChapters = $derived.by(() => {
        if (duration <= 0) return [];

        const sorted = [...chapters].sort((a, b) => a.start - b.start);

        let sanitized: { start: number; end: number; title: string }[] = [];
        let currentTimelineTime = 0;

        for (const ch of sorted) {
            if (ch.end <= ch.start) continue;

            if (ch.start > currentTimelineTime) {
                sanitized.push({
                    start: currentTimelineTime,
                    end: ch.start,
                    title: ''
                });
                currentTimelineTime = ch.start;
            }

            if (ch.start <= currentTimelineTime) {
                if (ch.end <= currentTimelineTime) continue;

                sanitized.push({
                    start: currentTimelineTime,
                    end: ch.end,
                    title: ch.title
                });
                currentTimelineTime = ch.end;
            }
        }

        if (currentTimelineTime < duration) {
            sanitized.push({
                start: currentTimelineTime,
                end: duration,
                title: ''
            });
        }

        return sanitized.map(seg => ({
            ...seg,
            width: ((seg.end - seg.start) / duration) * 100
        }));
    });

    function getSegmentProgress(start: number, end: number, current: number) {
        if (current <= start) return 0;
        if (current >= end) return 100;
        return ((current - start) / (end - start)) * 100;
    }

    const hoverTime = $derived(hoverFrac !== null ? hoverFrac * duration : null);
    const hoverChapter = $derived(
        hoverTime !== null ? chapters.find(c => hoverTime >= c.start && hoverTime < c.end) : null
    );
</script>

<svelte:window
        onmousemove={dragging ? onMouseMove : undefined}
        onmouseup={dragging ? onMouseUp : undefined}
        ontouchmove={dragging ? onTouchMove : undefined}
        ontouchend={dragging ? onTouchEnd : undefined}
/>

<div
        class="relative flex items-center w-full py-1 cursor-pointer touch-none group"
        bind:this={trackEl}
        onmousedown={onMouseDown}
        onmousemove={onMouseMove}
        onmouseleave={onMouseLeave}
        ontouchstart={onTouchStart}
        role="slider"
        tabindex="0"
        aria-label="Seek"
        aria-valuemin={0}
        aria-valuemax={duration}
        aria-valuenow={currentTime}
>
    <div class="relative flex items-center w-full h-[10px] gap-0.5">

        {#each processedChapters as segment}
            <div
                    class="relative h-[6px] bg-white/20 rounded-sm overflow-hidden transition-all duration-150 hover:h-[10px] hover:bg-white/30"
                    style="width: {segment.width}%"
            >
                <div
                        class="absolute inset-y-0 left-0 bg-white/30 pointer-events-none"
                        style="width: {getSegmentProgress(segment.start, segment.end, buffered * duration)}%"
                ></div>
                <div
                        class="absolute inset-y-0 left-0 bg-primary pointer-events-none"
                        style="width: {getSegmentProgress(segment.start, segment.end, dragging && dragFrac !== null ? dragFrac * duration : currentTime)}%"
                ></div>
            </div>
        {/each}

        <div
                class="absolute top-1/2 w-3.5 h-3.5 bg-white rounded-full pointer-events-none shadow-md transition-transform duration-150 z-10 origin-center"
                style="left: {progress * 100}%; transform: translate(-50%, -50%) scale({dragging || hoverFrac !== null ? '1' : '0'});"
        ></div>
    </div>

    {#if hoverFrac !== null && hoverTime !== null}
        <div class="absolute bottom-full mb-3 px-2.5 py-1.5 bg-black/90 text-white rounded-lg shadow-xl pointer-events-none flex flex-col items-center gap-0.5 z-[100] transform -translate-x-1/2 whitespace-nowrap" style="left: {hoverFrac * 100}%">
            {#if hoverChapter?.title}
                <span class="text-[11px] font-medium text-white/70">{hoverChapter.title}</span>
            {/if}
            <span class="text-xs font-bold tabular-nums">{formatTime(hoverTime)}</span>
        </div>
    {/if}
</div>