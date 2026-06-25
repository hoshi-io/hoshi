<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { ChevronLeft, SkipBack, SkipForward, Settings as SettingsIcon } from "lucide-svelte";
    import Subtitles from "@/components/player/controls/buttons/Subtitles.svelte";
    import Settings from "@/components/player/controls/buttons/Settings.svelte";
    import type { PlayerController } from "./PlayerController.svelte.js";
    import type { SubtitleSettings } from "@/components/player/subtitles/SubtitleSettings.svelte.js";
    import { layoutState } from "@/stores/layout.svelte";

    interface Props {
        cid:                string;
        animeTitle:         string;
        episodeTitle:       string;
        epNumber:           number;
        hasPrev:            boolean;
        hasNext:            boolean;
        visible:            boolean;
        onBack:             () => void;
        ctrl:               PlayerController;
        extensionItems:     { value: string; label: string }[];
        selectedExtension:  string | null;
        servers:            string[];
        serverItems:        { value: string; label: string }[];
        selectedServer:     string | null;
        supportsDub:        boolean;
        isDub:              boolean;
        isLoadingPlay:      boolean;
        subtitleSettings:   SubtitleSettings;
        onExtensionChange:  (val: string) => void;
        onServerChange:     () => void;
        onDubChange:        (val: boolean) => void;
        onManageExtensions: () => void;
    }

    let {
        cid,
        animeTitle,
        episodeTitle,
        epNumber,
        hasPrev,
        hasNext,
        visible,
        onBack,
        ctrl,
        extensionItems,
        selectedExtension = $bindable(),
        servers,
        serverItems,
        selectedServer = $bindable(),
        supportsDub,
        isDub = $bindable(),
        isLoadingPlay,
        subtitleSettings,
        onExtensionChange,
        onServerChange,
        onDubChange,
        onManageExtensions,
    }: Props = $props();

    let settingsOpen = $state(false);

    function toggleSettings(e: MouseEvent) {
        e.stopPropagation();
        settingsOpen = !settingsOpen;
    }

    $effect(() => {
        if (!visible && layoutState.isMobile && !settingsOpen) {
            settingsOpen = false;
        }
    });
</script>

<div
        class="player-top-bar"
        class:visible
        onclick={(e) => e.stopPropagation()}
        role="toolbar"
        aria-label="Player controls"
>
    <div class="left-group">
        <Button
                variant="secondary"
                size="icon"
                class="back-btn shrink-0"
                onclick={onBack}
        >
            <ChevronLeft class="size-6 text-primary" />
        </Button>

        <div class="title-block">
            <h1 class="anime-title font-heading">{animeTitle}</h1>
            <p class="episode-title">{episodeTitle}</p>
        </div>
    </div>

    <div class="right-group">
        <div class="pill bg-card/40 border-border/50">
            <Button
                    variant="ghost"
                    size="icon"
                    disabled={!hasPrev}
                    href={`/watch/${cid}/${epNumber - 1}`}
                    class="pill-btn"
            >
                <SkipBack class="size-4" />
            </Button>
            <div class="divider bg-border/20"></div>
            <Button
                    variant="ghost"
                    size="icon"
                    disabled={!hasNext}
                    href={`/watch/${cid}/${epNumber + 1}`}
                    class="pill-btn"
            >
                <SkipForward class="size-4" />
            </Button>
        </div>

        {#if layoutState.isMobile}
            <div class="relative flex items-center gap-1">
                {#if ctrl.subtitleTracks.length > 0}
                    <Subtitles {ctrl} />
                {/if}
                <button
                        class="flex items-center justify-center w-9 h-9 rounded-md bg-transparent
                           text-white/80 transition-colors duration-200
                           hover:bg-white/15 hover:text-white
                           {settingsOpen ? 'bg-white/20 text-white' : ''}"
                        onclick={toggleSettings}
                        title="Settings"
                        aria-label="Stream settings"
                >
                    <SettingsIcon class="w-5 h-5" />
                </button>
                <Settings
                        {ctrl}
                        open={settingsOpen}
                        {extensionItems}
                        bind:selectedExtension
                        {servers}
                        {serverItems}
                        bind:selectedServer
                        {supportsDub}
                        bind:isDub
                        {isLoadingPlay}
                        {onExtensionChange}
                        {onServerChange}
                        {onDubChange}
                        {onManageExtensions}
                        onClose={() => settingsOpen = false}
                        {subtitleSettings}
                />
            </div>
        {/if}
    </div>
</div>

<style>
    .player-top-bar {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        z-index: 60;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        padding: max(2rem, env(safe-area-inset-top)) max(1rem, env(safe-area-inset-right)) 1.5rem max(1rem, env(safe-area-inset-left));
        background: linear-gradient(to bottom, rgba(0, 0, 0, 0.9), rgba(0, 0, 0, 0.4) 60%, transparent);
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.3s ease;

        @media (min-width: 640px) {
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
        }
    }

    .player-top-bar.visible {
        opacity: 1;
        pointer-events: auto;
    }

    .left-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        min-width: 0;
        width: 100%;
        @media (min-width: 640px) { width: auto; }
    }

    :global(.back-btn) {
        border-radius: var(--radius-lg);
        background: rgba(0, 0, 0, 0.4) !important;
        backdrop-filter: blur(8px);
        color: white !important;
    }

    .title-block {
        display: flex;
        flex-direction: column;
        min-width: 0;
        filter: drop-shadow(0 1px 2px rgba(0,0,0,0.8));
    }

    .anime-title {
        font-size: 0.875rem;
        line-height: 1.2;
        color: white;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        margin: 0;
        @media (min-width: 640px) { font-size: 1.125rem; }
    }

    .episode-title {
        font-size: 0.625rem;
        font-weight: 700;
        color: var(--color-primary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        margin: 0;
        @media (min-width: 640px) { font-size: 0.75rem; }
    }

    .right-group {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.5rem;
        flex-shrink: 0;
        width: 100%;
        @media (min-width: 640px) { width: auto; }
    }

    .pill {
        display: flex;
        align-items: center;
        border-width: 1px;
        border-radius: var(--radius-lg);
        padding: 0.25rem;
        backdrop-filter: blur(12px);
    }

    :global(.pill-btn) {
        height: 2rem;
        width: 2.25rem;
        color: white !important;
    }
    :global(.pill-btn:hover) {
        color: var(--color-primary) !important;
    }

    .divider {
        width: 1px;
        height: 1.25rem;
        margin: 0 0.25rem;
        flex-shrink: 0;
        background: var(--color-border);
        opacity: 0.2;
    }
</style>