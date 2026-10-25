<script lang="ts">
    import {
        AudioLines, Captions, Gauge, List, Magnet, Mic2, Pause, Play,
        PuzzleIcon, Server, SkipBack, SkipForward
    } from 'lucide-svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import TvSeekBar from "@/tv/watch/TvSeekBar.svelte";
    import type {TvPanelId} from "@/tv/watch/TvPlayerUtils.ts";

    let {
        pageState,
        onopen
    }: {
        pageState: WatchState;
        onopen: (panel: TvPanelId) => void;
    } = $props();

    const showServer = $derived(!pageState.isTorrent && pageState.serverItems.length > 0);
</script>

{#snippet btn(
    Icon: any,
    label: string | null,
    onclick: () => void,
    opts: { disabled?: boolean; pressed?: boolean; fill?: boolean } = {}
)}
    <button
            type="button"
            data-sn
            disabled={opts.disabled}
            aria-pressed={opts.pressed}
            aria-label={label ?? undefined}
            {onclick}
            class="flex h-14 items-center gap-2 rounded-lg px-5 text-base font-bold outline-none
                   transition-transform duration-150 focus:scale-105 focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60
                   disabled:opacity-30
                   {opts.pressed ? 'bg-primary text-primary-foreground' : 'bg-white/10 text-white'}"
    >
        <Icon class="size-6 shrink-0 {opts.fill ? 'fill-current' : ''}" />
        {#if label}<span>{label}</span>{/if}
    </button>
{/snippet}

<TvSeekBar {pageState} />

<div class="mt-2 flex items-center justify-between gap-6">
    <div class="flex items-center gap-3">
        {@render btn(SkipBack, null, () => pageState.goToEpisode(pageState.epNumber - 1), { disabled: !pageState.hasPrev, fill: true })}
        {#if pageState.isPaused}
            {@render btn(Play, null, () => pageState.togglePlay(), { fill: true })}
        {:else}
            {@render btn(Pause, null, () => pageState.togglePlay(), { fill: true })}
        {/if}
        {@render btn(SkipForward, null, () => pageState.goToEpisode(pageState.epNumber + 1), { disabled: !pageState.hasNext, fill: true })}
    </div>

    <div class="flex flex-wrap items-center justify-end gap-3">
        {#if pageState.animeData}
            {@render btn(List, i18n.t('content.episodes_title', { default: 'Episodes' }), () => onopen('episodes'))}
        {/if}
        {#if pageState.supportsDub}
            {@render btn(Mic2, i18n.t('watch.settings.dub_audio'), () => pageState.toggleDub(), { pressed: pageState.isDub })}
        {/if}
        {#if pageState.audioTracks.length > 1}
            {@render btn(AudioLines, i18n.t('watch.settings.audio_track'), () => onopen('audio'))}
        {/if}
        {#if pageState.subtitleTracks.length > 0}
            {@render btn(Captions, i18n.t('watch.settings.subtitles'), () => onopen('subtitles'))}
        {/if}
        {#if pageState.videoTracks.length > 1}
            {@render btn(Gauge, i18n.t('watch.settings.quality'), () => onopen('quality'))}
        {/if}
        {#if showServer}
            {@render btn(Server, i18n.t('watch.settings.server'), () => onopen('server'))}
        {/if}
        {#if pageState.extensionItems.length > 0}
            {@render btn(PuzzleIcon, i18n.t('watch.settings.source'), () => onopen('source'))}
        {/if}
        {#if pageState.isTorrent}
            {@render btn(Magnet, i18n.t('watch.torrent_change'), () => onopen('torrent'))}
        {/if}
    </div>
</div>