<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { scheduleStore } from "@/app/schedule.svelte.js";
    import { untrack } from "svelte";

    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";
    import CalendarIcon from 'phosphor-svelte/lib/CalendarBlankIcon';
    import AlertCircle from 'phosphor-svelte/lib/WarningCircleIcon';
    import RefreshCw from 'phosphor-svelte/lib/ArrowsClockwiseIcon';
    import ListFilter from 'phosphor-svelte/lib/FunnelIcon';
    import { fade } from "svelte/transition";
    import { layoutState } from "@/stores/layout.svelte.js";
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import {Button} from "$lib/components/ui/button";
    import PageHeader from "@/components/PageHeader.svelte";
    import LazyCardGrid from "@/components/card/LazyCardGrid.svelte";
    import SmartImage from "@/components/SmartImage.svelte";
    import ListIcon from 'phosphor-svelte/lib/ListIcon';
    import { viewPrefs } from "@/stores/viewPrefs.svelte";
    import { getCardTitle, getCardShouldBlur, normalizeFullContent } from "@/utils/normalize";

    $effect(() => {
        layoutState.title    = i18n.t("schedule.title");
        layoutState.showBack = false;
        layoutState.backUrl  = null;
        layoutState.headerAction = headerActions;
        untrack(() => scheduleStore.load());
    });

    // Changing "days ahead" refetches (the store compares the window).
    $effect(() => {
        viewPrefs.data.scheduleDaysAhead;
        untrack(() => scheduleStore.load());
    });

    function getMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }

    const PAGE_SIZE = 30;
    let limit = $state(PAGE_SIZE);

    const total = $derived(scheduleStore.groups.reduce((n, g) => n + g.items.length, 0));
    const hasMore = $derived(limit < total);

    // ---- calendar ----
    function dayKey(d: Date) {
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    }

    const weeks = $derived.by(() => {
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const firstFetched = new Date(today);
        firstFetched.setDate(today.getDate() - 2);
        const last = new Date(today);
        last.setDate(today.getDate() + viewPrefs.data.scheduleDaysAhead);

        const byDay: Record<string, typeof scheduleStore.filteredEntries> = {};
        for (const e of scheduleStore.filteredEntries) {
            (byDay[dayKey(new Date(getMs(e.airingAt)))] ??= []).push(e);
        }

        const cursor = new Date(today);
        cursor.setDate(today.getDate() - ((today.getDay() - viewPrefs.data.weekStart + 7) % 7));

        const rows = [];
        while (cursor <= last) {
            const days = [];
            for (let i = 0; i < 7; i++) {
                const date = new Date(cursor);
                const key = dayKey(date);
                days.push({
                    key,
                    date,
                    isToday: date.getTime() === today.getTime(),
                    inRange: date >= firstFetched && date <= last,
                    items: (byDay[key] ?? []).sort((a, b) => a.airingAt - b.airingAt),
                });
                cursor.setDate(cursor.getDate() + 1);
            }
            rows.push(days);
        }
        return rows;
    });

    const weekdayNames = $derived(
        Array.from({ length: 7 }, (_, i) =>
            // 2024-01-07 was a Sunday.
            new Date(2024, 0, 7 + ((viewPrefs.data.weekStart + i) % 7)).toLocaleDateString(i18n.locale, { weekday: "short" })
        )
    );

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
</script>

<svelte:head>
    <title>{i18n.t("schedule.title")}</title>
</svelte:head>

