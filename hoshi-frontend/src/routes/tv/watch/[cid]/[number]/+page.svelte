<script lang="ts">
    import { onDestroy, onMount, tick, untrack } from 'svelte';
    import { fade } from 'svelte/transition';
    import { FastForward, Rewind } from 'lucide-svelte';
    import { goto } from '$app/navigation';

    import { WatchState } from '@/app/watch.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import TorrentBanner from '@/components/player/TorrentBanner.svelte';
    import {baseTrackLabel, buildTrackLabels, qualityLabel, type TvPanelId} from "@/tv/components/watch/TvPlayerUtils.js";
    import TvPlayerStatus from "@/tv/components/watch/TvPlayerStatus.svelte";
    import TvPlayerControls from "@/tv/components/watch/TvPlayerControls.svelte";
    import TvEpisodePanel from "@/tv/components/watch/TvEpisodePanel.svelte";
    import TvSidePanel from "@/tv/components/watch/TvSidePanel.svelte";
    import TvOptionList from "@/tv/components/watch/TvOptionList.svelte";
    import TvTorrentPicker from "@/tv/components/watch/TvTorrentPicker.svelte";

    const ps = new WatchState();

    onDestroy(() => {
        clearHide();
        if (toastTimer) clearTimeout(toastTimer);
        ps.destroy();
    });

    // ------------------------------------------------------------ panels
    let panel = $state<TvPanelId | null>(null);

    function openPanel(p: TvPanelId) {
        panel = p;
        if (p === 'torrent') ps.showTorrentPicker = true;
    }

    function closePanel() {
        panel = null;
        ps.showTorrentPicker = false;
    }

    // WatchState asks for the picker itself when no torrent matched.
    $effect(() => {
        if (ps.showTorrentPicker && panel !== 'torrent') panel = 'torrent';
    });

    const panelTitle = $derived.by(() => {
        switch (panel) {
            case 'episodes': return i18n.t('content.episodes_title');
            case 'audio': return i18n.t('watch.settings.audio_track');
            case 'subtitles': return i18n.t('watch.settings.subtitles');
            case 'quality': return i18n.t('watch.settings.quality');
            case 'source': return i18n.t('watch.settings.source');
            case 'server': return i18n.t('watch.settings.server');
            case 'torrent': return i18n.t('watch.torrent_pick_title');
            default: return '';
        }
    });

    // Track option lists (labels shared with the desktop settings menu logic)
    const audioLabels = $derived(buildTrackLabels(ps.audioTracks));
    const subLabels = $derived(buildTrackLabels(ps.subtitleTracks));

    const audioOptions = $derived(
        ps.audioTracks.map((t) => ({ id: String(t.id), label: audioLabels.get(t.id) ?? baseTrackLabel(t) }))
    );
    const audioActive = $derived(String(ps.audioTracks.find((t) => t.selected)?.id ?? ''));

    const subOptions = $derived([
        { id: 'off', label: i18n.t('watch.settings.off') },
        ...ps.subtitleTracks.map((t) => ({ id: String(t.id), label: subLabels.get(t.id) ?? baseTrackLabel(t) }))
    ]);
    const subActive = $derived.by(() => {
        const sel = ps.subtitleTracks.find((t) => t.selected);
        return sel ? String(sel.id) : 'off';
    });

    const qualityOptions = $derived(ps.videoTracks.map((t) => ({ id: String(t.id), label: qualityLabel(t) })));
    const qualityActive = $derived(String(ps.videoTracks.find((t) => t.selected)?.id ?? ''));

    const sourceOptions = $derived(ps.extensionItems.map((i) => ({ id: i.value, label: i.label })));
    const serverOptions = $derived(ps.serverItems.map((i) => ({ id: i.value, label: i.label })));

    function choose(action: () => unknown) {
        closePanel();
        void action();
    }

    // ------------------------------------------------------------ controls visibility
    const HIDE_DELAY_MS = 4000;
    let showControls = $state(true);
    let controlsEl = $state<HTMLElement | undefined>();
    let hideTimer: ReturnType<typeof setTimeout> | null = null;

    function clearHide() {
        if (hideTimer) {
            clearTimeout(hideTimer);
            hideTimer = null;
        }
    }

    function hideControls() {
        if (!showControls) return;
        showControls = false;
        (document.activeElement as HTMLElement | null)?.blur?.();
    }

    function scheduleHide() {
        clearHide();
        hideTimer = setTimeout(hideControls, HIDE_DELAY_MS);
    }

    async function revealControls() {
        const wasHidden = !showControls;
        showControls = true;
        scheduleHide();
        if (wasHidden) {
            await tick();
            controlsEl?.querySelector<HTMLElement>('[data-autofocus]')?.focus();
        }
    }

    // Controls stay up while paused, scrubbing, loading or with a panel open; otherwise idle-hide.
    $effect(() => {
        const hold =
            ps.isPaused || ps.isScrubbing || !!panel || ps.isLoadingMeta || ps.isLoadingPlay;
        if (ps.error) {
            clearHide();
            untrack(hideControls);
        } else if (hold) {
            clearHide();
            untrack(() => {
                if (!showControls) void revealControls();
            });
            clearHide();
        } else {
            untrack(scheduleHide);
        }
    });

    // ------------------------------------------------------------ hidden-controls quick seek
    let seekAcc = $state(0);
    let seekToast = $state(false);
    let toastTimer: ReturnType<typeof setTimeout> | null = null;

    function quickSeek(dir: 1 | -1) {
        const step = appConfig.data?.player.seekStep ?? 10;
        seekAcc += dir * step;
        seekToast = true;
        void ps.seekRelative(dir * step);
        if (toastTimer) clearTimeout(toastTimer);
        toastTimer = setTimeout(() => {
            seekToast = false;
            seekAcc = 0;
        }, 900);
    }

    // ------------------------------------------------------------ navigation / keys
    function leave() {
        goto(`/tv/c/${ps.cid}`);
    }

    const isBackKey = (e: KeyboardEvent) =>
        e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;

    // Swallow the matching key-up too, so a handled Back can't still reach the system on release.
    function onKeyup(e: KeyboardEvent) {
        if (!isBackKey(e)) return;
        e.preventDefault();
        e.stopImmediatePropagation();
    }

    function onKeydown(e: KeyboardEvent) {
        const k = e.key;
        const isBack = isBackKey(e);
        // stopImmediatePropagation also keeps the layout's Back handler and spatialNav out of it.
        const consume = () => {
            e.preventDefault();
            e.stopImmediatePropagation();
        };

        if (isBack) {
            // Temporary: if this never prints when you press Back, the key isn't reaching the page.
            console.debug('[tv-watch] back', { key: k, keyCode: e.keyCode, panel, showControls });
            consume();
            if (panel) closePanel();
            else if (ps.error) leave();
            else if (showControls) hideControls();
            else leave();
            return;
        }

        // Remote media keys work everywhere.
        switch (k) {
            case 'MediaPlayPause':
            case 'MediaPlay':
            case 'MediaPause':
                consume();
                void ps.togglePlay();
                return;
            case 'MediaFastForward':
                consume();
                quickSeek(1);
                return;
            case 'MediaRewind':
                consume();
                quickSeek(-1);
                return;
            case 'MediaTrackNext':
                consume();
                ps.goToEpisode(ps.epNumber + 1);
                return;
            case 'MediaTrackPrevious':
                consume();
                ps.goToEpisode(ps.epNumber - 1);
                return;
        }

        if (panel || ps.error) return; // panel / error card use normal focus navigation

        if (!showControls) {
            // Controls hidden: the whole remote acts on the video.
            if (k === 'ArrowLeft') {
                consume();
                quickSeek(-1);
            } else if (k === 'ArrowRight') {
                consume();
                quickSeek(1);
            } else if (k === 'Enter' || k === ' ') {
                consume();
                if (ps.manualSkipChapter) ps.executeSkip(ps.manualSkipChapter);
                else void ps.togglePlay();
            } else if (k === 'ArrowUp' || k === 'ArrowDown') {
                consume();
                void revealControls();
            }
            return;
        }

        // Controls visible: any key counts as activity.
        if (!ps.isPaused && !ps.isScrubbing) scheduleHide();
    }

    onMount(() => {
        window.addEventListener('keydown', onKeydown, true);
        window.addEventListener('keyup', onKeyup, true);
        return () => {
            window.removeEventListener('keydown', onKeydown, true);
            window.removeEventListener('keyup', onKeyup, true);
        };
    });

    const skipLabel = $derived(ps.currentSkippable?.type === 'outro' ? 'Skip outro' : 'Skip intro');
