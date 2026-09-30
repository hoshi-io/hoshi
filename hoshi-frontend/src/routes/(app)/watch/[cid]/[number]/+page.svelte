<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import * as Drawer from "$lib/components/ui/drawer";
    import { WatchState } from "@/app/watch.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";

    import PlayerHeader from "@/components/player/PlayerHeader.svelte";
    import LoadingError from "@/components/player/LoadingError.svelte";
    import SeekOverlay from "@/components/player/SeekOverlay.svelte";
    import LevelIndicator from "@/components/player/LevelIndicator.svelte";
    import DesktopControls from "@/components/player/DesktopControls.svelte";
    import MobileControls from "@/components/player/MobileControls.svelte";
    import { i18n } from "@/stores/i18n.svelte";
    import SettingsMenu from "@/components/player/settings/SettingsMenu.svelte";
    import {layoutState} from "@/stores/layout.svelte.js";
    import TorrentBanner from "@/components/player/TorrentBanner.svelte";
    import TorrentPickerDialog from "@/components/player/TorrentPickerDialog.svelte";

    const pageState = new WatchState();

    onDestroy(() => {
        pageState.destroy();
    });

    let isFullscreen = $state(false);
    let showSettings = $state(false);
    let showTorrentPicker = $state(false);

    const HIDE_DELAY_DESKTOP_MS = 500;
    const HIDE_DELAY_MOBILE_MS = 3000;
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
        if (pageState.isPaused || showSettings || showTorrentPicker) return;
        const delay = layoutState.isMobile ? HIDE_DELAY_MOBILE_MS : HIDE_DELAY_DESKTOP_MS;
        hideTimer = setTimeout(() => {
            showControls = false;
        }, delay);
    }

    function handleActivity() {
        showControls = true;
        scheduleHide();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (showTorrentPicker) return;
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

    // Visual Brightness/Volume State (mobile swipe gestures)
    let brightness = $state(50);
    let volume = $state(50);
    let showBrightnessOverlay = $state(false);
    let showVolumeOverlay = $state(false);
    let brightnessOverlayTimer: ReturnType<typeof setTimeout> | null = null;
    let volumeOverlayTimer: ReturnType<typeof setTimeout> | null = null;

    // Pixels of vertical swipe needed for a 1% change. Lower = more sensitive.
    const SWIPE_SENSITIVITY = 4;

    function flashBrightnessOverlay() {
        showBrightnessOverlay = true;
        if (brightnessOverlayTimer) clearTimeout(brightnessOverlayTimer);
        brightnessOverlayTimer = setTimeout(() => {
            showBrightnessOverlay = false;
        }, 800);
    }

    function flashVolumeOverlay() {
        showVolumeOverlay = true;
        if (volumeOverlayTimer) clearTimeout(volumeOverlayTimer);
        volumeOverlayTimer = setTimeout(() => {
            showVolumeOverlay = false;
        }, 800);
    }

    async function adjustBrightness(diffPx: number) {
        const delta = diffPx / SWIPE_SENSITIVITY;
        if (!Number.isFinite(delta) || delta === 0) return;
        brightness = Math.min(100, Math.max(0, (Number.isFinite(brightness) ? brightness : 50) + delta));
        flashBrightnessOverlay();
        try {
            await invoke("set_immersive_brightness", { level: brightness / 100 });
        } catch (e) {
            console.error("Failed to set brightness", e);
        }
    }

    async function adjustVolume(diffPx: number) {
        const delta = diffPx / SWIPE_SENSITIVITY;
        if (!Number.isFinite(delta) || delta === 0) return;
        volume = Math.min(100, Math.max(0, (Number.isFinite(volume) ? volume : 50) + delta));
        flashVolumeOverlay();
        try {
            await invoke("set_immersive_volume", { level: volume / 100 });
        } catch (e) {
            console.error("Failed to set volume", e);
        }
    }

    function toggleControlsTap() {
        showControls = !showControls;
        if (showControls) handleActivity();
    }

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

        // diff > 0 means swipe up (increase), diff < 0 means swipe down (decrease)
        if (side === 'left') {
            adjustBrightness(diff);
        } else {
            adjustVolume(diff);
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
        // Seed the indicators with the device's actual current values.
        // Both commands resolve a plain 0-1 float, not an object.
        invoke<number>("get_immersive_brightness")
            .then((level) => { brightness = Math.round(level * 100); })
            .catch((e) => console.error("Failed to get brightness", e));
        invoke<number>("get_immersive_volume")
            .then((level) => { volume = Math.round(level * 100); })
            .catch((e) => console.error("Failed to get volume", e));

        window.addEventListener("keydown", handleKeydown);

        // Desktop only: continuous mouse/wheel/touch activity wakes the
        // controls and resets the auto-hide timer, mirroring hover-driven
        // UIs. On mobile this fights with the deliberate tap-to-toggle
        // logic in handleMobileTap (a touchstart here would flip
        // showControls just before the tap's own click handler flips it
        // again), so mobile relies solely on taps instead.
        if (!layoutState.isMobile) {
            window.addEventListener("mousemove", handleActivity);
            window.addEventListener("mousedown", handleActivity);
            window.addEventListener("touchstart", handleActivity);
            window.addEventListener("wheel", handleActivity);
            scheduleHide();
        }

        return () => {
            window.removeEventListener("keydown", handleKeydown);
            window.removeEventListener("mousemove", handleActivity);
            window.removeEventListener("mousedown", handleActivity);
            window.removeEventListener("touchstart", handleActivity);
            window.removeEventListener("wheel", handleActivity);
            clearHideTimer();
            if (brightnessOverlayTimer) clearTimeout(brightnessOverlayTimer);
            if (volumeOverlayTimer) clearTimeout(volumeOverlayTimer);
        };
    });

    let isFullscreen2 = false;
    let wasMaximizedBeforeFullscreen = false;

    async function toggleFullscreen() {
        try {
            const win = getCurrentWindow();

            if (!isFullscreen2) {
                wasMaximizedBeforeFullscreen = await win.isMaximized();

                if (wasMaximizedBeforeFullscreen) { // some weird bug on windows if we dont do this.
                    await win.unmaximize();
                }

                await win.setFullscreen(true);
                isFullscreen2 = true;
            } else {
                await win.setFullscreen(false);

                if (wasMaximizedBeforeFullscreen) {
                    await win.maximize();
                }

                isFullscreen2 = false;
            }
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

    {#if pageState.manualSkipChapter}
        <div class="absolute bottom-28 right-4 md:bottom-24 md:right-8 z-50 transition-opacity duration-300 {showControls ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}">
            <button
                    class="bg-background/80 hover:bg-background text-foreground px-6 py-2 rounded-md border border-border font-semibold shadow-lg backdrop-blur-md transition-all"
                    onclick={(e) => {
                    e.stopPropagation();
                    pageState.executeSkip(pageState.manualSkipChapter);
                }}
            >
                Skip
            </button>
        </div>
    {/if}
    {#if pageState.selectedTorrent && pageState.torrentSessionId}
        <TorrentBanner
                torrent={pageState.selectedTorrent}
                sessionId={pageState.torrentSessionId}
                visible={showControls}
                onChange={() => showTorrentPicker = true}
        />
    {/if}

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

            <!-- Gesture & Click Invisible Zones. Left/right 30% strips handle
                 double-tap-to-seek and vertical swipe-to-adjust; the middle
                 40% is a plain tap-to-toggle-controls zone with no gestures. -->
            <div class="absolute inset-0 z-10 flex pointer-events-auto">
                <div
                        class="w-[30%] h-full"
                        ontouchstart={handleTouchStart}
                        ontouchmove={(e) => handleTouchMove(e, 'left')}
                        onclick={() => handleMobileTap('left')}
                        role="button"
                        tabindex="0"
                ></div>
                <div
                        class="w-[40%] h-full"
                        onclick={toggleControlsTap}
                        role="button"
                        tabindex="0"
                ></div>
                <div
                        class="w-[30%] h-full"
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

            <LevelIndicator show={showBrightnessOverlay} value={brightness} type="brightness" side="left" />
            <LevelIndicator show={showVolumeOverlay} value={volume} type="volume" side="right" />

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
<TorrentPickerDialog bind:open={showTorrentPicker} {pageState} />