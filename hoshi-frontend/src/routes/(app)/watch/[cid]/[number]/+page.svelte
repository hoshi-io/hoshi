<script lang="ts">
    import { onDestroy } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import {
        Play,
        Pause,
        Volume2,
        VolumeX,
        Settings,
        AlertCircle,
        Loader2,
        RotateCcw,
        RotateCw,
        SkipBack,
        SkipForward,
        Maximize,
        Minimize,
        Check,
    } from "lucide-svelte";
    import { WatchState } from "@/app/watch.svelte.js";

    let layoutState = $state({ isMobile: false });

    // Player State instance — single source of truth for playback state.
    // The page no longer keeps its own isPaused/currentTime/volume copies;
    // those live on pageState and are kept live by the player:// event
    // listeners it wires up itself.
    const pageState = new WatchState();

    onDestroy(() => {
        pageState.destroy();
    });

    // Buffered isn't wired up — there's no mpv property exposed for it yet
    // (no get_buffered command), so the buffered bar just renders at 0 for
    // now rather than faking a value.
    let buffered = $state(0);

    // Timebar drag/hover states
    let trackEl = $state<HTMLDivElement | null>(null);
    let dragging = $state(false);
    let dragFrac = $state<number | null>(null);
    let hoverFrac = $state<number | null>(null);

    // Fullscreen + settings popover — page-local UI state, not playback state.
    let isFullscreen = $state(false);
    let showSettings = $state(false);

    async function toggleFullscreen() {
        try {
            const win = getCurrentWindow();
            const next = !isFullscreen;
            await win.setFullscreen(next);
            isFullscreen = next;
        } catch (e) {
            console.error("Failed to toggle fullscreen", e);
        }
    }

    function handleSeek(time: number) {
        pageState.seek(time, false);
    }

    // Chapters already come as real {start, end, title} ranges from core
    // now (see EpisodeChapter) — no need to infer `end` from the next
    // chapter's start the way the old {title, time} shape required.
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
    const hoverChapter = $derived(
        hoverTime !== null ? processedChapters.find(c => hoverTime >= c.start && hoverTime < c.end) : null
    );
</script>

<svelte:head>
    <title>{pageState.animeTitle ? `${pageState.animeTitle} - ${pageState.episodeTitle}` : 'Player'}</title>
</svelte:head>

<svelte:window
        onmousemove={dragging ? onMouseMove : undefined}
        onmouseup={dragging ? onMouseUp : undefined}
/>

