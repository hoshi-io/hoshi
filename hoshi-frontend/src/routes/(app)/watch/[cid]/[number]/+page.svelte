<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { Spinner } from "$lib/components/ui/spinner";
    import * as Drawer from "$lib/components/ui/drawer";
    import {
        Play,
        Pause,
        Settings,
        AlertCircle,
        RotateCw,
        SkipBack,
        SkipForward,
        Maximize,
        Minimize,
        ArrowLeft,
        FastForward,
        Rewind
    } from "lucide-svelte";
    import { scale } from "svelte/transition";
    import { WatchState } from "@/app/watch.svelte.js";
    import SettingsMenu from "@/components/player/SettingsMenu.svelte";
    import TimeBar from "@/components/player/TimeBar.svelte";
    import VolumeControl from "@/components/player/VolumeControl.svelte";
    import { goto } from "$app/navigation";
    import { appConfig } from "@/stores/config.svelte.js";

    let layoutState = $state({ isMobile: false });

    const pageState = new WatchState();

    onDestroy(() => {
        pageState.destroy();
    });

    let isFullscreen = $state(false);
    let showSettings = $state(false);

    const HIDE_DELAY_MS = 500;
    let showControls = $state(true);
    let hideTimer: ReturnType<typeof setTimeout> | null = null;

    function clearHideTimer() {
        if (hideTimer) {
            clearTimeout(hideTimer);
            hideTimer = null;
        }
    }

    function scheduleHide() {
        clearHideTimer();
        if (pageState.isPaused || showSettings) return;
        hideTimer = setTimeout(() => {
            showControls = false;
        }, HIDE_DELAY_MS);
    }

    function handleActivity() {
        showControls = true;
        scheduleHide();
    }

    function handleKeydown(e: KeyboardEvent) {
        const target = e.target as HTMLElement;
        if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable) return;
        if (e.ctrlKey || e.metaKey || e.altKey) return;

        handleActivity();

        switch (e.key.toLowerCase()) {
            case " ":
            case "k":
                e.preventDefault();
                pageState.togglePlay();
                break;
            case "arrowleft":
            case "j":
                e.preventDefault();
                pageState.seekRelative(-(appConfig.data?.player.seekStep ?? 10));
                break;
            case "arrowright":
            case "l":
                e.preventDefault();
                pageState.seekRelative(appConfig.data?.player.seekStep ?? 10);
                break;
            case "arrowup":
                e.preventDefault();
                pageState.setVolume(Math.min(100, pageState.volume + 5));
                break;
            case "arrowdown":
                e.preventDefault();
                pageState.setVolume(Math.max(0, pageState.volume - 5));
                break;
            case "m":
                pageState.toggleMute();
                break;
            case "f":
                toggleFullscreen();
                break;
            case "n":
                if (pageState.hasNext) pageState.goToEpisode(pageState.epNumber + 1);
                break;
            case "p":
                if (pageState.hasPrev) pageState.goToEpisode(pageState.epNumber - 1);
                break;
            case "escape":
                if (showSettings) showSettings = false;
                break;
        }
    }

    // --- MOBILE GESTURE STATE ---
    let lastTapTime = 0;
    let touchStartY = 0;

    // Visual Seek State
    let seekAmount = $state(0);
    let showSeekOverlay = $state(false);
    let seekSide = $state<'left' | 'right'>('right');
    let seekOverlayTimer: ReturnType<typeof setTimeout> | null = null;

    function handleMobileTap(side: 'left' | 'right') {
        const now = Date.now();
        const DOUBLE_TAP_DELAY = 300;

        // Trigger if it's a fast double tap OR if the seek overlay is already active (chaining taps)
        if (now - lastTapTime < DOUBLE_TAP_DELAY || showSeekOverlay) {
            const step = appConfig.data?.player.seekStep ?? 10;
            const delta = side === 'left' ? -step : step;

            // Reset amount if they suddenly switch sides mid-seek
            if (showSeekOverlay && seekSide !== side) {
                seekAmount = 0;
            }

            seekSide = side;
            seekAmount += delta;
            pageState.seekRelative(delta);
            handleActivity();

            showSeekOverlay = true;

            // Reset the hide timer
            if (seekOverlayTimer) clearTimeout(seekOverlayTimer);
            seekOverlayTimer = setTimeout(() => {
                showSeekOverlay = false;
                // Wait for the fade out transition before clearing the number
                setTimeout(() => {
                    if (!showSeekOverlay) seekAmount = 0;
                }, 300);
            }, 800);

            lastTapTime = now;
        } else {
            // Single tap
            showControls = !showControls;
            if (showControls) handleActivity();
            lastTapTime = now;
        }
    }

    function handleTouchStart(e: TouchEvent) {
        touchStartY = e.touches[0].clientY;
    }

    function handleTouchMove(e: TouchEvent, side: 'left' | 'right') {
        const currentY = e.touches[0].clientY;
        const diff = touchStartY - currentY;

        // TODO: Handle slider logic
        // if diff > 0 (swipe up), if diff < 0 (swipe down)
        if (side === 'left') {
            // TODO: adjustBrightness(diff)
        } else {
            // TODO: adjustVolume(diff)
        }

        // Reset start position for continuous dragging feel
        touchStartY = currentY;
    }

    $effect(() => {
        const paused = pageState.isPaused;
        const settingsOpen = showSettings;

        if (paused || settingsOpen) {
            showControls = true;
            clearHideTimer();
        } else {
            scheduleHide();
        }
    });

    onMount(() => {
        // Evaluate initial mobile state
        layoutState.isMobile = window.innerWidth <= 768;

        window.addEventListener("mousemove", handleActivity);
        window.addEventListener("mousedown", handleActivity);
        window.addEventListener("keydown", handleKeydown);
        window.addEventListener("touchstart", handleActivity);
        window.addEventListener("wheel", handleActivity);
        scheduleHide();

        return () => {
            window.removeEventListener("mousemove", handleActivity);
            window.removeEventListener("mousedown", handleActivity);
            window.removeEventListener("keydown", handleKeydown);
            window.removeEventListener("touchstart", handleActivity);
            window.removeEventListener("wheel", handleActivity);
            clearHideTimer();
        };
    });

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

    function formatTime(seconds: number) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
        return `${m}:${s.toString().padStart(2, '0')}`;
    }

