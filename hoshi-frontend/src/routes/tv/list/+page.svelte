<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { goto } from '$app/navigation';
    import { AlertCircle, Compass, List, RefreshCw, Repeat, X } from 'lucide-svelte';

    import { listStore } from '@/app/list.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import LazyCardGrid from '@/components/card/LazyCardGrid.svelte';
    import TvCard from '@/tv/components/card/TvCard.svelte';
    import {openListEditor} from "@/stores/layout.svelte.ts";
    import {getCardTitle} from "@/utils/normalize.ts";

    onMount(() => {
        listStore.loadData(); // no-op when the list is already loaded
    });

    type StatusFilter = typeof listStore.activeStatus;
    type TypeFilter = typeof listStore.activeType;
    type SortOption = typeof listStore.activeSort;

    const statusOptions = $derived<{ value: StatusFilter; label: string }[]>([
        { value: 'ALL', label: i18n.t('list.all') },
        { value: 'CURRENT', label: i18n.t('list.current') },
        { value: 'COMPLETED', label: i18n.t('list.completed') },
        { value: 'PLANNING', label: i18n.t('list.planning') },
        { value: 'PAUSED', label: i18n.t('list.paused') },
        { value: 'DROPPED', label: i18n.t('list.dropped') }
    ]);

    const typeOptions = $derived<{ value: TypeFilter; label: string }[]>([
        { value: 'ALL', label: i18n.t('list.all') },
        { value: 'anime', label: i18n.t('list.anime') },
        { value: 'manga', label: i18n.t('list.manga') },
        { value: 'novel', label: i18n.t('list.novel') }
    ]);

    const sortOptions = $derived<{ value: SortOption; label: string }[]>([
        { value: 'SCORE_DESC', label: i18n.t('list.modal.score') },
        { value: 'TITLE_ASC', label: 'A-Z' },
        { value: 'TITLE_DESC', label: 'Z-A' }
    ]);

    const statusCounts = $derived<Record<StatusFilter, number>>({
        ALL: listStore.entries.length,
        CURRENT: listStore.stats?.watching ?? 0,
        COMPLETED: listStore.stats?.completed ?? 0,
        PLANNING: listStore.stats?.planning ?? 0,
        PAUSED: listStore.stats?.paused ?? 0,
        DROPPED: listStore.stats?.dropped ?? 0
    });

    const filtersActive = $derived(
        listStore.activeStatus !== 'ALL' ||
        listStore.activeType !== 'ALL' ||
        listStore.activeSort !== 'SCORE_DESC'
    );

    const PAGE_SIZE = 30;
    let visibleCount = $state(PAGE_SIZE);

    // Reset paging only when the user changes a filter. A background sync reload replaces
    // `sorted`, and resetting there would yank the scroll position / focus away mid-browse.
    $effect(() => {
        listStore.activeStatus;
        listStore.activeType;
        listStore.activeSort;
        visibleCount = PAGE_SIZE;
    });

    const visibleItems = $derived(listStore.sorted.slice(0, visibleCount));
    const hasNoEntries = $derived(listStore.entries.length === 0);
    const showSkeleton = $derived(listStore.isLoading || (!listStore.isInitialized && !listStore.error));

    const statusLabel = (status: string) =>
        statusOptions.find((o) => o.value === status)?.label ?? status;

    const STATUS_DOT: Record<string, string> = {
        COMPLETED: 'bg-emerald-500',
        CURRENT: 'bg-sky-500',
        PAUSED: 'bg-amber-500',
        DROPPED: 'bg-rose-500',
        REPEATING: 'bg-purple-500',
        PLANNING: 'bg-zinc-400'
    };
</script>

<svelte:head>
    <title>{i18n.t('list.title')}</title>
</svelte:head>

