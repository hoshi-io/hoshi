<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { Spinner } from "$lib/components/ui/spinner";
    import {
        Play,
        Pause,
        Settings,
        AlertCircle,
        RotateCw,
        SkipBack,
        SkipForward,
        Maximize,
        Minimize, ArrowLeft,
    } from "lucide-svelte";
    import { WatchState } from "@/app/watch.svelte.js";
    import SettingsMenu from "@/components/player/SettingsMenu.svelte";
    import TimeBar from "@/components/player/TimeBar.svelte";
    import VolumeControl from "@/components/player/VolumeControl.svelte";
    import {goto} from "$app/navigation";

    let layoutState = $state({ isMobile: false });

    const pageState = new WatchState();

    onDestroy(() => {
        pageState.destroy();
    });

    // Fullscreen + settings popover
    let isFullscreen = $state(false);
    let showSettings = $state(false);

    // Auto-hide UI on inactivity
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
        // Never hide while paused or while the settings menu is open
        if (pageState.isPaused || showSettings) return;
        hideTimer = setTimeout(() => {
            showControls = false;
        }, HIDE_DELAY_MS);
    }

    function handleActivity() {
        showControls = true;
        scheduleHide();
    }

    // Re-evaluate whenever pause state or settings menu visibility changes
    $effect(() => {
        // reading these makes the effect reactive to their changes
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
        window.addEventListener("mousemove", handleActivity);
        window.addEventListener("mousedown", handleActivity);
        window.addEventListener("keydown", handleActivity);
        window.addEventListener("touchstart", handleActivity);
        window.addEventListener("wheel", handleActivity);
        scheduleHide();

        return () => {
            window.removeEventListener("mousemove", handleActivity);
            window.removeEventListener("mousedown", handleActivity);
            window.removeEventListener("keydown", handleActivity);
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

<div class="relative w-full h-screen bg-transparent overflow-hidden font-sans select-none text-white {showControls ? '' : 'cursor-none'}">

    {#if !layoutState.isMobile}
        <!-- Desktop Player UI Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">

            <!-- TOP BAR: Title & Metadata -->
            <div class="z-20 p-6 bg-gradient-to-b from-black/80 via-black/40 to-transparent flex items-start gap-4 transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <!-- BACK BUTTON -->
                <button
                        onclick={() => goto(`/c/${pageState.cid}`)}
                        class="p-2 mt-0.5 rounded-lg bg-black/20 hover:bg-white/15 transition text-white/80 hover:text-white backdrop-blur-sm shrink-0"
                        aria-label="Back to details"
                >
                    <ArrowLeft class="w-5 h-5" />
                </button>

                <!-- TITLE & EPISODE METADATA -->
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

            <!-- MIDDLE OVERLAY: Center Spinner Loading State -->
            {#if pageState.isLoadingMeta || pageState.isLoadingPlay}
                <div class="absolute inset-0 z-30 flex items-center justify-center pointer-events-none">
                    <Spinner class="size-10" />
                </div>
            {/if}

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
            <div class="z-20 px-6 pb-5 pt-2 bg-gradient-to-t from-black/90 via-black/50 to-transparent flex flex-col gap-0.5 transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">

                <!-- TIMEBAR & CHAPTERS COMPONENT -->
                <TimeBar {pageState} buffered={pageState.bufferedFraction} />

                <!-- CONTROLS ROW -->
                <div class="flex items-center justify-between text-white mt-0.5">
                    <!-- Left: Play/Pause, Seek +10s, Volume, Time -->
                    <div class="flex items-center gap-1">
                        <button
                                onclick={() => pageState.togglePlay()}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white"
                                aria-label={pageState.isPaused ? "Play" : "Pause"}
                        >
                            {#if pageState.isPaused}
                                <Play class="w-5 h-5 fill-current" />
                            {:else}
                                <Pause class="w-5 h-5 fill-current" />
                            {/if}
                        </button>

                        <button
                                onclick={() => pageState.seekRelative(10)}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label="Seek forward 10 seconds"
                                title="Seek forward 10s"
                        >
                            <RotateCw class="w-4 h-4" />
                        </button>

                        <!-- VOLUME CONTROL COMPONENT -->
                        <VolumeControl {pageState} />

                        <!-- TIME DISPLAY -->
                        <div class="flex items-center ml-2 text-[13px] font-medium tabular-nums tracking-wide leading-none select-none">
                            <span class="text-white/95">{formatTime(pageState.currentTime)}</span>
                            <span class="mx-1.5 text-white/30 font-normal">/</span>
                            <span class="text-white/60">{formatTime(pageState.currentDuration)}</span>
                        </div>
                    </div>

                    <!-- Right: Episode Navigation, Settings, Fullscreen -->
                    <div class="relative flex items-center gap-1">
                        <!-- Episode Prev/Next -->
                        <div class="flex items-center gap-0.5 mr-1 pr-1.5 border-r border-white/10">
                            <button
                                    onclick={() => pageState.goToEpisode(pageState.epNumber - 1)}
                                    disabled={!pageState.hasPrev}
                                    class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white disabled:opacity-30 disabled:hover:bg-transparent"
                                    aria-label="Previous episode"
                                    title="Previous episode"
                            >
                                <SkipBack class="w-4 h-4 fill-current" />
                            </button>

                            <button
                                    onclick={() => pageState.goToEpisode(pageState.epNumber + 1)}
                                    disabled={!pageState.hasNext}
                                    class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white disabled:opacity-30 disabled:hover:bg-transparent"
                                    aria-label="Next episode"
                                    title="Next episode"
                            >
                                <SkipForward class="w-4 h-4 fill-current" />
                            </button>
                        </div>

                        {#if showSettings}
                            <SettingsMenu {pageState} />
                        {/if}

                        <button
                                onclick={() => showSettings = !showSettings}
                                class="p-2 rounded-lg hover:bg-white/10 transition text-white/90 hover:text-white"
                                aria-label="Settings"
                        >
                            <Settings class="w-5 h-5 transition-transform duration-300 ease-out {showSettings ? 'rotate-90 text-white' : ''}" />
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