</script>

<svelte:head>
    <title>{pageState.animeTitle ? `${pageState.animeTitle} - ${pageState.episodeTitle}` : 'Player'}</title>
</svelte:head>

<div class="relative w-full h-screen overflow-hidden font-sans select-none text-white {showControls ? '' : 'cursor-none'}">

    {#if !layoutState.isMobile}
        <!-- Desktop Player UI Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">

            <!-- TOP BAR: Title & Metadata -->
            <div class="z-20 p-6 bg-gradient-to-b from-black/80 via-black/40 to-transparent flex items-start gap-4 transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <button
                        onclick={() => goto(`/c/${pageState.cid}`)}
                        class="p-2 mt-0.5 rounded-lg bg-black/20 hover:bg-white/15 transition text-white/80 hover:text-white backdrop-blur-sm shrink-0"
                        aria-label="Back to details"
                >
                    <ArrowLeft class="w-5 h-5" />
                </button>
                <div class="space-y-0.5 max-w-2xl">
                    <h1 class="text-xl font-bold tracking-tight text-white/95 drop-shadow-sm leading-tight">
                        {pageState.animeTitle || (pageState.isLoadingMeta ? 'Loading metadata...' : '')}
                    </h1>
                    {#if pageState.episodeTitle}
                        <p class="text-sm font-medium text-white/70">
                            {pageState.episodeTitle}
                        </p>
                    {/if}
                </div>
            </div>

            <!-- Loading & Error Overlays -->
            {#if pageState.isLoadingMeta || pageState.isLoadingPlay}
                <div class="absolute inset-0 z-30 flex items-center justify-center pointer-events-none">
                    <Spinner class="size-10" />
                </div>
            {/if}
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

            <!-- BOTTOM BAR: Desktop Controls -->
            <div class="z-20 px-6 pb-5 pt-2 bg-gradient-to-t from-black/90 via-black/50 to-transparent flex flex-col gap-0.5 transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <TimeBar {pageState} buffered={pageState.bufferedFraction} />
                <div class="flex items-center justify-between text-white mt-0.5">
                    <div class="flex items-center gap-1">
                        <button onclick={() => pageState.togglePlay()} class="p-2 rounded-lg hover:bg-white/10 transition text-white">
                            {#if pageState.isPaused}
                                <Play class="w-5 h-5 fill-current" />
                            {:else}
                                <Pause class="w-5 h-5 fill-current" />
                            {/if}
                        </button>
                        <button onclick={() => pageState.seekRelative(appConfig.data?.player.seekStep ?? 10)} class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white">
                            <RotateCw class="w-4 h-4" />
                        </button>
                        <VolumeControl {pageState} />
                        <div class="flex items-center ml-2 text-[13px] font-medium tabular-nums tracking-wide leading-none select-none">
                            <span class="text-white/95">{formatTime(pageState.currentTime)}</span>
                            <span class="mx-1.5 text-white/30 font-normal">/</span>
                            <span class="text-white/60">{formatTime(pageState.currentDuration)}</span>
                        </div>
                    </div>
                    <div class="relative flex items-center gap-1">
                        <div class="flex items-center gap-0.5 mr-1 pr-1.5 border-r border-white/10">
                            <button onclick={() => pageState.goToEpisode(pageState.epNumber - 1)} disabled={!pageState.hasPrev} class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 disabled:opacity-30">
                                <SkipBack class="w-4 h-4 fill-current" />
                            </button>
                            <button onclick={() => pageState.goToEpisode(pageState.epNumber + 1)} disabled={!pageState.hasNext} class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 disabled:opacity-30">
                                <SkipForward class="w-4 h-4 fill-current" />
                            </button>
                        </div>
                        {#if showSettings}
                            <SettingsMenu {pageState} isMobile={false} />
                        {/if}
                        <button onclick={() => showSettings = !showSettings} class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white">
                            <Settings class="w-5 h-5 transition-transform duration-300 ease-out {showSettings ? 'rotate-90 text-white' : ''}" />
                        </button>
                        <button onclick={toggleFullscreen} class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white">
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
        <!-- Mobile Layout Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">

            <!-- Gesture & Click Invisible Zones -->
            <div class="absolute inset-0 z-10 flex pointer-events-auto">
                <!-- Left Screen (Brightness / Seek Back) -->
                <div
                        class="flex-1 h-full"
                        ontouchstart={handleTouchStart}
                        ontouchmove={(e) => handleTouchMove(e, 'left')}
                        onclick={() => handleMobileTap('left')}
                        role="button"
                        tabindex="0"
                ></div>

                <!-- Right Screen (Volume / Seek Forward) -->
                <div
                        class="flex-1 h-full"
                        ontouchstart={handleTouchStart}
                        ontouchmove={(e) => handleTouchMove(e, 'right')}
                        onclick={() => handleMobileTap('right')}
                        role="button"
                        tabindex="0"
                ></div>
            </div>

            <!-- TOP BAR -->
            <div class="z-20 p-4 bg-gradient-to-b from-black/80 via-black/40 to-transparent flex items-start justify-between gap-4 transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <div class="flex items-start gap-3">
                    <button
                            onclick={() => goto(`/c/${pageState.cid}`)}
                            class="p-2 mt-0.5 rounded-full bg-black/20 hover:bg-white/15 transition text-white/80 hover:text-white backdrop-blur-sm shrink-0"
                    >
                        <ArrowLeft class="w-5 h-5" />
                    </button>
                    <div class="space-y-0.5">
                        <h1 class="text-base font-bold tracking-tight text-white/95 drop-shadow-sm leading-tight line-clamp-1">
                            {pageState.animeTitle || (pageState.isLoadingMeta ? 'Loading...' : '')}
                        </h1>
                        {#if pageState.episodeTitle}
                            <p class="text-xs font-medium text-white/70 line-clamp-1">
                                {pageState.episodeTitle}
                            </p>
                        {/if}
                    </div>
                </div>
                <!-- Settings Cog Top Right -->
                <button
                        onclick={() => showSettings = true}
                        class="p-2 mt-0.5 rounded-full hover:bg-white/10 transition text-white/90 hover:text-white shrink-0"
                >
                    <Settings class="w-5 h-5" />
                </button>
            </div>
            {#if showSeekOverlay}
                <div class="absolute inset-y-0 {seekSide === 'left' ? 'left-0' : 'right-0'} w-1/2 flex items-center justify-center pointer-events-none z-30">
                    <div
                            transition:scale={{ duration: 200, start: 0.85 }}
                            class="bg-black/50 rounded-full w-24 h-24 flex flex-col items-center justify-center backdrop-blur-md shadow-2xl"
                    >
                        {#if seekSide === 'left'}
                            <Rewind class="w-8 h-8 text-white fill-current mb-1" />
                        {:else}
                            <FastForward class="w-8 h-8 text-white fill-current mb-1" />
                        {/if}
                        <span class="text-white font-bold text-sm tracking-wide">
                            {seekAmount > 0 ? `+${seekAmount}` : seekAmount}s
                        </span>
                    </div>
                </div>
            {/if}

            <!-- MIDDLE CONTROLS: Center Action Buttons -->
            <div class="absolute inset-0 z-20 flex items-center justify-center gap-8 pointer-events-none transition-opacity duration-300 ease-out {showControls ? 'opacity-100' : 'opacity-0'}">
                <button
                        onclick={() => pageState.goToEpisode(pageState.epNumber - 1)}
                        disabled={!pageState.hasPrev}
                        class="p-3 bg-black/40 rounded-full text-white pointer-events-auto disabled:opacity-30 disabled:hidden backdrop-blur-md transition-transform active:scale-95"
                >
                    <SkipBack class="w-8 h-8 fill-current" />
                </button>

                <button
                        onclick={() => pageState.togglePlay()}
                        class="p-5 bg-black/50 rounded-full text-white pointer-events-auto backdrop-blur-md transition-transform active:scale-95"
                >
                    {#if pageState.isPaused}
                        <Play class="w-12 h-12 fill-current ml-1" />
                    {:else}
                        <Pause class="w-12 h-12 fill-current" />
                    {/if}
                </button>

                <button
                        onclick={() => pageState.goToEpisode(pageState.epNumber + 1)}
                        disabled={!pageState.hasNext}
                        class="p-3 bg-black/40 rounded-full text-white pointer-events-auto disabled:opacity-30 disabled:hidden backdrop-blur-md transition-transform active:scale-95"
                >
                    <SkipForward class="w-8 h-8 fill-current" />
                </button>
            </div>

            <!-- BOTTOM BAR: Time & Scrubbing -->
            <div class="z-20 px-4 pb-3 pt-6 bg-gradient-to-t from-black/90 via-black/50 to-transparent flex flex-col transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <div class="flex items-center text-xs font-medium tabular-nums tracking-wide mb-2 ml-1">
                    <span class="text-white/95">{formatTime(pageState.currentTime)}</span>
                    <span class="mx-1.5 text-white/40 font-normal">/</span>
                    <span class="text-white/70">{formatTime(pageState.currentDuration)}</span>
                </div>
                <TimeBar {pageState} buffered={pageState.bufferedFraction} />
            </div>

        </div>

        <!-- Mobile Drawer Settings -->
        <Drawer.Root bind:open={showSettings}>
            <Drawer.Content class="bg-zinc-950/95 border-zinc-800 text-white backdrop-blur-xl">
                <!-- Add data-vaul-no-drag here -->
                <div class="px-4 pb-8 overflow-y-auto max-h-[60vh]" data-vaul-no-drag>
                    <SettingsMenu {pageState} isMobile={true} />
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}

</div>