{#snippet chip(label: string, active: boolean, onselect: () => void, count?: number, autofocus = false)}
    <button
            type="button"
            data-sn
            data-autofocus={autofocus ? '' : undefined}
            aria-pressed={active}
            onclick={onselect}
            class="flex h-11 scroll-my-24 items-center gap-2 whitespace-nowrap rounded-lg border-2 px-4 text-sm font-bold outline-none
                   transition-transform duration-150
                   focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                   {active
                ? 'border-primary bg-primary text-primary-foreground'
                : 'border-border/60 bg-card text-foreground'}"
    >
        {label}
        {#if count !== undefined}
            <span class="text-xs font-medium tabular-nums opacity-70">{count}</span>
        {/if}
    </button>
{/snippet}

<!-- data-sn-rows: row-based Up/Down/Left/Right inside this block (see spatialNav.ts). -->
<div class="space-y-6 pb-16" data-sn-rows>
    <header class="flex items-baseline justify-between gap-6">
        <h1 class="text-4xl font-black tracking-tight">{i18n.t('list.title')}</h1>
        {#if !showSkeleton && !listStore.error}
            <span class="text-lg font-bold tabular-nums text-muted-foreground">{listStore.sorted.length}</span>
        {/if}
    </header>

    <!-- Filters stay rendered in every state, so focus is never lost while loading or when a
         filter matches nothing. -->
    <div class="space-y-3">
        <div class="flex flex-wrap items-center gap-2">
            <span class="w-24 shrink-0 text-xs font-bold uppercase tracking-widest text-muted-foreground">
                {i18n.t('list.status')}
            </span>
            {#each statusOptions as option (option.value)}
                {@render chip(
                    option.label,
                    listStore.activeStatus === option.value,
                    () => (listStore.activeStatus = option.value),
                    statusCounts[option.value] ?? 0,
                    listStore.activeStatus === option.value
                )}
            {/each}
        </div>

        <div class="flex flex-wrap items-center gap-2">
            <span class="w-24 shrink-0 text-xs font-bold uppercase tracking-widest text-muted-foreground">
                {i18n.t('list.content_type')}
            </span>
            {#each typeOptions as option (option.value)}
                {@render chip(option.label, listStore.activeType === option.value, () => (listStore.activeType = option.value))}
            {/each}

            <span class="mx-2 h-6 w-px bg-border/60"></span>

            {#each sortOptions as option (option.value)}
                {@render chip(option.label, listStore.activeSort === option.value, () => (listStore.activeSort = option.value))}
            {/each}

            {#if filtersActive}
                <button
                        type="button"
                        data-sn
                        onclick={() => listStore.resetFilters()}
                        class="ml-2 flex h-11 items-center gap-2 rounded-lg px-4 text-sm font-bold text-muted-foreground outline-none
                               focus:bg-secondary focus:text-foreground focus:ring-4 focus:ring-primary"
                >
                    <X class="size-4" />
                    {i18n.t('list.clear_all')}
                </button>
            {/if}
        </div>
    </div>

    {#if showSkeleton}
        <div class="grid grid-cols-5 gap-x-5 gap-y-6 p-3">
            {#each Array(10) as _, i (i)}
                <Skeleton class="aspect-[2/3] w-full rounded-md" />
            {/each}
        </div>

    {:else if listStore.error}
        <div role="alert" class="flex flex-col items-center justify-center gap-6 py-20 text-center" in:fade={{ duration: 200 }}>
            <div class="rounded-full border border-destructive/20 bg-destructive/10 p-6 text-destructive">
                <AlertCircle class="size-14" />
            </div>
            <h2 class="text-2xl font-bold text-destructive">{i18n.t(listStore.error.key)}</h2>
            <button
                    type="button"
                    data-sn
                    onclick={() => listStore.refresh()}
                    class="flex items-center gap-2 rounded-lg border-2 border-destructive/30 px-6 py-3 text-base font-bold text-destructive outline-none
                           focus:bg-destructive focus:text-destructive-foreground focus:ring-4 focus:ring-destructive/40"
            >
                <RefreshCw class="size-5" />
                {i18n.t('content.retry')}
            </button>
        </div>

    {:else if listStore.filtered.length === 0}
        <div role="status" class="flex flex-col items-center justify-center gap-5 py-20 text-center" in:fade={{ duration: 200 }}>
            <div class="rounded-full border border-border/50 bg-card p-6">
                <List class="size-14 text-primary/80" />
            </div>

            {#if hasNoEntries}
                <h2 class="text-2xl font-bold">{i18n.t('list.empty_title')}</h2>
                <p class="max-w-md text-base text-muted-foreground">{i18n.t('list.empty_desc')}</p>
                <button
                        type="button"
                        data-sn
                        onclick={() => goto('/tv')}
                        class="flex items-center gap-2 rounded-lg bg-primary px-6 py-3 text-base font-bold text-primary-foreground outline-none focus:ring-4 focus:ring-primary/40"
                >
                    <Compass class="size-5" />
                    {i18n.t('list.browse')}
                </button>
            {:else}
                <h2 class="text-2xl font-bold">{i18n.t('list.no_matches_title')}</h2>
                <p class="max-w-md text-base text-muted-foreground">{i18n.t('list.no_matches_desc')}</p>
            {/if}
        </div>

    {:else}
        <LazyCardGrid
                items={visibleItems}
                keyFn={(item) => item.original.cid}
                hasMore={visibleCount < listStore.sorted.length}
                isLoading={listStore.isLoading}
                onLoadMore={() => (visibleCount = Math.min(visibleCount + PAGE_SIZE, listStore.sorted.length))}
                gridClass="grid grid-cols-5 gap-x-5 gap-y-6 p-3"
        >
            {#snippet cardContent(item)}
                {@const total = item.original.totalUnits || 0}
                {@const pct = total > 0 ? Math.min(100, ((item.original.progress || 0) / total) * 100) : 0}
                <TvCard
                        card={item.card}
                        widthClass="w-full"
                        onselect={() =>
            openListEditor({
                cid: item.original.cid,
                title: getCardTitle(item.card),
                contentType: String((item.original as any).contentType ?? (item.card as any).contentType ?? 'anime').toLowerCase(),
                coverImage: item.card.cover,
                headers: item.card.imageHeaders
            })}
                >
                    {#snippet overlay()}
                        <div class="space-y-1">
                            <div class="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wide text-zinc-200">
                                <span class="size-2 shrink-0 rounded-full {STATUS_DOT[item.original.status] ?? 'bg-zinc-400'}"></span>
                                <span class="truncate">{statusLabel(item.original.status)}</span>
                                {#if item.original.repeatCount > 0}
                                    <span class="ml-auto flex shrink-0 items-center gap-0.5 text-purple-300">
                                        <Repeat class="size-3" />
                                        {item.original.repeatCount}
                                    </span>
                                {/if}
                            </div>
                            <div class="font-mono text-sm font-black text-white">
                                <span class="text-primary">{item.original.progress}</span>
                                <span class="text-xs font-bold text-zinc-400">/ {total || '—'}</span>
                            </div>
                            {#if total > 0}
                                <div class="h-1 w-full overflow-hidden rounded-full bg-white/20">
                                    <div class="h-full bg-primary" style="width: {pct}%"></div>
                                </div>
                            {/if}
                        </div>
                    {/snippet}
                </TvCard>
            {/snippet}
        </LazyCardGrid>
    {/if}
</div>