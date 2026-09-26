<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import * as Drawer from "$lib/components/ui/drawer";
    import { WatchState } from "@/app/watch.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";

    import PlayerHeader from "@/components/player/PlayerHeader.svelte";
    import LoadingError from "@/components/player/LoadingError.svelte";
    import SeekOverlay from "@/components/player/SeekOverlay.svelte";
    import DesktopControls from "@/components/player/DesktopControls.svelte";
    import MobileControls from "@/components/player/MobileControls.svelte";
	import { i18n } from "@/stores/i18n.svelte";
    import SettingsMenu from "@/components/player/settings/SettingsMenu.svelte";

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
    <title>{pageState.animeTitle ? `${pageState.animeTitle} - ${pageState.episodeTitle}` : i18n.t('watch.title_fallback')}</title>
</svelte:head>

<div class="relative w-full h-screen overflow-hidden font-sans select-none text-foreground {showControls ? '' : 'cursor-none'}">

    {#if !layoutState.isMobile}
        <!-- Desktop Player UI Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">
            <div class="transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <PlayerHeader
                        cid={pageState.cid}
                        title={pageState.animeTitle}
                        episodeTitle={pageState.episodeTitle}
                        isLoadingMeta={pageState.isLoadingMeta}
                        isMobile={false}
                />
            </div>

            <LoadingError
                    isLoading={pageState.isLoadingMeta || pageState.isLoadingPlay}
                    error={pageState.error}
            />

            <div class="transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <DesktopControls
                        {pageState}
                        {isFullscreen}
                        {showSettings}
                        onToggleSettings={() => showSettings = !showSettings}
                        onToggleFullscreen={toggleFullscreen}
                        {formatTime}
                />
            </div>
        </div>

    {:else}
        <!-- Mobile Layout Overlay -->
        <div class="relative w-full h-full flex flex-col justify-between pointer-events-none">

            <!-- Gesture & Click Invisible Zones -->
            <div class="absolute inset-0 z-10 flex pointer-events-auto">
                <div
                        class="flex-1 h-full"
                        ontouchstart={handleTouchStart}
                        ontouchmove={(e) => handleTouchMove(e, 'left')}
                        onclick={() => handleMobileTap('left')}
                        role="button"
                        tabindex="0"
                ></div>
                <div
                        class="flex-1 h-full"
                        ontouchstart={handleTouchStart}
                        ontouchmove={(e) => handleTouchMove(e, 'right')}
                        onclick={() => handleMobileTap('right')}
                        role="button"
                        tabindex="0"
                ></div>
            </div>

            <div class="transition-opacity duration-300 ease-out {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
                <PlayerHeader
                        cid={pageState.cid}
                        title={pageState.animeTitle}
                        episodeTitle={pageState.episodeTitle}
                        isLoadingMeta={pageState.isLoadingMeta}
                        isMobile={true}
                        onSettingsClick={() => showSettings = true}
                />
            </div>

            <SeekOverlay show={showSeekOverlay} side={seekSide} amount={seekAmount} />

            <LoadingError
                    isLoading={pageState.isLoadingMeta || pageState.isLoadingPlay}
                    error={pageState.error}
            />

            <MobileControls {pageState} {showControls} {formatTime} />
        </div>

        <!-- Mobile Drawer Settings -->
        <Drawer.Root bind:open={showSettings}>
            <Drawer.Content class="bg-popover border-border text-popover-foreground z-[100]">
                <div class="px-4 pb-8 overflow-y-auto max-h-[60vh]" data-vaul-no-drag>
                    <SettingsMenu {pageState} isMobile={true} />
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}

</div>