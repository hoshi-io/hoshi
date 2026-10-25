<script lang="ts">
    import {
        AudioLines, Captions, Gauge, List, Magnet, Mic2, Pause, Play,
        PuzzleIcon, Server, SkipBack, SkipForward
    } from 'lucide-svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import TvSeekBar from "@/tv/components/watch/TvSeekBar.svelte";
    import {baseTrackLabel, buildTrackLabels, qualityLabel, type TvPanelId} from "@/tv/components/watch/TvPlayerUtils.js";

    let {
        pageState,
        onopen
    }: {
        pageState: WatchState;
        onopen: (panel: TvPanelId) => void;
    } = $props();

    // One text line under the row names the focused button (plus its current value),
    // so the buttons themselves can stay icon-only and fit on a single row.
    let hint = $state('');

    const showServer = $derived(!pageState.isTorrent && pageState.serverItems.length > 0);

    const audioLabels = $derived(buildTrackLabels(pageState.audioTracks));
    const subLabels = $derived(buildTrackLabels(pageState.subtitleTracks));

    const audioNow = $derived.by(() => {
        const t = pageState.audioTracks.find((x) => x.selected);
        return t ? (audioLabels.get(t.id) ?? baseTrackLabel(t)) : '';
    });
    const subsNow = $derived.by(() => {
        const t = pageState.subtitleTracks.find((x) => x.selected);
        return t ? (subLabels.get(t.id) ?? baseTrackLabel(t)) : i18n.t('watch.settings.off');
    });
    const qualityNow = $derived.by(() => {
        const t = pageState.videoTracks.find((x) => x.selected);
        return t ? qualityLabel(t) : i18n.t('watch.settings.auto');
    });
    const sourceNow = $derived(
        pageState.extensionItems.find((i) => i.value === pageState.selectedExtension)?.label ?? ''
    );
    const serverNow = $derived(pageState.selectedServer ?? '');

    const withValue = (label: string, value: string) => (value ? `${label}: ${value}` : label);
</script>

{#snippet btn(
    Icon: any,
    label: string,
    text: string,
    onclick: () => void,
    opts: { disabled?: boolean; pressed?: boolean; fill?: boolean; big?: boolean } = {}
)}
    <button
            type="button"
            data-sn
            disabled={opts.disabled}
            aria-pressed={opts.pressed}
            aria-label={label}
            {onclick}
            onfocus={() => (hint = text)}
            onblur={() => (hint = '')}
            class="grid shrink-0 place-items-center rounded-full outline-none
                   transition-transform duration-150 focus:scale-110 focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60
                   disabled:opacity-30
                   {opts.big ? 'size-16' : 'size-14'}
                   {opts.pressed ? 'bg-primary text-primary-foreground' : 'bg-black/40 text-white'}"
    >
        <Icon class="{opts.big ? 'size-7' : 'size-6'} {opts.fill ? 'fill-current' : ''}" />
    </button>
{/snippet}

<TvSeekBar {pageState} />

<div class="mt-2 flex items-center justify-between gap-6">
    <div class="flex items-center gap-3">
        {@render btn(SkipBack, 'Previous', '', () => pageState.goToEpisode(pageState.epNumber - 1), { disabled: !pageState.hasPrev, fill: true })}
        {#if pageState.isPaused}
            {@render btn(Play, 'Play', '', () => pageState.togglePlay(), { fill: true, big: true })}
        {:else}
            {@render btn(Pause, 'Pause', '', () => pageState.togglePlay(), { fill: true, big: true })}
        {/if}
        {@render btn(SkipForward, 'Next', '', () => pageState.goToEpisode(pageState.epNumber + 1), { disabled: !pageState.hasNext, fill: true })}
    </div>

    <div class="flex items-center justify-end gap-3">
        {#if pageState.animeData}
            {@render btn(List, i18n.t('content.episodes_title'), i18n.t('content.episodes_title'), () => onopen('episodes'))}
        {/if}
        {#if pageState.supportsDub}
            {@render btn(Mic2, i18n.t('watch.settings.dub_audio'), i18n.t('watch.settings.dub_audio'), () => pageState.toggleDub(), { pressed: pageState.isDub })}
        {/if}
        {#if pageState.audioTracks.length > 1}
            {@render btn(AudioLines, i18n.t('watch.settings.audio_track'), withValue(i18n.t('watch.settings.audio_track'), audioNow), () => onopen('audio'))}
        {/if}
        {#if pageState.subtitleTracks.length > 0}
            {@render btn(Captions, i18n.t('watch.settings.subtitles'), withValue(i18n.t('watch.settings.subtitles'), subsNow), () => onopen('subtitles'))}
        {/if}
        {#if pageState.videoTracks.length > 1}
            {@render btn(Gauge, i18n.t('watch.settings.quality'), withValue(i18n.t('watch.settings.quality'), qualityNow), () => onopen('quality'))}
        {/if}
        {#if showServer}
            {@render btn(Server, i18n.t('watch.settings.server'), withValue(i18n.t('watch.settings.server'), serverNow), () => onopen('server'))}
        {/if}
        {#if pageState.extensionItems.length > 0}
            {@render btn(PuzzleIcon, i18n.t('watch.settings.source'), withValue(i18n.t('watch.settings.source'), sourceNow), () => onopen('source'))}
        {/if}
        {#if pageState.isTorrent}
            {@render btn(Magnet, i18n.t('watch.torrent_change'), i18n.t('watch.torrent_change'), () => onopen('torrent'))}
        {/if}
    </div>
</div>

<!-- Fixed height so the controls don't jump when the hint appears. -->
<div class="mt-3 h-7 truncate text-right text-lg font-semibold text-white/90" aria-hidden="true">{hint}</div>