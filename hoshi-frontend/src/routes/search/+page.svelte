<script lang="ts">
    import { untrack } from "svelte";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fade } from "svelte/transition";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { searchState } from '@/app/search.svelte.js';

    import SearchFilters from "$lib/components/search/SearchFilters.svelte";
    import SearchSourceGrid from "$lib/components/search/SearchSourceGrid.svelte";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Search, SearchX, ListFilter, X, AlertCircle, Tv, Book, BookOpen } from "lucide-svelte";
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import LazyCardGrid from "@/components/card/LazyCardGrid.svelte";

    let isDrawerOpen = $state(false);
    let isMobileSearchActive = $state(false);
    let extFiltersSchema = $state<Record<string, any>>({});

    let searchTimeout: ReturnType<typeof setTimeout> | null = null;

    const debouncedSearch = () => {
        if (searchTimeout) clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            handleSearch();
        }, 450);
    };

    $effect(() => {
        layoutState.title = isMobileSearchActive ? "" : i18n.t('search.title');
        layoutState.showBack = false;
        layoutState.headerAction = mobileHeaderAction;
        return () => { layoutState.headerAction = undefined; };
    });

    $effect(() => {
        const currentExts = searchState.availableExtensions;
        untrack(() => {
            if (searchState.searchMode === "extension" && (!searchState.selectedExtension || !currentExts.find(e => e.id === searchState.selectedExtension))) {
                if (currentExts.length > 0) {
                    searchState.selectedExtension = currentExts[0].id;
                } else {
                    searchState.selectedExtension = "";
                    searchState.searchMode = "tracker";
                }
            }

            if (!searchState.hasSearched) {
                handleSearch();
            }
        });
    });

    $effect(() => {
        if (searchState.searchMode === "extension" && searchState.selectedExtension) {
            extensionsApi.getFilters(searchState.selectedExtension)
                .then(res => {
                    extFiltersSchema = res.filters || {};

                    Object.entries(extFiltersSchema).forEach(([key, filterDef]) => {
                        if (filterDef.type === 'boolean' && searchState.extFilterValues[key] === undefined) {
                            searchState.extFilterValues[key] = false;
                        }
                    });
                })
                .catch(() => { extFiltersSchema = {}; });
        } else {
            extFiltersSchema = {};
        }
    });

    const handleSearch = () => {
        searchState.page = 1;
        if (searchTimeout) clearTimeout(searchTimeout);
        if (searchState.searchMode === "tracker") {
            searchState.search();
        } else {
            searchState.extensionSearch();
        }
    };

    const clearFilters = () => {
        searchState.clearFilters();
        handleSearch();
    };

    const selectSource = (mode: "tracker" | "extension", extId: string = "", tracker: "anilist" | "mal" | "kitsu" = "anilist", isMobile = false) => {
        searchState.searchMode = mode;
        if (mode === "extension") searchState.selectedExtension = extId;
        else searchState.tracker = tracker;

        if (!isMobile) {
            handleSearch();
        }
    };

    const clearQuery = () => {
        searchState.query = "";
        handleSearch();
    };
</script>

<svelte:head>
    <title>{i18n.t('search.title')}</title>
</svelte:head>

