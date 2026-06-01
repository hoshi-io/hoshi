<script lang="ts">
    import { untrack } from "svelte";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fade, fly } from "svelte/transition";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { searchState } from '@/app/search.svelte.js';

    import SearchFilters from "$lib/components/search/SearchFilters.svelte";
    import SearchSourceGrid from "$lib/components/search/SearchSourceGrid.svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import * as Popover from "$lib/components/ui/popover";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Spinner } from "$lib/components/ui/spinner";
    import { Search, SearchX, Plug, SlidersHorizontal, Tv, Book, BookOpen, LayoutGrid, ListFilter, X, AlertCircle } from "lucide-svelte";
    import CardWrapper from "@/components/card/CardWrapper.svelte";

    let isSourcePopoverOpen = $state(false);
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

    function infiniteScroll(node: HTMLElement) {
        const observer = new IntersectionObserver((entries) => {
            if (entries[0].isIntersecting && !searchState.isLoading) {
                searchState.nextPage();
            }
        }, { rootMargin: '400px' });
        observer.observe(node);
        return { destroy() { observer.disconnect(); } };
    }

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
            handleSearch();
        });
    });

    $effect(() => {
        if (searchState.searchMode === "extension" && searchState.selectedExtension) {
            extensionsApi.getFilters(searchState.selectedExtension)
                .then(res => { extFiltersSchema = res.filters || {}; })
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
            isSourcePopoverOpen = false;
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
                        class="pl-9 pr-3 h-9 text-sm rounded-full border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
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
        <!-- ... (drawer se mantiene igual) ... -->
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
                                    <button onclick={() => searchState.contentType = 'anime'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'anime' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.anime')}</button>
                                    <button onclick={() => searchState.contentType = 'manga'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'manga' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.manga')}</button>
                                    <button onclick={() => searchState.contentType = 'novel'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'novel' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.novel')}</button>
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
        <aside class="hidden lg:block w-[280px] sticky top-18">
            <div class="pb-6">
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
            <div class="hidden md:flex flex-col xl:flex-row gap-4 items-start xl:items-center justify-between w-full">
                <form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="relative w-full xl:flex-1 group">
                    <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input
                            type="text"
                            placeholder={i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })}
                            class="pl-12 pr-12 h-12 text-base rounded-xl border border-border/40 bg-muted/10 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-sm"
                            bind:value={searchState.query}
                            oninput={debouncedSearch}
                    />
                    {#if searchState.query}
                        <button
                                type="button"
                                onclick={clearQuery}
                                class="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                        >
                            <X class="w-5 h-5" />
                        </button>
                    {/if}
                </form>

                <div class="flex flex-wrap items-center gap-2 sm:gap-3 shrink-0">
                    <Select.Root type="single" bind:value={searchState.contentType}>
                        <Select.Trigger class="w-[140px] h-12 min-h-[48px] px-4 flex items-center justify-center bg-muted/20 border-none rounded-xl text-base font-semibold">                            {#if searchState.contentType === "anime"}
                                <Tv class="w-4 h-4 mr-2 text-primary" />
                            {:else if searchState.contentType === "manga"}
                                <Book class="w-4 h-4 mr-2 text-primary" />
                            {:else}
                                <BookOpen class="w-4 h-4 mr-2 text-primary" />
                            {/if}
                            {i18n.t(searchState.contentType)}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="anime">{i18n.t('search.anime')}</Select.Item>
                            <Select.Item value="manga">{i18n.t('search.manga')}</Select.Item>
                            <Select.Item value="novel">{i18n.t('search.novel')}</Select.Item>
                        </Select.Content>
                    </Select.Root>

                    <Popover.Root bind:open={isSourcePopoverOpen}>
                        <Popover.Trigger>
                            {#snippet child({ props })}
                                <Button {...props} variant="secondary" class="h-12 rounded-xl text-base font-semibold gap-2 border-none bg-muted/20 hover:bg-muted/30 px-4">
                                    {#if searchState.searchMode === "tracker"}
                                        {searchState.tracker === 'mal' ? 'MyAnimeList' : searchState.tracker === 'kitsu' ? 'Kitsu' : 'AniList'}
                                    {:else}
                                        {@const ext = searchState.availableExtensions.find(e => e.id === searchState.selectedExtension)}
                                        {#if ext?.icon}
                                            <img src={ext.icon} class="w-5 h-5 rounded-md object-cover" alt={ext?.name} />
                                        {:else}
                                            <Plug class="w-4 h-4 text-primary" />
                                        {/if}
                                        {ext?.name}
                                    {/if}
                                </Button>
                            {/snippet}
                        </Popover.Trigger>
                        <Popover.Content align="start" class="w-[360px] p-5 rounded-2xl border-border/50 shadow-2xl bg-card">
                            <h3 class="font-black text-xs text-muted-foreground uppercase tracking-widest mb-4 flex items-center gap-2">
                                <LayoutGrid class="w-4 h-4" /> {i18n.t('search.select_source')}
                            </h3>
                            <SearchSourceGrid isMobile={false} availableExtensions={searchState.availableExtensions} onSelectSource={selectSource} />
                        </Popover.Content>
                    </Popover.Root>
                </div>
            </div>

            <div class="hidden md:block w-full border-t border-border/40 mt-2 mb-2"></div>

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
                    <Empty.Root class="border border-dashed border-destructive/40 py-24 rounded-2xl bg-destructive/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon" class="bg-destructive/10 text-destructive mb-4 p-4 rounded-full">
                                <AlertCircle class="w-10 h-10" />
                            </Empty.Media>
                            <Empty.Title class="text-xl font-bold text-destructive">
                                {i18n.t(searchState.error.key)}
                            </Empty.Title>
                            <Button variant="outline" class="mt-6 border-destructive/20 hover:bg-destructive/10 text-destructive" onclick={handleSearch}>
                                {i18n.t("content.retry")}
                            </Button>
                        </Empty.Header>
                    </Empty.Root>
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
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8">
                        {#each searchState.displayResults as card, index (card.cid)}
                            <div in:fly={{ y: 30, duration: 350, delay: Math.min(index * 35, 420) }}>
                                <CardWrapper {...card} disablePreview={true} />
                            </div>
                        {/each}

                    </div>

                    {#if searchState.isLoading && searchState.page > 1}
                        <div class="flex justify-center w-full py-8">
                            <Spinner class="w-8 h-8 text-primary animate-spin" />
                        </div>
                    {/if}

                    <div use:infiniteScroll class="h-10 w-full"></div>
                {/if}
            </div>
        </div>
    </section>
</div>