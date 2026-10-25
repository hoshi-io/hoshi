<script lang="ts">
    import { AlertCircle, ArrowLeft, Magnet, RefreshCw } from 'lucide-svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Spinner } from '@/components/ui/spinner';

    let {
        pageState,
        onleave
    }: {
        pageState: WatchState;
        onleave: () => void;
    } = $props();

    let box = $state<HTMLElement | undefined>();

    const loading = $derived((pageState.isLoadingMeta || pageState.isLoadingPlay) && !pageState.error);

    // The error card appears on its own, so it takes focus (Retry first).
    $effect(() => {
        if (!pageState.error) return;
        requestAnimationFrame(() => box?.querySelector<HTMLElement>('[data-sn]')?.focus());
    });

    const btn =
        'flex h-14 items-center gap-2 rounded-lg bg-white/10 px-6 text-base font-bold text-white outline-none ' +
        'focus:scale-105 focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60';
</script>

{#if loading}
    <div class="pointer-events-none absolute inset-0 z-20 flex items-center justify-center">
        <Spinner class="size-16 animate-spin text-white/70" />
    </div>
{:else if pageState.error}
    <div class="absolute inset-0 z-40 flex items-center justify-center bg-black/70">
        <div bind:this={box} class="flex max-w-xl flex-col items-center gap-6 text-center">
            <AlertCircle class="size-16 text-destructive/80" />
            <p class="text-2xl font-bold">{i18n.t(pageState.error.key || 'errors.unknown_error')}</p>
            <div class="flex flex-wrap justify-center gap-3">
                <button type="button" data-sn class={btn} onclick={() => pageState.loadPlay()}>
                    <RefreshCw class="size-5" />
                    {i18n.t('content.retry')}
                </button>
                {#if pageState.isTorrent}
                    <button type="button" data-sn class={btn} onclick={() => (pageState.showTorrentPicker = true)}>
                        <Magnet class="size-5" />
                        {i18n.t('watch.torrent_change')}
                    </button>
                {/if}
                <button type="button" data-sn class={btn} onclick={onleave}>
                    <ArrowLeft class="size-5" />
                    {i18n.t('watch.back_to_details')}
                </button>
            </div>
        </div>
    </div>
{/if}