<script lang="ts">
    import { auth } from "@/stores/auth.svelte.js";
    import { listStore } from "@/app/list.svelte.js";
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import ListEditor from "@/components/modals/ListEditor.svelte";
    import * as Empty from "$lib/components/ui/empty";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import {
        Search, List, MoreVertical, AlertCircle, SlidersHorizontal, X
    } from "lucide-svelte";

    import { fade } from "svelte/transition";

    import { i18n } from "@/stores/i18n.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from "@/stores/config.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import LazyCardGrid from "@/components/card/LazyCardGrid.svelte";

    $effect(() => {
        layoutState.title = listStore.isMobileSearchActive ? "" : i18n.t('list.title');
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileTopbar;
    });

    const statusOptions = $derived([
        { value: "ALL",       label: i18n.t('list.all') },
        { value: "CURRENT",   label: i18n.t('list.current') },
        { value: "COMPLETED", label: i18n.t('list.completed') },
        { value: "PLANNING",  label: i18n.t('list.planning') },
        { value: "PAUSED",    label: i18n.t('list.paused') },
        { value: "DROPPED",   label: i18n.t('list.dropped') },
    ]);

    const statusCounts = $derived({
        ALL:       listStore.entries.length,
        CURRENT:   listStore.stats?.watching   ?? 0,
        COMPLETED: listStore.stats?.completed  ?? 0,
        PLANNING:  listStore.stats?.planning   ?? 0,
        PAUSED:    listStore.stats?.paused     ?? 0,
        DROPPED:   listStore.stats?.dropped    ?? 0,
    });

    const PAGE_SIZE = 30;
    let visibleCount = $state(PAGE_SIZE);

    $effect(() => {
        listStore.sorted;
        visibleCount = PAGE_SIZE;
    });

    const visibleItems = $derived(listStore.sorted.slice(0, visibleCount));

    $effect(() => {
        console.log("Visible count is now:", visibleCount, "out of:", listStore.sorted.length);
    });
</script>

