<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { Calendar as CalendarIcon, AlertCircle, ListFilter, Check } from 'lucide-svelte';

    import { i18n } from '@/stores/i18n.svelte.js';
    import { scheduleStore } from '@/app/schedule.svelte.js';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import LazyCardGrid from '@/components/card/LazyCardGrid.svelte';
    import TvCard from '@/tv/TvCard.svelte';

    onMount(() => {
        scheduleStore.load();
    });

    function getMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }

    function formatTime(ts: number) {
        return new Date(getMs(ts)).toLocaleTimeString(i18n.locale, {
            hour: '2-digit',
            minute: '2-digit',
            hour12: false
        });
    }

    // 30 = divisible by 5/6 columns, so pages end on full rows.
    const PAGE_SIZE = 30;
    let limit = $state(PAGE_SIZE);

    const total = $derived(scheduleStore.groups.reduce((n, g) => n + g.items.length, 0));
    const hasMore = $derived(limit < total);

    const visibleGroups = $derived.by(() => {
        let left = limit;
        const out: typeof scheduleStore.groups = [];
        for (const g of scheduleStore.groups) {
            if (left <= 0) break;
            out.push(left >= g.items.length ? g : { ...g, items: g.items.slice(0, left) });
            left -= g.items.length;
        }
        return out;
    });

    function toggleMyList() {
        scheduleStore.toggleMyList();
        limit = PAGE_SIZE;
    }
</script>

<svelte:head>
    <title>{i18n.t('schedule.title')}</title>
</svelte:head>

<div class="space-y-8 pb-16">
    <header class="flex items-center justify-between gap-6">
        <h1 class="text-4xl font-black tracking-tight">{i18n.t('schedule.upcoming_episodes')}</h1>

        <button
                type="button"
                data-sn
                onclick={toggleMyList}
                aria-pressed={scheduleStore.myListOnly}
                class="flex h-12 shrink-0 items-center gap-3 rounded-lg border-2 border-transparent px-5 text-base font-bold
                       outline-none transition-colors
                       focus:border-primary focus:ring-4 focus:ring-primary/40
                       {scheduleStore.myListOnly ? 'bg-primary text-primary-foreground' : 'bg-secondary text-foreground'}"
        >
            {#if scheduleStore.myListOnly}
                <Check class="size-5" />
            {:else}
                <ListFilter class="size-5" />
            {/if}
            {scheduleStore.myListOnly
                ? i18n.t('schedule.my_list_filter')
                : i18n.t('schedule.my_list_filter')}
        </button>
    </header>

    {#if scheduleStore.isLoading}
        <div class="space-y-10">
            {#each Array(2) as _, i (i)}
                <div class="space-y-5">
                    <Skeleton class="h-9 w-64 rounded-lg" />
                    <div class="grid grid-cols-5 gap-5">
                        {#each Array(5) as __, j (j)}
                            <Skeleton class="aspect-[2/3] w-full rounded-md" />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>

    {:else if scheduleStore.error}
        <div class="flex flex-col items-center justify-center gap-6 rounded-xl border-2 border-dashed border-destructive/20 bg-destructive/5 py-24 text-center" in:fade>
            <div class="rounded-full border border-destructive/20 bg-destructive/10 p-6 text-destructive">
                <AlertCircle class="size-14" />
            </div>
            <h3 class="text-2xl font-bold text-destructive">{i18n.t(scheduleStore.error.key)}</h3>
            <button
                    type="button"
                    data-sn
                    onclick={() => scheduleStore.load(true)}
                    class="rounded-lg border-2 border-destructive/30 px-6 py-3 text-base font-bold text-destructive outline-none
                           focus:bg-destructive focus:text-destructive-foreground focus:ring-4 focus:ring-destructive/40"
            >
                {i18n.t('reader.retry')}
            </button>
        </div>

    {:else if scheduleStore.groups.length === 0}
        <div class="flex flex-col items-center justify-center gap-5 rounded-xl border-2 border-dashed border-muted/20 bg-muted/5 py-24 text-center" in:fade>
            <div class="rounded-full border border-border/50 bg-background p-6">
                <CalendarIcon class="size-14 text-muted-foreground/40" />
            </div>
            <h3 class="text-2xl font-bold">{i18n.t('schedule.empty_title')}</h3>
            <p class="max-w-md text-base text-muted-foreground">{i18n.t('schedule.empty_desc')}</p>
        </div>

    {:else}
        {#each visibleGroups as group, gi (group.key)}
            <section in:fade={{ duration: 200 }}>
                <!-- No sticky/blur header on TV: backdrop-blur is costly on TV SoCs and
                     the focused row scrolls the page anyway. -->
                <div class="mb-4 flex items-center gap-4">
                    <h2 class="text-3xl font-black tracking-tight {group.isToday ? 'text-primary' : 'text-foreground'}">
                        {group.header}
                    </h2>
                    {#if group.isToday}
                        <span class="rounded-md bg-primary px-3 py-1 text-xs font-black uppercase tracking-widest text-primary-foreground">
                            {i18n.t('schedule.airing_today')}
                        </span>
                    {/if}
                    <div class="h-px flex-1 bg-border/40"></div>
                </div>

                <!-- p-3: room for the focus scale + ring so the edge columns are not clipped. -->
                <LazyCardGrid
                        items={group.items}
                        keyFn={(i) => `${i.trackerId}-${i.episode}`}
                        hasMore={hasMore && gi === visibleGroups.length - 1}
                        onLoadMore={() => (limit += PAGE_SIZE)}
                        gridClass="grid grid-cols-5 gap-x-5 gap-y-6 p-3"
                >
                    {#snippet cardContent(item)}
                        <TvCard card={item.card} widthClass="w-full">
                            {#snippet overlay()}
                                <div class="flex w-full items-end justify-between gap-2">
                                    <span class="text-xs font-black uppercase tracking-wider text-white">
                                        {i18n.t('schedule.episode_number', { num: item.episode })}
                                    </span>
                                    <span class="rounded-sm bg-primary px-2 py-1 text-xs font-bold text-primary-foreground">
                                        {formatTime(item.airingAt)}
                                    </span>
                                </div>
                            {/snippet}
                        </TvCard>
                    {/snippet}
                </LazyCardGrid>
            </section>
        {/each}
    {/if}
</div>