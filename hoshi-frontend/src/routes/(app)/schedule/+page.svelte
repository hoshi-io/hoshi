<script lang="ts">
    import { auth } from "@/stores/auth.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { scheduleStore } from "@/app/schedule.svelte.js";
    import { untrack } from "svelte";

    import * as Avatar from "$lib/components/ui/avatar";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";
    import { Calendar as CalendarIcon, AlertCircle, RefreshCw, ListFilter } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { layoutState } from "@/stores/layout.svelte.js";
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import {Button} from "$lib/components/ui/button";

    $effect(() => {
        layoutState.title    = i18n.t("schedule.title");
        layoutState.showBack = false;
        layoutState.backUrl  = null;
        layoutState.headerAction = headerActions;
        untrack(() => scheduleStore.load());
    });

    function getMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }
</script>

<svelte:head>
    <title>{i18n.t("schedule.title")}</title>
</svelte:head>

{#snippet headerActions()}
    <div class="flex items-center gap-0.5">
        <Button
                variant="ghost"
                size="icon"
                class="h-10 w-10 rounded-full hover:bg-muted/50 {scheduleStore.myListOnly ? 'text-primary bg-primary/10 hover:bg-primary/15' : ''}"
                onclick={() => scheduleStore.toggleMyList()}
                title={scheduleStore.myListOnly ? i18n.t('schedule.showing_my_list') : i18n.t('schedule.show_my_list')}
        >
            <ListFilter class="w-[22px] h-[22px]" />
        </Button>

        <Button
                variant="ghost"
                size="icon"
                class="h-10 w-10 rounded-full hover:bg-muted/50"
                onclick={() => scheduleStore.load(true)}
                disabled={scheduleStore.isLoading}
        >
            <RefreshCw class="w-[22px] h-[22px] {scheduleStore.isLoading ? 'animate-spin' : ''}" />
        </Button>
    </div>
{/snippet}

<main class="bg-background px-4 md:px-8 lg:pl-32 lg:pr-12 lg:pt-20 pb-16 w-full max-w-[2000px] mx-auto space-y-10 pt-5">
    <header class="hidden md:flex md:flex-row md:items-center justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-center gap-5">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || "U"}
                </Avatar.Fallback>
            </Avatar.Root>
            <div class="space-y-0.5">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">
                    {i18n.t("schedule.upcoming_episodes")}
                </h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider">
                    {i18n.t("schedule.release_calendar", { name: auth.user?.username || i18n.t("schedule.my") })}
                </p>
            </div>
        </div>

        <div class="flex items-center gap-3">
            <button
                    class="flex items-center gap-2 h-11 px-4 rounded-sm border text-xs font-bold transition-colors {scheduleStore.myListOnly
                    ? 'bg-primary text-primary-foreground border-primary'
                    : 'border-border/40 bg-muted/10 hover:bg-muted/30 text-foreground/70'}"
                    onclick={() => scheduleStore.toggleMyList()}
            >
                <ListFilter class="h-3.5 w-3.5" />
                {i18n.t("schedule.my_list_filter")}
            </button>

            <button
                    class="flex items-center justify-center h-11 w-11 rounded-sm border border-border/40 bg-muted/10 hover:bg-muted/30 transition-colors backdrop-blur-sm shadow-sm"
                    onclick={() => scheduleStore.load(true)}
                    disabled={scheduleStore.isLoading}
            >
                <RefreshCw class="h-4 w-4 {scheduleStore.isLoading ? 'animate-spin opacity-50' : ''}" />
            </button>
        </div>
    </header>

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

        {:else}
            <div class="space-y-12 md:space-y-16 relative">
                <div class="hidden lg:block absolute left-[19px] top-4 bottom-0 w-[2px] bg-border/40 z-0 rounded-full"></div>

                {#each scheduleStore.groups as group (group.key)}
                    <div class="relative z-10" in:fade={{ duration: 400 }}>
                        <div class="flex items-center gap-4 mb-6 top-9 bg-background/95 backdrop-blur-md py-4 z-20 lg:-ml-[5px]">
                            <div class="hidden lg:flex h-12 w-12 rounded-full border-4 border-background items-center justify-center shrink-0 shadow-sm z-10 {group.isToday ? 'bg-primary border-primary/20 text-primary-foreground' : 'bg-muted text-muted-foreground'}">
                                <CalendarIcon class="h-5 w-5" />
                            </div>
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

                        <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 2xl:grid-cols-8 gap-4 pl-0 lg:pl-16">
                            {#each group.items as item (`${item.trackerId}-${item.episode}`)}
                                <CardWrapper {...item.card} disablePreview={true}>
                                    {#snippet overlay()}
                                        <div class="flex items-end justify-between px-2 pb-2 w-full"
                                             style="background: linear-gradient(to top, rgba(0,0,0,0.75) 0%, transparent 100%); padding-top: 2rem;">
                                            <span class="text-[10px] font-black uppercase tracking-wider text-white">
                                                {i18n.t("schedule.episode_number", { num: item.episode })}
                                            </span>
                                            <span class="bg-primary/90 backdrop-blur-sm text-primary-foreground text-[10px] font-bold px-2 py-1 rounded-md">
                                                {new Date(getMs(item.airingAt)).toLocaleTimeString(i18n.locale, { hour: "2-digit", minute: "2-digit", hour12: false })}
                                            </span>
                                        </div>
                                    {/snippet}
                                </CardWrapper>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</main>