<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { RefreshCw } from 'lucide-svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Spinner } from '@/components/ui/spinner';

    let {
        pageState,
        onpick
    }: {
        pageState: WatchState;
        onpick: () => void;
    } = $props();

    // No text input on TV: the picker always uses the auto-generated query.
    const query = pageState.torrentSearchQuery();

    let results = $state<any[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let searched = $state(false);
    let seq = 0;

    async function search() {
        const q = query.trim();
        if (!q || !pageState.selectedExtension) {
            searched = true;
            return;
        }
        const mine = ++seq;
        loading = true;
        error = null;
        try {
            const res = await invoke<{ results: any[] }>('search_torrents', {
                id: pageState.selectedExtension,
                query: q,
                filters: {},
                page: 1
            });
            if (mine !== seq) return;
            results = [...res.results].sort((a, b) => (b.seeders ?? 0) - (a.seeders ?? 0));
        } catch (e: any) {
            if (mine !== seq) return;
            error = e?.key ? i18n.t(e.key) : i18n.t('errors.unknown_error');
            results = [];
        } finally {
            if (mine === seq) {
                loading = false;
                searched = true;
            }
        }
    }

    onMount(() => {
        void search();
    });

    function pick(t: any) {
        onpick();
        void pageState.chooseTorrent(t);
    }
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
        <p class="min-w-0 flex-1 truncate text-sm font-semibold text-white/60">{query}</p>
        <button
                type="button"
                data-sn
                disabled={loading}
                onclick={search}
                class="flex h-11 shrink-0 items-center gap-2 rounded-lg bg-white/10 px-4 text-sm font-bold outline-none
                       focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60 disabled:opacity-40"
        >
            <RefreshCw class="size-4" />
            {i18n.t('watch.torrent_search')}
        </button>
    </div>

    {#if loading}
        <div class="flex justify-center py-10"><Spinner class="size-10 animate-spin text-primary" /></div>
    {:else if error}
        <p class="py-8 text-center text-base text-destructive">{error}</p>
    {:else if searched && results.length === 0}
        <p class="py-8 text-center text-base text-white/60">{i18n.t('watch.torrent_no_results')}</p>
    {:else}
        <ul class="flex flex-col gap-2">
            {#each results as t (t.id)}
                <li>
                    <button
                            type="button"
                            data-sn
                            onclick={() => pick(t)}
                            class="w-full rounded-lg bg-white/5 p-4 text-left outline-none focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60"
                    >
                        <p class="line-clamp-2 text-base font-semibold leading-snug">{t.title}</p>
                        <div class="mt-1.5 flex flex-wrap gap-x-4 text-sm opacity-70">
                            {#if t.releaseGroup}<span>{t.releaseGroup}</span>{/if}
                            {#if t.resolution}<span>{t.resolution}</span>{/if}
                            {#if t.size}<span>{t.size}</span>{/if}
                            <span>S {t.seeders ?? 0}</span>
                            <span>L {t.leechers ?? 0}</span>
                        </div>
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</div>