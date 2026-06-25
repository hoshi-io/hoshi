<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import type { PlayerController } from './PlayerController.svelte.js';
    import type { SubtitleSettings } from './subtitles/SubtitleSettings.svelte.js';
    import SubtitleOverlay from './subtitles/SubtitleOverlay.svelte';
    import {invoke} from "@tauri-apps/api/core";
    import {layoutState} from "@/stores/layout.svelte";

    interface Props {
        src:        string;
        controller: PlayerController;
        subtitleSettings: SubtitleSettings;
    }

    let { src, controller, subtitleSettings }: Props = $props();

    let videoEl: HTMLVideoElement;

    onMount(() => {
        controller.attachVideo(videoEl);

        if (layoutState.isMobile) controller.toggleFullscreen();
    });

    $effect(() => {
        if (src) controller.loadSrc(src);
    });

    onDestroy(() => {
        invoke('exit_fullscreen').catch(() => {});
        controller.destroy()
    });
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
        bind:this={videoEl}
        class="video-el"
        width="1920"
        height="1080"
        oncanplay={() => controller.onCanPlay()}
        ontimeupdate={() => controller.onTimeUpdate()}
        onended={() => controller.onEnded()}
        onprogress={() => controller.onProgress()}
        onwaiting={() => controller.onWaiting()}
        onplaying={() => controller.onPlaying()}
        playsinline
        crossorigin="anonymous"
>
    {#if controller.isReady}
        {#each controller.subtitleTracks as sub (sub.id)}
            <track kind="subtitles" src={sub.url} srclang={sub.srclang} label={sub.label} />
        {/each}
    {/if}
</video>

<SubtitleOverlay {controller} settings={subtitleSettings} />
<style>
    .video-el {
        width: 100%;
        height: 100%;
        object-fit: contain;
        display: block;
        background: #000;
        object-position: center;
        aspect-ratio: 16 / 9;
    }
</style>