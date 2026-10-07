<script lang="ts">
    import {
        Play, Pause, RotateCw, SkipBack, SkipForward,
        Settings, Maximize, Minimize
    } from "lucide-svelte";
    import TimeBar from "@/components/player/TimeBar.svelte";
    import VolumeControl from "@/components/player/VolumeControl.svelte";
    import SettingsMenu from "@/components/player/settings/SettingsMenu.svelte";
    import EpisodesButton from "@/components/player/EpisodesButton.svelte";
    import { appConfig } from "@/stores/config.svelte.js";

    let {
        pageState,
        isFullscreen,
        showSettings,
        onToggleSettings,
        onToggleEpisodes,
        onToggleFullscreen,
        formatTime
    }: {
        pageState: any;
        isFullscreen: boolean;
        showSettings: boolean;
        onToggleSettings: () => void;
        onToggleEpisodes?: () => void;
        onToggleFullscreen: () => void;
        formatTime: (seconds: number) => string;
    } = $props();
</script>

<div class="z-20 px-6 pb-5 pt-2 flex flex-col gap-0.5">
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
            <button onclick={() => pageState.seekRelative(appConfig.data?.player.seekStep ?? 10)} class="p-2 rounded-lg hover:bg-white/10 transition text-white">
                <RotateCw class="w-4 h-4" />
            </button>
            <VolumeControl {pageState} />
            <div class="flex items-center ml-2 text-[13px] font-medium tabular-nums tracking-wide leading-none select-none">
                <span class="text-white">{formatTime(pageState.currentTime)}</span>
                <span class="mx-1.5 text-white/30 font-normal">/</span>
                <span class="text-white/60">{formatTime(pageState.currentDuration)}</span>
            </div>
        </div>
        <div class="relative flex items-center gap-1">
            <div class="flex items-center gap-0.5 mr-1 pr-1.5 border-r border-white/20">
                <button onclick={() => pageState.goToEpisode(pageState.epNumber - 1)} disabled={!pageState.hasPrev} class="p-2 rounded-lg hover:bg-white/10 transition text-white disabled:opacity-30">
                    <SkipBack class="w-4 h-4 fill-current" />
                </button>
                <button onclick={() => pageState.goToEpisode(pageState.epNumber + 1)} disabled={!pageState.hasNext} class="p-2 rounded-lg hover:bg-white/10 transition text-white disabled:opacity-30">
                    <SkipForward class="w-4 h-4 fill-current" />
                </button>
            </div>
            {#if onToggleEpisodes}
                <EpisodesButton onclick={onToggleEpisodes} />
            {/if}
            {#if showSettings}
                <SettingsMenu {pageState} isMobile={false} />
            {/if}
            <button onclick={onToggleSettings} class="p-2 rounded-lg hover:bg-white/10 transition text-white">
                <Settings class="w-5 h-5 transition-transform duration-300 ease-out {showSettings ? 'rotate-90' : ''}" />
            </button>
            <button onclick={onToggleFullscreen} class="p-2 rounded-lg hover:bg-white/10 transition text-white">
                {#if isFullscreen}
                    <Minimize class="w-5 h-5" />
                {:else}
                    <Maximize class="w-5 h-5" />
                {/if}
            </button>
        </div>
    </div>
</div>