<div class="relative w-full h-screen bg-transparent overflow-hidden font-sans select-none text-white">

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

                <div class="flex items-center gap-2.5">
                    {#if pageState.isLoadingMeta || pageState.isLoadingPlay}
                        <div class="flex items-center gap-2.5 px-3.5 py-1.5 rounded-full bg-black/60 border border-white/10 text-xs font-medium text-white/90 backdrop-blur-md shadow-lg">
                            <Loader2 class="w-4 h-4 animate-spin text-primary" />
                            <span>Loading video stream...</span>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- MIDDLE OVERLAY: Error Messages -->
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

            <!-- BOTTOM BAR: Timebar & Controls -->
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
                        aria-valuemax={pageState.currentDuration}
                        aria-valuenow={pageState.currentTime}
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
                                            style="width: {getSegmentProgress(segment.start, segment.end, buffered * pageState.currentDuration)}%"
                                    ></div>
                                    <div
                                            class="absolute inset-y-0 left-0 bg-primary pointer-events-none"
                                            style="width: {getSegmentProgress(segment.start, segment.end, dragging && dragFrac !== null ? dragFrac * pageState.currentDuration : pageState.currentTime)}%"
                                    ></div>
                                </div>
                            {/each}
                        {:else}
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

                        <div
                                class="absolute top-1/2 w-3.5 h-3.5 bg-white rounded-full pointer-events-none shadow-md transition-transform duration-150 z-10 origin-center"
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

                <!-- SEEK BUTTONS (beneath the bar) -->
                <div class="flex items-center justify-center gap-6 -mt-0.5 mb-0.5">
                    <button
                            onclick={() => pageState.seekRelative(-10)}
                            class="p-1 rounded-lg hover:bg-white/10 transition text-white/70 hover:text-white flex items-center gap-1 text-[11px] font-medium"
                            aria-label="Seek back 10 seconds"
                    >
                        <RotateCcw class="w-4 h-4" />
                        10s
                    </button>
                    <button
                            onclick={() => pageState.seekRelative(10)}
                            class="p-1 rounded-lg hover:bg-white/10 transition text-white/70 hover:text-white flex items-center gap-1 text-[11px] font-medium"
                            aria-label="Seek forward 10 seconds"
                    >
                        10s
                        <RotateCw class="w-4 h-4" />
                    </button>
                </div>

                <!-- CONTROLS ROW -->
                <div class="flex items-center justify-between text-white mt-1">
                    <!-- Left: prev/play/next, time, volume -->
                    <div class="flex items-center gap-1">
                        <button
                                onclick={() => pageState.goToEpisode(pageState.epNumber - 1)}
                                disabled={!pageState.hasPrev}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white disabled:opacity-30 disabled:hover:bg-transparent"
                                aria-label="Previous episode"
                        >
                            <SkipBack class="w-4 h-4 fill-current" />
                        </button>

                        <button
                                onclick={() => pageState.togglePlay()}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label={pageState.isPaused ? "Play" : "Pause"}
                        >
                            {#if pageState.isPaused}
                                <Play class="w-5 h-5 fill-current" />
                            {:else}
                                <Pause class="w-5 h-5 fill-current" />
                            {/if}
                        </button>

                        <button
                                onclick={() => pageState.goToEpisode(pageState.epNumber + 1)}
                                disabled={!pageState.hasNext}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white disabled:opacity-30 disabled:hover:bg-transparent"
                                aria-label="Next episode"
                        >
                            <SkipForward class="w-4 h-4 fill-current" />
                        </button>

                        <div class="text-xs font-mono text-white/80 tabular-nums ml-2">
                            {formatTime(pageState.currentTime)} / {formatTime(pageState.currentDuration)}
                        </div>

                        <div class="flex items-center gap-2 group/volume ml-3">
                            <button
                                    onclick={() => pageState.toggleMute()}
                                    class="p-1.5 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                    aria-label={pageState.isMuted ? "Unmute" : "Mute"}
                            >
                                {#if pageState.isMuted || pageState.volume === 0}
                                    <VolumeX class="w-5 h-5" />
                                {:else}
                                    <Volume2 class="w-5 h-5" />
                                {/if}
                            </button>

                            <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    value={pageState.isMuted ? 0 : pageState.volume}
                                    oninput={(e) => pageState.setVolume(Number((e.target as HTMLInputElement).value))}
                                    class="w-20 h-1 bg-white/20 rounded-lg appearance-none cursor-pointer accent-primary focus:outline-none"
                            />
                        </div>
                    </div>

                    <!-- Right: settings + fullscreen -->
                    <div class="relative flex items-center gap-2">
                        {#if showSettings}
                            <div class="absolute bottom-full right-0 mb-3 w-64 max-h-96 overflow-y-auto bg-black/95 border border-white/10 rounded-xl shadow-2xl backdrop-blur-md p-3 space-y-4 text-sm">

                                {#if pageState.extensionItems.length > 0}
                                    <div class="space-y-1.5">
                                        <p class="text-[11px] font-semibold uppercase tracking-wide text-white/50">Source</p>
                                        <select
                                                value={pageState.selectedExtension}
                                                onchange={(e) => pageState.selectExtension((e.target as HTMLSelectElement).value)}
                                                class="w-full bg-white/10 rounded-lg px-2 py-1.5 text-xs text-white outline-none"
                                        >
                                            {#each pageState.extensionItems as item}
                                                <option value={item.value}>{item.label}</option>
                                            {/each}
                                        </select>
                                    </div>
                                {/if}

                                {#if pageState.serverItems.length > 0}
                                    <div class="space-y-1.5">
                                        <p class="text-[11px] font-semibold uppercase tracking-wide text-white/50">Server</p>
                                        <select
                                                value={pageState.selectedServer}
                                                onchange={(e) => pageState.selectServer((e.target as HTMLSelectElement).value)}
                                                class="w-full bg-white/10 rounded-lg px-2 py-1.5 text-xs text-white outline-none"
                                        >
                                            {#each pageState.serverItems as item}
                                                <option value={item.value}>{item.label}</option>
                                            {/each}
                                        </select>
                                    </div>
                                {/if}

                                {#if pageState.supportsDub}
                                    <label class="flex items-center justify-between text-xs text-white/80 cursor-pointer">
                                        <span>Dub</span>
                                        <input
                                                type="checkbox"
                                                checked={pageState.isDub}
                                                onchange={() => pageState.toggleDub()}
                                                class="accent-primary"
                                        />
                                    </label>
                                {/if}

                                {#if pageState.audioTracks.length > 0}
                                    <div class="space-y-1">
                                        <p class="text-[11px] font-semibold uppercase tracking-wide text-white/50">Audio</p>
                                        {#each pageState.audioTracks as track}
                                            <button
                                                    onclick={() => pageState.setAudioTrack(track.id)}
                                                    class="w-full flex items-center justify-between px-2 py-1.5 rounded-lg hover:bg-white/10 text-xs text-left"
                                            >
                                                <span class="truncate">{track.title || track.lang || `Track ${track.id}`}</span>
                                                {#if track.selected}<Check class="w-3.5 h-3.5 text-primary shrink-0" />{/if}
                                            </button>
                                        {/each}
                                    </div>
                                {/if}

                                {#if pageState.subtitleTracks.length > 0}
                                    <div class="space-y-1">
                                        <p class="text-[11px] font-semibold uppercase tracking-wide text-white/50">Subtitles</p>
                                        <button
                                                onclick={() => pageState.setSubtitleTrack(null)}
                                                class="w-full flex items-center justify-between px-2 py-1.5 rounded-lg hover:bg-white/10 text-xs text-left"
                                        >
                                            <span>Off</span>
                                            {#if pageState.subtitleTracks.every(t => !t.selected)}<Check class="w-3.5 h-3.5 text-primary shrink-0" />{/if}
                                        </button>
                                        {#each pageState.subtitleTracks as track}
                                            <button
                                                    onclick={() => pageState.setSubtitleTrack(track.id)}
                                                    class="w-full flex items-center justify-between px-2 py-1.5 rounded-lg hover:bg-white/10 text-xs text-left"
                                            >
                                                <span class="truncate">{track.title || track.lang || `Track ${track.id}`}</span>
                                                {#if track.selected}<Check class="w-3.5 h-3.5 text-primary shrink-0" />{/if}
                                            </button>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/if}

                        <button
                                onclick={() => showSettings = !showSettings}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label="Settings"
                        >
                            <Settings class="w-5 h-5" />
                        </button>

                        <button
                                onclick={toggleFullscreen}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
                        >
                            {#if isFullscreen}
                                <Minimize class="w-5 h-5" />
                            {:else}
                                <Maximize class="w-5 h-5" />
                            {/if}
                        </button>
                    </div>
                </div>

            </div>
        </div>
    {:else}
        <!-- Mobile Layout Placeholder -->
    {/if}

</div>