{#snippet statusSelect()}
    <ResponsiveSelect
            bind:value={listStore.activeStatus}
            items={statusOptions}
            class="h-11 rounded-sm font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet sortSelect()}
    <ResponsiveSelect
            bind:value={listStore.activeSort}
            items={[
            { value: "SCORE_DESC", label: i18n.t('list.modal.score')},
            { value: "TITLE_ASC", label: "A-Z" },
            { value: "TITLE_DESC", label: "Z-A" },
        ]}
            class="h-11 rounded-sm font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet searchBar()}
    <div class="relative w-full group">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
        <Input
                placeholder={i18n.t('list.search_placeholder')}
                class="pl-9 pr-3 h-9 text-sm rounded-sm border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
                bind:value={listStore.searchQuery}
        />
    </div>
{/snippet}

{#snippet desktopStatusList()}
    <div class="flex flex-col gap-1 w-full">
        {#each statusOptions as option}
            <button
                    class="flex w-full items-center justify-start rounded-md px-3 py-2 text-sm font-medium transition-colors {listStore.activeStatus === option.value ? 'bg-muted/80 text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
                    onclick={() => listStore.activeStatus = option.value}
            >
                {option.label}
                <span class="ml-auto text-xs text-muted-foreground/60 font-normal tabular-nums">
                    {statusCounts[option.value] ?? 0}
                </span>
            </button>
        {/each}
    </div>
{/snippet}

{#snippet typeSelect()}
    <ResponsiveSelect
            bind:value={listStore.activeType}
            items={[
                { value: "ALL", label: i18n.t('list.all') },
                { value: "anime", label: i18n.t('list.anime') },
                { value: "manga", label: i18n.t('list.manga') },
                { value: "novel", label: i18n.t('list.novel') }
            ]}
            class="h-11 rounded-xl font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet desktopTypeList()}
    <div class="flex flex-col gap-1 w-full">
        {#each [
            { value: "ALL", label: i18n.t('list.all') },
            { value: "anime", label: i18n.t('list.anime') },
            { value: "manga", label: i18n.t('list.manga') },
            { value: "novel", label: i18n.t('list.novel') }
        ] as option}
            <button
                    class="flex w-full items-center justify-start rounded-md px-3 py-2 text-sm font-medium transition-colors {listStore.activeType === option.value ? 'bg-muted/80 text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
                    onclick={() => listStore.activeType = option.value}
            >
                {option.label}
            </button>
        {/each}
    </div>
{/snippet}

{#snippet mobileTopbar()}
    {#if listStore.isMobileSearchActive}
        <div class="flex items-center gap-1 w-full pl-2" in:fade={{ duration: 150 }}>
            <div class="flex-1 min-w-0">{@render searchBar()}</div>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full shrink-0" onclick={() => listStore.isMobileSearchActive = false}>
                <X class="w-[22px] h-[22px]" />
            </Button>
        </div>
    {:else}
        <div class="flex items-center gap-0.5 w-full justify-end" in:fade={{ duration: 150 }}>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50" onclick={() => listStore.isMobileSearchActive = true}>
                <Search class="w-[22px] h-[22px]" />
            </Button>

            <Drawer.Root bind:open={listStore.isDrawerOpen}>
                <Drawer.Trigger>
                    <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50 relative">
                        <SlidersHorizontal class="w-[22px] h-[22px]" />
                        {#if listStore.activeStatus !== 'ALL' || listStore.activeType !== 'ALL' || listStore.activeSort !== 'TITLE_ASC'}
                            <span class="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full border border-background"></span>
                        {/if}
                    </Button>
                </Drawer.Trigger>

                <Drawer.Content class="max-h-[85vh] rounded-t-3xl border-border/50">
                    <div class="w-full h-full flex flex-col overflow-hidden">
                        <div class="flex items-center justify-between p-6 pb-2">
                            <h3 class="font-black text-2xl tracking-tight">{i18n.t("search.filters")}</h3>
                            <Button variant="ghost" size="sm" class="text-xs font-bold text-muted-foreground hover:text-primary" onclick={() => listStore.resetFilters()}>
                                {i18n.t("list.clear_all")}
                            </Button>
                        </div>

                        <div class="flex-1 p-6 pt-2 overflow-y-auto hide-scrollbar space-y-6">
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.sort_by")}</Label>
                                {@render sortSelect()}
                            </div>
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.content_type")}</Label>
                                {@render typeSelect()}
                            </div>
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.status")}</Label>
                                {@render statusSelect()}
                            </div>
                        </div>

                        <div class="shrink-0 p-4 bg-background border-t border-border/40 pb-8">
                            <Button class="w-full h-12 rounded-xl font-bold text-base shadow-sm" onclick={() => listStore.isDrawerOpen = false}>
                                {i18n.t("search.apply_search")}
                            </Button>
                        </div>
                    </div>
                </Drawer.Content>
            </Drawer.Root>
        </div>
    {/if}
{/snippet}

<svelte:head>
    <title>{i18n.t('list.title')}</title>
</svelte:head>

<main class="bg-background px-4 md:px-8 lg:pl-32 lg:pr-12 lg:pt-20 w-full max-w-[2000px] mx-auto space-y-10 pt-5">
    <header class="hidden lg:flex lg:flex-row lg:items-start justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-center gap-5 w-full">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm shrink-0">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>
            <div class="flex flex-col w-full">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight leading-none">
                    {i18n.t('list.header_title', { name: auth.user?.username || i18n.t('list.default_user')})}
                </h1>
            </div>
        </div>
    </header>

    <div class="flex items-start gap-8 w-full pt-4">
        <aside class="hidden lg:flex flex-col gap-5 w-68 shrink-0 sticky top-24 h-[calc(100vh-8rem)] pb-4 overflow-y-auto hide-scrollbar">
            <div class="space-y-2.5 p-0.5">{@render searchBar()}</div>
            <div class="space-y-8 flex-1 flex flex-col justify-start">
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.sort_by")}</h3>
                    {@render sortSelect()}
                </div>
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.content_type")}</h3>
                    {@render desktopTypeList()}
                </div>
                <div class="space-y-2 border-t border-border/40 pt-5 flex-1 flex flex-col">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1 mb-1">{i18n.t("list.status")}</h3>
                    <div class="flex-1 overflow-y-auto hide-scrollbar pr-0.5">
                        {@render desktopStatusList()}
                    </div>
                </div>
            </div>
        </aside>

        <section class="flex-1 min-w-0">
            {#if listStore.isLoading}
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-7 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8 mb-10">
                    {#each Array(14) as _}
                        <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                    {/each}
                </div>
            {:else if listStore.error}
                <Empty.Root class="border border-dashed border-destructive/40 bg-destructive/5 rounded-2xl py-24 min-h-[40vh] flex flex-col items-center justify-center text-center px-4">
                    <Empty.Header>
                        <Empty.Media variant="icon" class="bg-destructive/10 text-destructive mb-4 p-4 rounded-full">
                            <AlertCircle class="size-8" />
                        </Empty.Media>
                        <Empty.Title class="text-xl font-bold text-destructive">{i18n.t(listStore.error.key)}</Empty.Title>
                        <Button variant="outline" class="mt-6 border-destructive/20 hover:bg-destructive/10 text-destructive" onclick={() => listStore.refresh()}>{i18n.t("content.retry")}</Button>
                    </Empty.Header>
                </Empty.Root>
            {:else if listStore.filtered.length === 0}
                <Empty.Root class="border border-dashed border-border/40 bg-muted/5 rounded-2xl py-24 min-h-[40vh] flex items-center justify-center">
                    <Empty.Header>
                        <Empty.Media variant="icon" class="bg-primary/10 text-primary mb-4 p-4 rounded-full"><List class="size-8" /></Empty.Media>
                        <Empty.Title class="text-xl font-bold">{i18n.t('list.empty_title')}</Empty.Title>
                        <Empty.Description class="text-muted-foreground font-medium">{i18n.t('list.empty_desc')}</Empty.Description>
                    </Empty.Header>
                </Empty.Root>
            {:else}
                <LazyCardGrid
                        items={visibleItems}
                        keyFn={(item) => item.original.cid}
                        hasMore={visibleCount < listStore.sorted.length}
                        isLoading={listStore.isLoading} onLoadMore={() => {
            visibleCount = Math.min(visibleCount + PAGE_SIZE, listStore.sorted.length);
        }}
                >
                    {#snippet cardContent(item)}
                        <CardWrapper {...item.card} disablePreview={true}>
                            {#snippet overlay()}
                                <div class="absolute inset-0 p-2 flex flex-col justify-between pointer-events-none select-none">
                                    <!-- Top Row: Status, Privacy, and Actions -->
                                    <div class="flex items-start justify-between w-full gap-2">
                                        <!-- Status & Privacy Badges -->
                                        <div class="flex flex-wrap gap-1.5 items-center max-w-[75%]">
                                            <!-- Status Badge -->
                                            <span class="inline-flex items-center gap-1.5 bg-zinc-950/70 backdrop-blur-md px-2 py-0.5 rounded-md text-[10px] font-bold tracking-wide text-zinc-200 border border-white/5 shadow-sm">
                        <span class="h-1.5 w-1.5 rounded-full" class:bg-emerald-500={item.original.status === 'COMPLETED'} class:bg-sky-500={item.original.status === 'CURRENT'} class:bg-amber-500={item.original.status === 'PAUSED'} class:bg-rose-500={item.original.status === 'DROPPED'} class:bg-purple-500={item.original.status === 'REPEATING'} class:bg-zinc-400={item.original.status === 'PLANNING'}></span>
                                                {item.original.status}
                    </span>

                                            <!-- Private Indicator -->
                                            {#if item.original.isPrivate}
                                                <div class="bg-zinc-950/70 backdrop-blur-md p-1 rounded-md text-zinc-400 border border-white/5 shadow-sm" title="Private Entry">
                                                    <!-- Simple SVG Lock (Replace with your icon library lock if preferred) -->
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                                                </div>
                                            {/if}

                                            <!-- Repeat Tracker -->
                                            {#if item.original.repeatCount > 0}
                        <span class="inline-flex items-center gap-1 bg-purple-950/50 backdrop-blur-md px-1.5 py-0.5 rounded-md text-[10px] font-bold text-purple-300 border border-purple-500/20 shadow-sm">
                            <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/></svg>
                            {item.original.repeatCount}
                        </span>
                                            {/if}
                                        </div>

                                        <!-- Action Button -->
                                        <button
                                                type="button"
                                                class="pointer-events-auto opacity-100 lg:opacity-0 group-hover:opacity-100 transition-all duration-200 h-7 w-7 rounded-md bg-zinc-950/70 backdrop-blur-md text-zinc-300 border border-white/5 hover:bg-primary hover:text-primary-foreground hover:scale-105 shadow-sm flex items-center justify-center"
                                                onclick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        listStore.openEdit(item.original);
                    }}
                                        >
                                            <MoreVertical class="h-4 w-4" />
                                        </button>
                                    </div>

                                    <!-- Bottom Row: Progress and Score Counters -->
                                    <div class="flex items-center justify-between w-full mt-auto">
                                        <!-- Progress Container -->
                                        <div class="bg-zinc-950/75 backdrop-blur-md px-2 py-1 rounded-md shadow-md border border-white/5 flex items-center gap-1.5 font-mono">
                                            <span class="text-xs font-black text-primary">{item.original.progress}</span>
                                            <span class="text-[10px] font-medium text-zinc-500">/</span>
                                            <span class="text-[10px] font-bold text-zinc-400">{item.original.totalUnits || '—'}</span>
                                        </div>

                                        <!-- Score Container -->
                                        {#if item.original.score}
                                            <div class="bg-zinc-950/75 backdrop-blur-md px-2 py-1 rounded-md shadow-md border border-white/5 flex items-center gap-1 font-mono text-xs font-black text-amber-400">
                                                <span class="text-[10px] text-amber-500">★</span>
                                                <span>{item.original.score}</span>
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            {/snippet}
                        </CardWrapper>
                    {/snippet}
                </LazyCardGrid>
            {/if}
        </section>
    </div>
</main>

{#if listStore.selectedEntry}
    <ListEditor
            bind:open={listStore.isModalOpen}
            cid={listStore.selectedEntry.cid}
            title={listStore.selectedEntry.titleI18n?.[appConfig.data?.ui?.titleLanguage || 'romaji'] || listStore.selectedEntry.title}
            contentType={listStore.selectedEntry.contentType}
            coverImage={listStore.selectedEntry.coverImage ?? undefined}
    />
{/if}