<script lang="ts">
    import { onMount, tick, untrack } from 'svelte';
    import { Search, SearchX, ListFilter, X, Blocks } from 'lucide-svelte';
    import { extensionsApi } from '$lib/api/extensions/extensions';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { searchState } from '@/app/search.svelte.js';

    import LazyCardGrid from '@/components/card/LazyCardGrid.svelte';
    import TvCard from '@/tv/TvCard.svelte';
    import TvPanel from '@/tv/TvPanel.svelte';
    import TvSearchSources, { TRACKERS, type Tracker } from '@/tv/TvSearchSources.svelte';
    import TvSearchFilters from '@/tv/TvSearchFilters.svelte';

    // Fixed 9rem columns (= TvCard width), spread evenly across the row.
    const TV_GRID = 'grid grid-cols-[repeat(auto-fill,9rem)] justify-between gap-x-4 gap-y-6';

    let panel = $state<'source' | 'filters' | null>(null);
    let filtersDirty = false;
    let extFiltersSchema = $state<Record<string, any>>({});

    let editing = $state(false);
    let queryAtStart = '';
    let queryButton: HTMLButtonElement | null = $state(null);
    let queryInput: HTMLInputElement | null = $state(null);

    // ---- search plumbing (same behaviour as the desktop page, minus debounce/mobile) ----

    function handleSearch() {
        searchState.page = 1;
        if (searchState.searchMode === 'tracker') searchState.search();
        else searchState.extensionSearch();
    }

    // searchState is a singleton shared with the desktop UI: make sure it's in anime mode.
    onMount(() => {
        if (searchState.contentType !== 'anime') {
            searchState.searchMode = 'tracker';
            searchState.selectedExtension = '';
            searchState.extFilterValues = {};
            searchState.updateContentType('anime');
        }
    });

    $effect(() => {
        const currentExts = searchState.availableExtensions;
        untrack(() => {
            if (
                searchState.searchMode === 'extension' &&
                (!searchState.selectedExtension || !currentExts.find((e) => e.id === searchState.selectedExtension))
            ) {
                if (currentExts.length > 0) {
                    searchState.selectedExtension = currentExts[0].id;
                } else {
                    searchState.selectedExtension = '';
                    searchState.searchMode = 'tracker';
                }
            }

            if (!searchState.hasSearched) handleSearch();
        });
    });

    $effect(() => {
        if (searchState.searchMode === 'extension' && searchState.selectedExtension) {
            extensionsApi
                .getFilters(searchState.selectedExtension)
                .then((res) => {
                    extFiltersSchema = res.filters || {};

                    Object.entries(extFiltersSchema).forEach(([key, def]) => {
                        if (def.type === 'boolean' && searchState.extFilterValues[key] === undefined) {
                            searchState.extFilterValues[key] = false;
                        }
                    });
                })
                .catch(() => {
                    extFiltersSchema = {};
                });
        } else {
            extFiltersSchema = {};
        }
    });

    // ---- sources ----

    function selectSource(mode: 'tracker' | 'extension', extId = '', tracker: Tracker = 'anilist') {
        if (mode === 'extension') {
            // A different extension has a different filter schema.
            if (searchState.selectedExtension !== extId) searchState.extFilterValues = {};
            searchState.selectedExtension = extId;
        } else {
            // This is what the desktop SearchFilters does in an effect keyed on `tracker`;
            // here the panel unmounts, so it lives with the action instead.
            if (searchState.tracker !== tracker) {
                searchState.status = '';
                searchState.genre = '';
                searchState.format = '';
                if (tracker === 'kitsu') searchState.nsfw = false;
            }
            searchState.tracker = tracker;
        }
        searchState.searchMode = mode;

        filtersDirty = false;
        panel = null;
        handleSearch();
    }

    const sourceLabel = $derived(
        searchState.searchMode === 'extension'
            ? (searchState.availableExtensions.find((e) => e.id === searchState.selectedExtension)?.name ?? '—')
            : (TRACKERS.find((t) => t.id === searchState.tracker)?.label ?? '')
    );

    // ---- filters: edit freely, search once when the panel closes ----

    const hasActiveFilters = $derived(
        !!(searchState.status || searchState.genre || searchState.format || searchState.nsfw) ||
        Object.values(searchState.extFilterValues).some((v) => (Array.isArray(v) ? v.length > 0 : !!v))
    );

    function closePanel() {
        const wasFilters = panel === 'filters';
        panel = null;
        if (wasFilters && filtersDirty) {
            filtersDirty = false;
            handleSearch();
        }
    }

    function clearFilters() {
        searchState.clearFilters();
        filtersDirty = true;
    }

    // ---- query field: an input would trap D-pad focus, so it only exists while editing ----

    async function startEdit() {
        queryAtStart = searchState.query;
        editing = true;
        await tick();
        queryInput?.focus();
        queryInput?.select();
    }

    async function stopEdit() {
        if (!editing) return;
        editing = false;
        await tick();
        queryButton?.focus();
        if (searchState.query !== queryAtStart) handleSearch();
    }

    function onQueryKeydown(e: KeyboardEvent) {
        const exit =
            e.key === 'Enter' || e.key === 'Escape' || e.key === 'GoBack' || e.keyCode === 4 ||
            // spatial nav ignores inputs, so Up/Down also leave the field
            e.key === 'ArrowUp' || e.key === 'ArrowDown';
        if (!exit) return;
        e.preventDefault();
        e.stopPropagation();
        stopEdit();
    }

    function clearQuery() {
        searchState.query = '';
        handleSearch();
    }

    const placeholder = $derived(
        i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })
    );

    const bar =
        'flex h-12 items-center gap-2 rounded-md bg-muted/30 px-5 text-base font-semibold text-foreground outline-none ' +
        'transition-[transform,background-color] duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70';