{#snippet mobileHeaderAction()}
    {#if isMobileSearchActive}
        <div class="flex items-center gap-1 w-full pl-2" in:fade={{duration: 150}}>
            <form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="relative w-full group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground focus-within:text-primary transition-colors" />
                <Input
                        id="mobile-search-input"
                        type="text"
                        placeholder={i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })}
                        class="pl-9 pr-3 h-9 text-sm rounded-sm border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
                        bind:value={searchState.query}
                        oninput={debouncedSearch}
                />
            </form>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full shrink-0" onclick={() => {
                isMobileSearchActive = false;
                if(searchState.query.trim() === '') handleSearch();
            }}>
                <X class="w-[22px] h-[22px] text-foreground" />
            </Button>
        </div>
    {:else}
        <div class="flex items-center text-foreground gap-0.5" in:fade={{duration: 150}}>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50" onclick={() => {
                isMobileSearchActive = true;
                setTimeout(() => document.getElementById('mobile-search-input')?.focus(), 50);
            }}>
                <Search class="w-[22px] h-[22px]" />
            </Button>

            <Drawer.Root bind:open={isDrawerOpen}>
                <Drawer.Trigger>
                    <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50 relative">
                        {#if searchState.searchMode === 'extension' || searchState.status || searchState.genre}
                            <div class="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full"></div>
                        {/if}
                        <ListFilter class="w-[22px] h-[22px]" />
                    </Button>
                </Drawer.Trigger>
                <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                    <div class="w-full h-full flex flex-col overflow-hidden">
                        <div class="flex-1 p-6 overflow-y-auto hide-scrollbar flex flex-col gap-8 pb-6">
                            <h3 class="font-black text-2xl tracking-tight flex items-center gap-2">
                                <ListFilter class="w-5 h-5 text-primary" />
                                {i18n.t('search.search_settings')}
                            </h3>

                            <div class="space-y-3">
                                <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.type')}</h4>
                                <div class="bg-muted/20 p-1.5 rounded-xl grid grid-cols-3 gap-1">
                                    <button onclick={() => { searchState.contentType = 'anime'; handleSearch(); }} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'anime' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.anime')}</button>
                                    <button onclick={() => { searchState.contentType = 'manga'; handleSearch(); }} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'manga' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.manga')}</button>
                                    <button onclick={() => { searchState.contentType = 'novel'; handleSearch(); }} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'novel' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.novel')}</button>
                                </div>
                            </div>

                            <div class="space-y-3">
                                <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.source')}</h4>
                                <SearchSourceGrid isMobile={true} availableExtensions={searchState.availableExtensions} onSelectSource={selectSource} />
                            </div>

                            <div class="space-y-3">
                                <div class="flex items-center justify-between">
                                    <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.filters')}</h4>
                                    <button onclick={clearFilters} class="text-xs font-semibold text-primary/80 hover:text-primary">{i18n.t('search.clear')}</button>
                                </div>
                                <SearchFilters
                                        searchMode={searchState.searchMode}
                                        tracker={searchState.tracker}
                                        bind:status={searchState.status}
                                        bind:genre={searchState.genre}
                                        bind:format={searchState.format}
                                        bind:nsfw={searchState.nsfw}
                                        {extFiltersSchema}
                                        bind:extFilterValues={searchState.extFilterValues}
                                        onChange={handleSearch}
                                        onClear={clearFilters}
                                />
                            </div>
                        </div>

                        <div class="shrink-0 p-4 bg-background border-t border-border/40 pb-safe z-10">
                            <Button class="w-full h-12 rounded-xl font-bold text-base shadow-sm" onclick={() => { handleSearch(); isDrawerOpen = false; }}>
                                {i18n.t('search.apply_search')}
                            </Button>
                        </div>
                    </div>
                </Drawer.Content>
            </Drawer.Root>
        </div>
    {/if}
{/snippet}

<div class="bg-background px-4 md:px-8 lg:pl-32 lg:pr-12 lg:pt-20 w-full max-w-[2000px] mx-auto space-y-10 pt-5">
    <section class="flex flex-col lg:flex-row gap-8 lg:gap-10 w-full items-start">

        <aside class="hidden lg:flex flex-col gap-5 w-[280px] shrink-0 sticky top-18 max-h-[calc(100vh-6rem)] overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden select-none">
            <div class="space-y-1.5 p-0.5">
                <form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="relative w-full group">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input
                            type="text"
                            placeholder={i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })}
                            class="pl-9 pr-9 h-10 text-sm rounded-md border border-border/40 bg-muted/10 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-sm"
                            bind:value={searchState.query}
                            oninput={debouncedSearch}
                    />
                    {#if searchState.query}
                        <button
                                type="button"
                                onclick={clearQuery}
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                        >
                            <X class="w-4 h-4" />
                        </button>
                    {/if}
                </form>
            </div>

            <div class="space-y-1">
                <div class="flex flex-col gap-0.5">
                    <button
                            onclick={() => { searchState.contentType = 'anime'; handleSearch(); }}
                            class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm font-medium transition-all w-full {searchState.contentType === 'anime' ? 'bg-muted/30 text-primary font-semibold shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/20'}"
                    >
                        <Tv class="w-4 h-4" />
                        {i18n.t('search.anime')}
                    </button>
                    <button
                            onclick={() => { searchState.contentType = 'manga'; handleSearch(); }}
                            class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm font-medium transition-all w-full {searchState.contentType === 'manga' ? 'bg-muted/30 text-primary font-semibold shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/20'}"
                    >
                        <Book class="w-4 h-4" />
                        {i18n.t('search.manga')}
                    </button>
                    <button
                            onclick={() => { searchState.contentType = 'novel'; handleSearch(); }}
                            class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm font-medium transition-all w-full {searchState.contentType === 'novel' ? 'bg-muted/30 text-primary font-semibold shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-muted/20'}"
                    >
                        <BookOpen class="w-4 h-4" />
                        {i18n.t('search.novel')}
                    </button>
                </div>
            </div>

            <div class="border-t border-border/40"></div>

            <SearchSourceGrid
                    isMobile={false}
                    availableExtensions={searchState.availableExtensions}
                    onSelectSource={selectSource}
            />

            <div class="border-t border-border/40"></div>

            <div class="space-y-3">
                <SearchFilters
                        searchMode={searchState.searchMode}
                        tracker={searchState.tracker}
                        bind:status={searchState.status}
                        bind:genre={searchState.genre}
                        bind:format={searchState.format}
                        bind:nsfw={searchState.nsfw}
                        {extFiltersSchema}
                        bind:extFilterValues={searchState.extFilterValues}
                        onChange={handleSearch}
                        onClear={clearFilters}
                />
            </div>
        </aside>

        <div class="flex-1 min-w-0 w-full flex flex-col gap-6">
            <div class="w-full">
                {#if searchState.isLoading && searchState.page === 1}
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8">
                        {#each Array.from({ length: 12 }) as _, i}
                            <div class="flex flex-col animate-pulse">
                                <div class="aspect-[2/3] bg-muted rounded-3xl mb-4"></div>
                                <div class="h-5 bg-muted rounded-xl w-4/5 mb-2"></div>
                                <div class="h-3 bg-muted rounded-xl w-2/3"></div>
                            </div>
                        {/each}
                    </div>
                {:else if searchState.error}
                    <div class="flex min-h-[50vh] flex-col items-center justify-center text-center">
                        <h2 class="text-xl font-medium">
                            {i18n.t(searchState.error.key)}
                        </h2>

                        <Button
                                variant="outline"
                                class="mt-6"
                                onclick={handleSearch}
                        >
                            {i18n.t("content.retry")}
                        </Button>
                    </div>
                {:else if searchState.hasSearched && searchState.displayResults.length === 0}
                    <Empty.Root class="border border-dashed py-24 rounded-2xl bg-muted/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon"><SearchX class="w-12 h-12" /></Empty.Media>
                            <Empty.Title class="text-2xl">{i18n.t('search.empty_title')}</Empty.Title>
                            <Empty.Description class="max-w-sm mx-auto text-base">
                                {i18n.t('search.empty_desc')}
                            </Empty.Description>
                        </Empty.Header>
                    </Empty.Root>
                {:else if searchState.displayResults.length > 0}
                    <LazyCardGrid
                            items={searchState.displayResults}
                            keyFn={(card) => card.cid}
                            hasMore={true}
                            isLoading={searchState.isLoading && searchState.page > 1}
                            onLoadMore={() => {
                                if (!searchState.isLoading) {
                                    searchState.nextPage();
                                }
                            }}
                    >
                        {#snippet cardContent(card)}
                            <CardWrapper {...card} disablePreview={true} />
                        {/snippet}
                    </LazyCardGrid>
                {/if}
            </div>
        </div>
    </section>
</div>