</script>

<svelte:head>
    <title>{ps.animeTitle ? `${ps.animeTitle} - ${ps.episodeTitle}` : i18n.t('watch.title_fallback')}</title>
</svelte:head>

<!-- Transparent on purpose: the video is rendered natively behind the webview. -->
<div class="dark fixed inset-0 select-none overflow-hidden text-white {showControls ? '' : 'cursor-none'}">
    <!-- Header -->
    <div
            class="pointer-events-none absolute inset-x-0 top-0 z-30 bg-gradient-to-b from-black/80 to-transparent px-12 pb-16 pt-8
                   transition-opacity duration-300 {showControls ? 'opacity-100' : 'opacity-0'}"
    >
        <div class="max-w-xl space-y-1">
            <h1 class="line-clamp-2 text-3xl font-black leading-tight tracking-tight">
                {ps.animeTitle || (ps.isLoadingMeta ? i18n.t('watch.loading_metadata') : '')}
            </h1>
            {#if ps.episodeTitle}
                <p class="line-clamp-1 text-lg font-semibold text-white/80">{ps.episodeTitle}</p>
            {/if}
        </div>
    </div>

    {#if ps.selectedTorrent && ps.torrentSessionId}
        <TorrentBanner
                torrent={ps.selectedTorrent}
                sessionId={ps.torrentSessionId}
                visible={showControls}
                onChange={() => openPanel('torrent')}
        />
    {/if}

    <TvPlayerStatus pageState={ps} onleave={leave} />

    <!-- Quick-seek toast (controls hidden) -->
    {#if seekToast}
        <div
                class="pointer-events-none absolute left-1/2 top-1/2 z-40 flex -translate-x-1/2 -translate-y-1/2 items-center gap-3 rounded-2xl bg-black/70 px-8 py-4 text-3xl font-black tabular-nums"
                transition:fade={{ duration: 120 }}
        >
            {#if seekAcc < 0}<Rewind class="size-8" />{:else}<FastForward class="size-8" />{/if}
            {seekAcc > 0 ? '+' : ''}{seekAcc}s
        </div>
    {/if}

    <!-- Skip intro / outro: reachable with OK while the controls are hidden -->
    {#if ps.manualSkipChapter}
        <button
                type="button"
                data-sn={showControls ? '' : undefined}
                onclick={() => ps.executeSkip(ps.manualSkipChapter)}
                class="absolute bottom-64 right-12 z-40 flex h-14 items-center gap-3 rounded-lg border-2 border-white/30 bg-black/70 px-7 text-lg font-bold outline-none
                       focus:scale-105 focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60
                       {showControls ? '' : 'translate-y-48'}"
        >
            {skipLabel}
            {#if !showControls}<span class="rounded bg-white/20 px-2 py-0.5 text-xs">OK</span>{/if}
        </button>
    {/if}

    <!-- Bottom controls. data-sn-rows: Down from the seek bar reaches the button row. -->
    <div
            bind:this={controlsEl}
            inert={!showControls}
            data-sn-rows
            class="absolute inset-x-0 bottom-0 z-30 bg-gradient-to-t from-black/90 via-black/60 to-transparent px-12 pb-12 pt-28
                   transition-[opacity,visibility] duration-300 {showControls ? 'opacity-100' : 'invisible opacity-0'}"
    >
        <TvPlayerControls pageState={ps} onopen={openPanel} />
    </div>

    <!-- Side panels -->
    {#if panel}
        <TvSidePanel title={panelTitle} wide={panel === 'episodes' || panel === 'torrent'}>
            {#if panel === 'episodes'}
                <TvEpisodePanel pageState={ps} onselect={closePanel} />
            {:else if panel === 'torrent'}
                <TvTorrentPicker pageState={ps} onpick={closePanel} />
            {:else if panel === 'audio'}
                <TvOptionList
                        options={audioOptions}
                        activeId={audioActive}
                        onselect={(id) => choose(() => ps.setAudioTrack(Number(id)))}
                />
            {:else if panel === 'subtitles'}
                <TvOptionList
                        options={subOptions}
                        activeId={subActive}
                        onselect={(id) => choose(() => ps.setSubtitleTrack(id === 'off' ? null : Number(id)))}
                />
            {:else if panel === 'quality'}
                <TvOptionList
                        options={qualityOptions}
                        activeId={qualityActive}
                        onselect={(id) => choose(() => ps.setVideoTrack(Number(id)))}
                />
            {:else if panel === 'source'}
                <TvOptionList
                        options={sourceOptions}
                        activeId={ps.selectedExtension}
                        onselect={(id) => choose(() => ps.selectExtension(id))}
                />
            {:else if panel === 'server'}
                <TvOptionList
                        options={serverOptions}
                        activeId={ps.selectedServer}
                        onselect={(id) => choose(() => ps.selectServer(id))}
                />
            {/if}
        </TvSidePanel>
    {/if}
</div>