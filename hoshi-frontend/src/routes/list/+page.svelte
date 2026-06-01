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
        Search, List, MoreVertical, CheckCircle2,
        PlayCircle, Clock, PauseCircle, XCircle, Monitor, Library, AlertCircle, SlidersHorizontal, X
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from "@/stores/config.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import CardContainer from "@/components/card/CardContainer.svelte";

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
    let sentinel = $state<HTMLElement | null>(null);

    $effect(() => {
        listStore.sorted;
        visibleCount = PAGE_SIZE;
    });

    $effect(() => {
        if (!sentinel) return;

        const observer = new IntersectionObserver((entries) => {
            if (entries[0].isIntersecting) {
                visibleCount = Math.min(visibleCount + PAGE_SIZE, listStore.sorted.length);
            }
        }, { rootMargin: '200px' });

        observer.observe(sentinel);
        return () => observer.disconnect();
    });

    const visibleItems = $derived(listStore.sorted.slice(0, visibleCount));
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
        <div class="flex items-start gap-5 w-full">
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
        <aside class="hidden lg:flex flex-col gap-8 w-64 shrink-0 sticky top-16 h-fit">
            <div class="space-y-3">{@render searchBar()}</div>
            <div class="space-y-6">
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.sort_by")}</h3>
                    {@render sortSelect()}
                </div>
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.content_type")}</h3>
                    {@render desktopTypeList()}
                </div>
                <div class="space-y-2 border-t border-border/40 pt-6">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.status")}</h3>
                    {@render desktopStatusList()}
                </div>
            </div>
        </aside>

        <section class="flex-1 min-w-0">
            {#if listStore.isLoading}
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-7 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8 mb-10 mb-10">
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
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-7 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8 mb-10 mb-10">
                    {#each visibleItems as item (item.original.cid)}
                        <div in:fade={{ duration: 200 }} class="group">
                            <CardWrapper {...item.card} disablePreview={true}>
                                {#snippet overlay()}
                                    <div class="flex items-start justify-between p-2 w-full mb-auto">
                                        <div class="bg-black/80 backdrop-blur-md px-2.5 py-1 rounded-md shadow-sm border border-white/10 flex items-center gap-2">
                                            <span class="text-xs font-black text-primary">{item.original.progress}</span>
                                            <span class="text-[10px] font-bold text-white/60">/ {item.original.totalUnits || '?'}</span>
                                            {#if item.original.score}
                                                <div class="w-px h-3 bg-white/20"></div>
                                                <span class="text-[11px] font-black text-yellow-500">★ {item.original.score}</span>
                                            {/if}
                                        </div>

                                        <button
                                                class="pointer-events-auto opacity-100 lg:opacity-0 group-hover:opacity-100 transition-opacity duration-200 h-7 w-7 rounded-md bg-black/80 backdrop-blur-md text-white/90 border border-white/10 hover:bg-primary hover:text-primary-foreground shadow-sm flex items-center justify-center"
                                                onclick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    listStore.openEdit(item.original);
                }}
                                        >
                                            <MoreVertical class="h-4 w-4" />
                                        </button>
                                    </div>
                                {/snippet}
                            </CardWrapper>
                        </div>
                    {/each}
                </div>

                {#if visibleCount < listStore.sorted.length}
                    <div bind:this={sentinel} class="h-10 flex items-center justify-center mt-4">
                        <div class="flex gap-1">
                            <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:0ms]"></span>
                            <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:150ms]"></span>
                            <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:300ms]"></span>
                        </div>
                    </div>
                {/if}
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