</script>

<svelte:head><title>{i18n.t('search.title')}</title></svelte:head>

<div class="space-y-6">
    <!-- top bar -->
    <div class="flex items-center gap-3">
        {#if editing}
            <input
                    bind:this={queryInput}
                    bind:value={searchState.query}
                    data-sn
                    onkeydown={onQueryKeydown}
                    onblur={stopEdit}
                    {placeholder}
                    class="h-12 min-w-0 flex-1 rounded-md border-2 border-primary bg-secondary/50 px-4 text-lg outline-none"
            />
        {:else}
            <button
                    bind:this={queryButton}
                    type="button"
                    data-sn
                    data-autofocus
                    onclick={startEdit}
                    class="{bar} min-w-0 flex-1 justify-start"
            >
                <Search class="size-5 shrink-0 opacity-70" />
                <span class="truncate {searchState.query ? '' : 'text-muted-foreground'}">
                    {searchState.query || placeholder}
                </span>
            </button>

            {#if searchState.query}
                <button type="button" data-sn aria-label={i18n.t('search.clear')} onclick={clearQuery} class="{bar} px-4">
                    <X class="size-5" />
                </button>
            {/if}
        {/if}

        <button type="button" data-sn onclick={() => (panel = 'source')} class="{bar} max-w-56">
            <Blocks class="size-5 shrink-0 opacity-70" />
            <span class="truncate">{sourceLabel}</span>
        </button>

        <button type="button" data-sn onclick={() => (panel = 'filters')} class="{bar} relative">
            <ListFilter class="size-5 shrink-0 opacity-70" />
            {i18n.t('search.filters')}
            {#if hasActiveFilters}
                <span class="absolute right-2 top-2 size-2 rounded-full bg-primary"></span>
            {/if}
        </button>
    </div>

    <!-- results -->
    {#if searchState.isLoading && searchState.page === 1}
        <div class={TV_GRID}>
            {#each Array(10) as _}
                <div class="w-36 animate-pulse">
                    <div class="aspect-[2/3] rounded-md bg-muted/40"></div>
                    <div class="mt-2 h-3 w-2/3 rounded bg-muted/40"></div>
                    <div class="mt-1.5 h-4 w-5/6 rounded bg-muted/30"></div>
                </div>
            {/each}
        </div>
    {:else if searchState.error}
        <div class="flex min-h-[40dvh] flex-col items-center justify-center gap-6 text-center">
            <h2 class="text-xl font-medium">{i18n.t(searchState.error.key)}</h2>
            <button type="button" data-sn onclick={handleSearch} class="{bar} px-8">
                {i18n.t('content.retry')}
            </button>
        </div>
    {:else if searchState.hasSearched && searchState.displayResults.length === 0}
        <div class="flex min-h-[40dvh] flex-col items-center justify-center gap-3 text-center">
            <SearchX class="size-14 text-muted-foreground" />
            <h2 class="text-2xl font-bold">{i18n.t('search.empty_title')}</h2>
            <p class="max-w-md text-muted-foreground">{i18n.t('search.empty_desc')}</p>
        </div>
    {:else if searchState.displayResults.length > 0}
        <LazyCardGrid
                items={searchState.displayResults}
                keyFn={(card) => card.cid}
                hasMore={searchState.hasMore}
                isLoading={searchState.isLoading && searchState.page > 1}
                gridClass={TV_GRID}
                onLoadMore={() => {
                if (!searchState.isLoading) searchState.nextPage();
            }}
        >
            {#snippet cardContent(card)}
                <TvCard {card} />
            {/snippet}
        </LazyCardGrid>
    {/if}
</div>

{#if panel === 'source'}
    <TvPanel title={i18n.t('search.source')} onclose={closePanel}>
        <TvSearchSources availableExtensions={searchState.availableExtensions} onSelectSource={selectSource} />
    </TvPanel>
{:else if panel === 'filters'}
    <TvPanel title={i18n.t('search.filters')} onclose={closePanel}>
        <TvSearchFilters
                {extFiltersSchema}
                onChange={() => {
                searchState.page = 1;
                filtersDirty = true;
            }}
        />

        {#snippet footer()}
            <div class="flex gap-3">
                <button
                        type="button"
                        data-sn
                        onclick={closePanel}
                        class="{bar} flex-1 justify-center bg-primary text-primary-foreground"
                >
                    {i18n.t('search.apply_search')}
                </button>
                <button type="button" data-sn onclick={clearFilters} class="{bar} justify-center">
                    {i18n.t('search.clear_filters')}
                </button>
            </div>
        {/snippet}
    </TvPanel>
{/if}