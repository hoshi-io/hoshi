<script lang="ts">
    import { onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import {
        Play,
        Pause,
        Volume2,
        VolumeX,
        Settings,
        AlertCircle,
        Loader2
    } from "lucide-svelte";
    import {WatchState} from "@/app/watch.svelte.js";

    let layoutState = $state({ isMobile: false });

    // Player State instance
    const pageState = new WatchState();

    onDestroy(() => {
        pageState.destroy();
    });

    // Playback control states
    let isPaused = $state(false);
    let volume = $state(100);
    let isMuted = $state(false);

    // Time placeholder states (no Tauri event for live updates yet)
    let currentTime = $state(0);
    let duration = $state(0);
    let buffered = $state(0);

    // Timebar drag/hover states
    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

    async function togglePlay() {
        try {
            isPaused = await invoke<boolean>("toggle_pause");
        } catch (e) {
            console.error("Failed to toggle pause", e);
        }
    }

    async function handleVolumeChange(e: Event) {
        const target = e.target as HTMLInputElement;
        volume = Number(target.value);
        try {
            await invoke("set_volume", { volume });
            if (isMuted && volume > 0) {
                isMuted = false;
                await invoke("set_muted", { muted: false });
            }
        } catch (e) {
            console.error("Failed to set volume", e);
        }
    }

    async function toggleMute() {
        isMuted = !isMuted;
        try {
            await invoke("set_muted", { muted: isMuted });
        } catch (e) {
            console.error("Failed to set muted", e);
        }
    }

    async function handleSeek(time: number) {
        try {
            await invoke("seek", { target: time, relative: false });
        } catch (e) {
            console.error("Failed to seek", e);
        }
    }

    // Adapt PlayerState chapter marks to timeline chapters
    const formattedChapters = $derived.by(() => {
        if (!pageState.chapters || pageState.chapters.length === 0) return [];
        const sorted = [...pageState.chapters].sort((a, b) => a.time - b.time);

        return sorted.map((ch, idx) => {
            const nextTime = sorted[idx + 1]?.time ?? duration;
            return {
                start: ch.time,
                end: nextTime,
                title: ch.title || ''
            };
        });
    });

    // Processed chapter segments for timebar rendering
    const processedChapters = $derived.by(() => {
        if (duration <= 0) return [];

        const sorted = [...formattedChapters].sort((a, b) => a.start - b.start);
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
            if (duration > 0) handleSeek(frac * duration);
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

    const hoverTime = $derived(hoverFrac !== null ? hoverFrac * duration : null);
    const hoverChapter = $derived(
        hoverTime !== null ? formattedChapters.find(c => hoverTime >= c.start && hoverTime < c.end) : null
    );
</script>

<svelte:head>
    <title>{pageState.animeTitle ? `${pageState.animeTitle} - ${pageState.episodeTitle}` : 'Player'}</title>
</svelte:head>

<svelte:window
        onmousemove={dragging ? onMouseMove : undefined}
        onmouseup={dragging ? onMouseUp : undefined}
/>

<div class="relative w-full h-screen bg-black overflow-hidden font-sans select-none text-white">

    {#if !layoutState.isMobile}
        <!-- Desktop Player UI Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">

            <!-- TOP BAR: Title & Metadata -->
            <div class="z-20 p-6 bg-gradient-to-b from-black/80 via-black/40 to-transparent flex items-start justify-between pointer-events-auto">
                <div class="space-y-1 max-w-2xl">
                    <h1 class="text-xl font-bold tracking-tight text-white/95 drop-shadow-sm">
                        {pageState.animeTitle || (pageState.isLoadingMeta ? 'Loading metadata...' : '')}
                    </h1>
                    {#if pageState.episodeTitle}
                        <p class="text-sm font-medium text-white/70">
                            {pageState.episodeTitle}
                        </p>
                    {/if}
                </div>

                <!-- Non-blocking Loading Status Badge -->
                {#if pageState.isLoadingMeta || pageState.isLoadingPlay}
                    <div class="flex items-center gap-2.5 px-3.5 py-1.5 rounded-full bg-black/60 border border-white/10 text-xs font-medium text-white/90 backdrop-blur-md shadow-lg">
                        <Loader2 class="w-4 h-4 animate-spin text-primary" />
                        <span>Loading video stream...</span>
                    </div>
                {/if}
            </div>

            <!-- MIDDLE OVERLAY: Integrated Non-blocking Status & Error Messages -->
            {#if pageState.error}
                <div class="absolute inset-x-0 top-20 z-30 flex justify-center px-6 pointer-events-none">
                    <div class="pointer-events-auto max-w-lg w-full bg-red-950/90 border border-red-500/30 text-red-100 p-3.5 rounded-xl shadow-2xl backdrop-blur-md flex items-start gap-3">
                        <AlertCircle class="w-5 h-5 text-red-400 shrink-0 mt-0.5" />
                        <div class="flex-1 text-xs space-y-0.5">
                            <p class="font-semibold text-red-200">
                                {pageState.error.key || 'Playback Error'}
                            </p>
                            {#if pageState.error.message}
                                <p class="text-red-300/80 leading-relaxed">
                                    {pageState.error.message}
                                </p>
                            {/if}
                        </div>
                    </div>
                </div>
            {/if}

            <!-- BOTTOM BAR: Timebar & Controls (YouTube Style) -->
            <div class="z-20 p-6 bg-gradient-to-t from-black/90 via-black/50 to-transparent flex flex-col gap-2 pointer-events-auto">

                <!-- TIMEBAR & CHAPTERS -->
                <div
                        class="relative flex items-center w-full py-1 cursor-pointer touch-none group"
                        bind:this={trackEl}
                        onmousedown={onMouseDown}
                        onmousemove={onMouseMove}
                        onmouseleave={onMouseLeave}
                        role="slider"
                        tabindex="0"
                        aria-label="Seek"
                        aria-valuemin={0}
                        aria-valuemax={duration}
                        aria-valuenow={currentTime}
                >
                    <div class="relative flex items-center w-full h-[10px] gap-0.5">
                        {#if processedChapters.length > 0}
                            {#each processedChapters as segment}
                                <div
                                        class="relative h-[6px] bg-white/20 rounded-sm overflow-hidden transition-all duration-150 group-hover:h-[8px] group-hover:bg-white/30"
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
                        {:else}
                            <!-- Default Bar fallback if no chapters are loaded -->
                            <div class="relative w-full h-[6px] bg-white/20 rounded-sm overflow-hidden group-hover:h-[8px]">
                                <div
                                        class="absolute inset-y-0 left-0 bg-white/30 pointer-events-none"
                                        style="width: {buffered * 100}%"
                                ></div>
                                <div
                                        class="absolute inset-y-0 left-0 bg-primary pointer-events-none"
                                        style="width: {progress * 100}%"
                                ></div>
                            </div>
                        {/if}

                        <!-- Drag/Hover Indicator Thumb -->
                        <div
                                class="absolute top-1/2 w-3.5 h-3.5 bg-white rounded-full pointer-events-none shadow-md transition-transform duration-150 z-10 origin-center"
                                style="left: {progress * 100}%; transform: translate(-50%, -50%) scale({dragging || hoverFrac !== null ? '1' : '0'});"
                        ></div>
                    </div>

                    <!-- Hover Time/Chapter Tooltip -->
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

                <!-- CONTROLS ROW -->
                <div class="flex items-center justify-between text-white mt-1">
                    <!-- Left Section: Play/Pause, Time, Volume -->
                    <div class="flex items-center gap-4">
                        <button
                                onclick={togglePlay}
                                class="p-2 -ml-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label={isPaused ? "Play" : "Pause"}
                        >
                            {#if isPaused}
                                <Play class="w-5 h-5 fill-current" />
                            {:else}
                                <Pause class="w-5 h-5 fill-current" />
                            {/if}
                        </button>

                        <!-- Static Time Display -->
                        <div class="text-xs font-mono text-white/80 tabular-nums">
                            {formatTime(currentTime)} / {formatTime(duration)}
                        </div>

                        <!-- Volume Controls -->
                        <div class="flex items-center gap-2 group/volume">
                            <button
                                    onclick={toggleMute}
                                    class="p-1.5 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                    aria-label={isMuted ? "Unmute" : "Mute"}
                            >
                                {#if isMuted || volume === 0}
                                    <VolumeX class="w-5 h-5" />
                                {:else}
                                    <Volume2 class="w-5 h-5" />
                                {/if}
                            </button>

                            <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    value={isMuted ? 0 : volume}
                                    oninput={handleVolumeChange}
                                    class="w-20 h-1 bg-white/20 rounded-lg appearance-none cursor-pointer accent-primary focus:outline-none"
                            />
                        </div>
                    </div>

                    <!-- Right Section: Settings Cogwheel -->
                    <div class="flex items-center gap-2">
                        <button
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label="Settings"
                        >
                            <Settings class="w-5 h-5" />
                        </button>
                    </div>
                </div>

            </div>
        </div>
    {:else}
        <!-- Mobile Layout Placeholder -->
    {/if}

</div>