{#snippet viewToggle()}
    <div class="flex items-center rounded-sm bg-muted/50 p-0.5" role="radiogroup" aria-label={i18n.t('schedule.view')}>
        {#each [['list', ListIcon, 'schedule.view_list'], ['calendar', CalendarIcon, 'schedule.view_calendar']] as const as [view, Icon, key] (view)}
            {@const on = viewPrefs.data.scheduleView === view}
            <button
                    type="button"
                    role="radio"
                    aria-checked={on}
                    title={i18n.t(key)}
                    aria-label={i18n.t(key)}
                    onclick={() => viewPrefs.set({ scheduleView: view })}
                    class="grid h-9 w-9 place-items-center rounded-sm transition-colors {on ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
            >
                <Icon class="size-[18px]" />
            </button>
        {/each}
    </div>
{/snippet}

{#snippet headerActions()}
    <div class="flex items-center gap-0.5">
        {@render viewToggle()}
        <Button
                variant="ghost"
                size="icon"
                class="h-10 w-10 rounded-sm hover:bg-muted/50 {scheduleStore.myListOnly ? 'text-primary bg-primary/10 hover:bg-primary/15' : ''}"
                onclick={() => scheduleStore.toggleMyList()}
                title={scheduleStore.myListOnly ? i18n.t('schedule.showing_my_list') : i18n.t('schedule.show_my_list')}
        >
            <ListFilter class="w-[22px] h-[22px]" />
        </Button>

        <Button
                variant="ghost"
                size="icon"
                class="h-10 w-10 rounded-sm hover:bg-muted/50"
                onclick={() => scheduleStore.load(true)}
                disabled={scheduleStore.isLoading}
        >
            <RefreshCw class="w-[22px] h-[22px] {scheduleStore.isLoading ? 'animate-spin' : ''}" />
        </Button>
    </div>
{/snippet}

<main class="bg-background px-4 md:px-8 lg:pl-32 lg:pr-12 lg:pt-20 pb-16 w-full max-w-[2000px] mx-auto space-y-10 pt-5">
    <PageHeader
            title={i18n.t("schedule.upcoming_episodes")}
            actions={headerActions}
    />

    <section class="relative w-full">
        {#if scheduleStore.isLoading}
            <div class="space-y-12">
                {#each Array(3) as _, i (i)}
                    <div class="space-y-6">
                        <Skeleton class="h-8 w-48 rounded-lg" />
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 2xl:grid-cols-6 gap-4">
                            {#each Array(5) as __, j (j)}
                                <Skeleton class="aspect-[2/3] w-full rounded-sm" />
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>

        {:else if scheduleStore.error}
            <div class="flex flex-col items-center justify-center py-32 text-center space-y-5 border-2 border-dashed rounded-xl border-destructive/20 bg-destructive/5" in:fade>
                <div class="bg-destructive/10 text-destructive p-6 rounded-full shadow-sm border border-destructive/20">
                    <AlertCircle class="h-12 w-12" />
                </div>
                <div class="space-y-2 px-6">
                    <h3 class="text-2xl font-bold text-destructive">{i18n.t(scheduleStore.error.key)}</h3>
                    <button
                            class="text-sm font-medium mt-6 px-4 py-2 border border-destructive/20 text-destructive rounded-md hover:bg-destructive/10 transition-colors"
                            onclick={() => scheduleStore.load()}
                    >
                        {i18n.t("reader.retry")}
                    </button>
                </div>
            </div>

        {:else if scheduleStore.groups.length === 0}
            <div class="flex flex-col items-center justify-center py-32 text-center space-y-5 border-2 border-dashed rounded-xl border-muted/20 bg-muted/5" in:fade>
                <div class="bg-background p-6 rounded-full shadow-sm border border-border/50">
                    <CalendarIcon class="h-12 w-12 text-muted-foreground/30" />
                </div>
                <div class="space-y-2 px-6">
                    <h3 class="text-2xl font-bold">{i18n.t("schedule.empty_title")}</h3>
                    <p class="text-sm text-muted-foreground max-w-[300px] mx-auto">{i18n.t("schedule.empty_desc")}</p>
                </div>
            </div>

        {:else if viewPrefs.data.scheduleView === 'calendar'}
            <div class="overflow-x-auto pb-2 lg:pl-16" in:fade={{ duration: 300 }}>
                <div class="grid min-w-[980px] grid-cols-7 gap-px overflow-hidden rounded-sm border border-border/50 bg-border/50">
                    {#each weekdayNames as name, i (i)}
                        <div class="bg-muted/40 px-3 py-2 text-[11px] font-bold uppercase tracking-widest text-muted-foreground">{name}</div>
                    {/each}
                    {#each weeks as week, wi (wi)}
                        {#each week as day (day.key)}
                            <div class="min-h-32 bg-background p-2 {day.inRange ? '' : 'opacity-40'}">
                                <div class="mb-2 flex items-center gap-2">
                                    <span class="grid size-7 place-items-center rounded-full text-sm font-bold tabular-nums
                                                 {day.isToday ? 'bg-primary text-primary-foreground' : 'text-foreground/80'}">
                                        {day.date.getDate()}
                                    </span>
                                    {#if day.items.length}
                                        <span class="text-[10px] font-semibold text-muted-foreground">{day.items.length}</span>
                                    {/if}
                                </div>
                                <!-- About three entries tall; busy days scroll inside the cell. -->
                                <div class="max-h-[11.5rem] space-y-1.5 overflow-y-auto overscroll-contain pr-0.5 thin-scrollbar">
                                    {#each day.items as entry (`${entry.trackerId}-${entry.episode}`)}
                                        {@const card = normalizeFullContent(entry.fullContent)}
                                        {@const ms = getMs(entry.airingAt)}
                                        <a
                                                href={card.href}
                                                class="flex items-center gap-2 rounded-sm p-1 transition-colors hover:bg-muted/50 {ms <= Date.now() ? 'opacity-60' : ''}"
                                        >
                                            <div class="relative h-12 w-8 shrink-0 overflow-hidden rounded-sm bg-muted">
                                                <SmartImage src={card.cover} alt="" imageHeaders={card.imageHeaders} shouldBlur={getCardShouldBlur(card)} />
                                            </div>
                                            <div class="min-w-0">
                                                <p class="line-clamp-2 text-xs font-semibold leading-tight">{getCardTitle(card)}</p>
                                                <p class="mt-0.5 text-[10px] font-medium text-muted-foreground tabular-nums">
                                                    {i18n.t("schedule.episode_number", { num: entry.episode })} · {viewPrefs.formatTime(ms)}
                                                </p>
                                            </div>
                                        </a>
                                    {/each}
                                </div>
                            </div>
                        {/each}
                    {/each}
                </div>
            </div>
        {:else}
            {#each visibleGroups as group, gi (group.key)}
                <div class="relative z-10" in:fade={{ duration: 400 }}>
                    <div class="flex items-center gap-4 mb-6 top-9 bg-background/95 backdrop-blur-md pt-10 px-14 z-20 lg:-ml-[5px]">
                        <h2 class="text-2xl font-black tracking-tight {group.isToday ? 'text-primary' : 'text-foreground'}">
                            {group.header}
                        </h2>
                        {#if group.isToday}
                            <Badge variant="default" class="uppercase tracking-widest text-[10px] font-black">
                                {i18n.t("schedule.airing_today")}
                            </Badge>
                        {/if}
                        <div class="h-[1px] flex-1 bg-border/40 ml-4 hidden sm:block"></div>
                    </div>

                    <LazyCardGrid
                            items={group.items}
                            keyFn={(i) => `${i.trackerId}-${i.episode}`}
                            hasMore={hasMore && gi === visibleGroups.length - 1}
                            onLoadMore={() => (limit += PAGE_SIZE)}
                            class="lg:pl-16"
                            gridClass="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 2xl:grid-cols-8 gap-4"
                    >
                        {#snippet cardContent(item)}
                            <CardWrapper {...item.card} disablePreview={true}>
                                {#snippet overlay()}
                                    <div class="flex items-end justify-between px-2 pb-2 w-full"
                                         style="padding-top: 2rem;">
                                            <span class="text-[10px] font-black uppercase tracking-wider text-white">
                                                {i18n.t("schedule.episode_number", { num: item.episode })}
                                            </span>
                                        <span class="bg-primary/90 backdrop-blur-sm text-primary-foreground text-[10px] font-bold px-2 py-1 rounded-sm">
                                                {viewPrefs.formatTime(getMs(item.airingAt))}
                                            </span>
                                    </div>
                                {/snippet}
                            </CardWrapper>
                        {/snippet}
                    </LazyCardGrid>
                </div>
            {/each}
        {/if}
    </section>
</main>
<style>
    .thin-scrollbar {
        scrollbar-width: thin;
        scrollbar-color: color-mix(in srgb, var(--foreground) 20%, transparent) transparent;
    }
</style>
