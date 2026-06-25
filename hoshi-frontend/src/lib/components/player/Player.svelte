<script lang="ts">
    import { goto } from "$app/navigation";

    import Core from './Core.svelte';
    import Controls from './controls/Controls.svelte';
    import TopBar from './TopBar.svelte';
    import Status from './Status.svelte';
    import { PlayerController } from './PlayerController.svelte.js';
    import { SubtitleSettings } from './subtitles/SubtitleSettings.svelte.js';
    import { SkipForward, Loader2 } from 'lucide-svelte';
    import type { Subtitle, Chapter } from './types.js';
    import type { PlayerState } from "@/app/watch.svelte.js";
    import { layoutState } from "@/stores/layout.svelte";

    export type { Subtitle, Chapter };

    interface Props {
        playerState: PlayerState;
        onPlay?: () => void;
        onManageExtensions: () => void;
    }

    let { playerState, onPlay, onManageExtensions }: Props = $props();

    const ctrl             = new PlayerController();
    const subtitleSettings = new SubtitleSettings();

    ctrl.attachSubtitleSettings(subtitleSettings);

    $effect(() => ctrl.setCallbacks({
        onTimeUpdate: (data) => playerState.onTimeUpdate(data),
        onPlay,
        onPause: () => playerState.onPause(),
        onSeek: (time) => playerState.onSeek(time),
        onEnded: () => playerState.onEnded()
    }));

    $effect(() => { if (rootEl) ctrl.attachRoot(rootEl); });
    $effect(() => ctrl.setSubtitles(playerState.subtitles));
    $effect(() => ctrl.setChapters(playerState.chapters));
    $effect(() => ctrl.setInitialTime(playerState.initialTime));
    $effect(() => { if (playerState.m3u8Url) ctrl.loadSrc(playerState.m3u8Url); });

    $effect(() => {
        function handleKeyDown(e: KeyboardEvent) {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
            if (!playerState.m3u8Url) return;

            switch (e.key) {
                case " ":
                case "k":
                    e.preventDefault();
                    ctrl.nudgeControls();
                    ctrl.togglePlay();
                    break;

                case "ArrowRight":
                case "l":
                    e.preventDefault();
                    ctrl.nudgeControls();
                    ctrl.seekBy(e.shiftKey ? 30 : 10);
                    break;

                case "ArrowLeft":
                case "j":
                    e.preventDefault();
                    ctrl.nudgeControls();
                    ctrl.seekBy(e.shiftKey ? -30 : -10);
                    break;

                case "ArrowUp":
                    e.preventDefault();
                    ctrl.nudgeControls();
                    ctrl.setVolume(ctrl.volume + 0.1);
                    break;

                case "ArrowDown":
                    e.preventDefault();
                    ctrl.nudgeControls();
                    ctrl.setVolume(ctrl.volume - 0.1);
                    break;

                case "m":
                    e.preventDefault();
                    ctrl.toggleMute();
                    break;

                case "f":
                    e.preventDefault();
                    ctrl.toggleFullscreen();
                    break;

                case "s":
                    if (ctrl.showSkipButton) {
                        e.preventDefault();
                        ctrl.skipChapter();
                    }
                    break;
            }
        }

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    });

    let rootEl: HTMLElement;
    const topBarVisible = $derived(!playerState.m3u8Url || ctrl.controlsVisible);

    export function getControlsVisible() { return ctrl.controlsVisible; }
    export function toggleFullscreen() { ctrl.toggleFullscreen(); }
</script>

<div
        bind:this={rootEl}
        id="video-player-fullscreen-root"
        class="player-root relative w-full h-full bg-black overflow-hidden select-none [&:not(:has(.controls-root.visible)):not(:has(.status-overlay))]:cursor-none"
        onmousemove={() => !layoutState.isMobile && ctrl.nudgeControls()}
        onclick={() => {
        if (!playerState.m3u8Url) return;
        if (layoutState.isMobile) {
            ctrl.toggleControls();
        } else {
            ctrl.togglePlay();
        }
    }}
        role="presentation"
>
    {#if playerState.m3u8Url}
        <Core src={playerState.m3u8Url} controller={ctrl} {subtitleSettings} />
    {/if}

    <TopBar
            cid={playerState.cid}
            animeTitle={playerState.animeTitle}
            episodeTitle={playerState.episodeTitle}
            epNumber={playerState.epNumber}
            hasPrev={playerState.hasPrev}
            hasNext={playerState.hasNext}
            visible={topBarVisible}
            onBack={() => { playerState.destroy(); goto(`/c/${playerState.cid}`); }}
            {ctrl}
            extensionItems={playerState.extensionItems}
            bind:selectedExtension={playerState.selectedExtension}
            servers={playerState.servers}
            serverItems={playerState.serverItems}
            bind:selectedServer={playerState.selectedServer}
            supportsDub={playerState.supportsDub}
            bind:isDub={playerState.isDub}
            isLoadingPlay={playerState.isLoadingPlay}
            {subtitleSettings}
            onExtensionChange={(val) => playerState.selectExtension(val)}
            onServerChange={() => playerState.loadPlay()}
            onDubChange={(v) => { playerState.isDub = v; playerState.loadPlay(); }}
            {onManageExtensions}
    />

    <Status
            error={playerState.error}
            isLoadingPlay={playerState.isLoadingPlay}
            isLoadingMeta={playerState.isLoadingMeta}
            noExtensions={!playerState.isLoadingMeta && playerState.extensions.length === 0}
            isMappingError={playerState.isMappingError}
            hlsError={ctrl.hlsError}
            onRetry={() => playerState.loadPlay()}
            {onManageExtensions}
    />

    {#if ctrl.isBuffering && !playerState.isLoadingPlay}
        <div class="absolute inset-0 z-[45] flex items-center justify-center pointer-events-none">
            <Loader2 class="w-10 h-10 animate-spin text-white/80" />
        </div>
    {/if}

    {#if ctrl.showSkipButton}
        <button
                class="absolute bottom-20 right-5 z-[60] flex items-center gap-2 px-4 py-2 text-[13px] font-semibold tracking-wide text-white transition-all duration-200 border border-white/20 rounded-xl bg-black/65 backdrop-blur-md hover:bg-black/85 active:scale-95 animate-in slide-in-from-right-4 fade-in"
                onclick={(e) => { e.stopPropagation(); ctrl.skipChapter(); }}
        >
            <SkipForward class="w-4 h-4 fill-current" />
            {ctrl.skipLabel}
        </button>
    {/if}

    {#if playerState.m3u8Url}
        <Controls
                ctrl={ctrl}
                paused={ctrl.paused}
                currentTime={ctrl.currentTime}
                duration={ctrl.duration}
                buffered={ctrl.buffered}
                chapters={playerState.chapters}
                visible={ctrl.controlsVisible}
                extensionItems={playerState.extensionItems}
                bind:selectedExtension={playerState.selectedExtension}
                servers={playerState.servers}
                serverItems={playerState.serverItems}
                bind:selectedServer={playerState.selectedServer}
                supportsDub={playerState.supportsDub}
                bind:isDub={playerState.isDub}
                isLoadingPlay={playerState.isLoadingPlay}
                onExtensionChange={(val) => playerState.selectExtension(val)}
                onServerChange={() => playerState.loadPlay()}
                onDubChange={(v) => { playerState.isDub = v; playerState.loadPlay(); }}
                {onManageExtensions}
                {subtitleSettings}
                onPlayPause={() => ctrl.togglePlay()}
                onSeek={(t) => ctrl.seek(t)}
                onFullscreen={() => ctrl.toggleFullscreen()}
        />
    {/if}
</div>