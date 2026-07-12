<script lang="ts">
    import Play from '@/components/player/controls/buttons/Play.svelte';
    import TimeBar from './TimeBar.svelte';
    import TimeDisplay from './TimeDisplay.svelte';
    import Seek from '@/components/player/controls/buttons/Seek.svelte';
    import Volume from '@/components/player/controls/buttons/Volume.svelte';
    import Fullscreen from '@/components/player/controls/buttons/Fullscreen.svelte';
    import Subtitles from '@/components/player/controls/buttons/Subtitles.svelte';
    import { Settings as SettingsIcon } from 'lucide-svelte';
    import { appConfig } from "@/stores/config.svelte";
    import type { PlayerController } from '../PlayerController.svelte.js';
    import type { PlayerState } from "@/app/watch.svelte.js";
    import { layoutState } from "@/stores/layout.svelte";
    import Settings from "@/components/player/controls/buttons/Settings.svelte";
    import type { SubtitleSettings } from "@/components/player/subtitles/SubtitleSettings.svelte.js";

    interface Props {
        ctrl:               PlayerController;
        playerState:        PlayerState;
        subtitleSettings:   SubtitleSettings;
        visible:            boolean;
        fullscreenEl:       HTMLElement | undefined;
        onManageExtensions: () => void;
    }

    let { ctrl, playerState, subtitleSettings, visible, fullscreenEl, onManageExtensions }: Props = $props();

    const hasError = $derived(!!playerState.error || !!ctrl.hlsError);
    const seekStep = $derived(appConfig.data?.player?.seekStep ?? 10);

    let settingsOpen = $state(false);

    function toggleSettings(e: MouseEvent) {
        e.stopPropagation();
        settingsOpen = !settingsOpen;
    }

    $effect(() => { if (!visible) settingsOpen = false; });
</script>

{#snippet settingsPopover(bgClass: string)}
    <div class="relative">
        <button
                class="settings-btn flex items-center justify-center w-9 h-9 rounded-md
                   {bgClass} text-white/75 cursor-pointer
                   transition-colors duration-200
                   hover:bg-white/15 hover:text-white
                   {settingsOpen ? 'bg-white/20 text-white' : ''}"
                onclick={toggleSettings}
                title="Settings"
                aria-label="Stream settings"
        >
            <SettingsIcon
                    class="w-5 h-5 transition-transform duration-300 ease-in-out
                       {settingsOpen ? 'rotate-[60deg]' : 'rotate-0'}"
            />
        </button>
        <Settings
                {ctrl}
                {playerState}
                {subtitleSettings}
                open={settingsOpen}
                {fullscreenEl}
                {onManageExtensions}
                onClose={() => settingsOpen = false}
        />
    </div>
{/snippet}

{#if layoutState.isMobile}
    <div
            class="controls-root absolute inset-0 z-50 transition-opacity duration-300
               {visible ? 'visible opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}"
    >
        {#if !hasError}
            <!-- Center: seek-back | play/pause | seek-forward -->
            <div class="absolute inset-0 flex items-center justify-center gap-12 pointer-events-none">
                <div class="pointer-events-auto">
                    <Seek seconds={-seekStep} onclick={() => ctrl.seekBy(-seekStep)} />
                </div>
                <div class="pointer-events-auto">
                    <Play paused={ctrl.paused} onclick={() => ctrl.togglePlay()} size="xl" />
                </div>
                <div class="pointer-events-auto">
                    <Seek seconds={seekStep} onclick={() => ctrl.seekBy(seekStep)} />
                </div>
            </div>

            <!-- Bottom: gradient + scrubber row -->
            <div class="absolute inset-x-0 bottom-0">
                <div class="absolute inset-0 pointer-events-none bg-gradient-to-t from-black/80 via-black/30 to-transparent"></div>
                <div class="relative flex flex-col gap-1 px-4 pb-4 pt-8">
                    <div class="flex items-center gap-3 w-full px-2 mb-2">
                        <TimeDisplay currentTime={ctrl.currentTime} duration={ctrl.duration} mode="current" />

                        <div class="flex-1 py-2">
                            <TimeBar
                                    currentTime={ctrl.currentTime}
                                    duration={ctrl.duration}
                                    buffered={ctrl.buffered}
                                    chapters={playerState.chapters}
                                    onSeek={(t) => ctrl.seek(t)}
                            />
                        </div>

                        <TimeDisplay currentTime={ctrl.currentTime} duration={ctrl.duration} mode="total" />
                    </div>
                </div>
            </div>
        {:else}
            <div class="absolute top-4 right-4 pointer-events-auto">
                {@render settingsPopover('bg-black/50')}
            </div>
        {/if}
    </div>

{:else}
    <div
            class="controls-root absolute inset-x-0 bottom-0 z-50 w-full transition-opacity duration-300
               {visible ? 'visible opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}"
            onclick={(e) => e.stopPropagation()}
    >
        <div class="absolute inset-0 pointer-events-none bg-gradient-to-t from-black/80 via-black/40 to-transparent"></div>

        <div class="relative flex flex-col gap-1 px-4 pb-3">
            {#if !hasError}
                <TimeBar
                        currentTime={ctrl.currentTime}
                        duration={ctrl.duration}
                        buffered={ctrl.buffered}
                        chapters={playerState.chapters}
                        onSeek={(t) => ctrl.seek(t)}
                />
            {/if}

            <div class="flex items-center justify-between gap-3 pt-2">
                <div class="flex items-center gap-3">
                    {#if !hasError}
                        <Play paused={ctrl.paused} onclick={() => ctrl.togglePlay()} />
                        <Volume
                                volume={ctrl.volume}
                                muted={ctrl.muted}
                                onVolumeChange={(v) => ctrl.setVolume(v)}
                                onToggleMute={() => ctrl.toggleMute()}
                        />
                        <TimeDisplay currentTime={ctrl.currentTime} duration={ctrl.duration} />
                        <Seek seconds={-seekStep} onclick={() => ctrl.seekBy(-seekStep)} />
                        <Seek seconds={seekStep} onclick={() => ctrl.seekBy(seekStep)} />
                    {/if}
                </div>

                <div class="relative flex items-center gap-2">
                    {#if !hasError && ctrl.subtitleTracks.length > 0}
                        <Subtitles {ctrl} />
                    {/if}
                    {@render settingsPopover('bg-transparent')}
                    <Fullscreen onclick={() => ctrl.toggleFullscreen()} />
                </div>
            </div>
        </div>
    </div>
{/if}