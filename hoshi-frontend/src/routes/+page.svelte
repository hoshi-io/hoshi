<script lang="ts">
    import { Skeleton } from '@/components/ui/skeleton';
    import { Tv, Book, BookText } from "lucide-svelte";
    import HomeHero from '@/components/hero/HomeHero.svelte';
    import ContentCardCarousel from '@/components/carousel/CardCarousel.svelte';
    import ContinueCarouselCarousel from '@/components/carousel/ContinueCarousel.svelte';
    import { fade, fly } from 'svelte/transition';
    import type { ContentType } from '@/api/content/types';
    import { appConfig } from "@/stores/config.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { auth } from '@/stores/auth.svelte.js';
    import { homeState } from '@/app/home.svelte.js';

    let currentMode = $state<ContentType>('anime');
    let initializedMode = $state(false);

    const modes = [
        { id: 'anime', label: 'Anime', icon: Tv },
        { id: 'manga', label: 'Manga', icon: Book },
        { id: 'novel', label: 'Novel', icon: BookText }
    ] as const;

    const isSkeletonVisible = $derived(
        (auth.loading || !auth.initialized || homeState.loading) && !homeState.hasData
    );

    $effect(() => {
        if (appConfig.data && !initializedMode) {
            currentMode = appConfig.data.ui.defaultHomeSection as ContentType;
            initializedMode = true;
        }
        layoutState.title = i18n.t("home.title");
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => { layoutState.headerAction = undefined; };
    });

    const currentSection = $derived(homeState.getSection(currentMode));
    const currentTrending = $derived(currentSection?.trending ?? []);

    $effect(() => {
        if (auth.loading || !auth.initialized || !auth.user) return;

        if (homeState.hasData) {
            homeState.refreshContinueWatching();
        } else {
            homeState.load();
        }
    });
</script>

<svelte:head>
    <title>{i18n.t("home.title")}</title>
</svelte:head>

{#snippet mobileHeaderAction()}
    <div class="flex items-center bg-muted/40 p-1 rounded-xl border border-border/40 shadow-inner h-10 mr-1" in:fade={{duration: 150}}>
        {#each modes as { id, label, icon: Icon }}
            <button
                    class="relative flex items-center justify-center px-3 sm:px-4 h-full rounded-lg text-xs font-bold transition-all duration-300 {currentMode === id ? 'bg-background text-primary shadow-sm ring-1 ring-border/30' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => currentMode = id}
                    aria-label={label}
            >
                <Icon class="w-4.5 h-4.5 transition-transform duration-300 {currentMode === id ? 'scale-110' : ''} sm:mr-1.5" />
                <span class="hidden sm:inline-block tracking-wide">{label}</span>
            </button>
        {/each}
    </div>
{/snippet}

<div class="bg-background overflow-x-hidden relative -mt-24 lg:mt-0">

    {#if isSkeletonVisible}
        <div class="w-full space-y-12" in:fade={{ duration: 300 }}>
            <div class="w-full h-[85vh] bg-card/30 flex items-end p-4 md:p-12">
                <div class="space-y-6 max-w-3xl w-full mb-10">
                    <Skeleton class="h-16 md:h-24 w-3/4 rounded-2xl" />
                    <Skeleton class="h-6 md:h-8 w-1/2 rounded-lg" />
                </div>
            </div>
        </div>
    {:else}
        <div
                in:fly={{ y: 20, duration:400, delay: 150}}
                out:fade={{ duration: 150 }}
        >
            {#if currentTrending.length > 0}
                <div class="w-full relative">
                    <HomeHero items={currentTrending.slice(0, 5)}/>
                </div>
            {/if}

            <div class="w-full px-4 md:px-12 lg:pl-32 py-8 relative z-20 space-y-6 md:space-y-10 -mt-20 md:-mt-28 pb-safe">
            <div class="hidden md:flex items-center gap-8 border-b border-border/10 pb-4">
                    {#each modes as { id, label, icon: Icon }}
                        <button
                                class="group relative flex items-center gap-2.5 py-2 transition-all duration-300"
                                onclick={() => currentMode = id}
                        >
                            <Icon class="size-5 transition-colors {currentMode === id ? 'text-primary' : 'text-muted-foreground group-hover:text-foreground'}" />
                            <span class="text-sm font-black uppercase tracking-widest transition-colors {currentMode === id ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'}">
                                {label}
                            </span>
                            {#if currentMode === id}
                                <div class="absolute -bottom-4 left-0 right-0 h-1 bg-primary rounded-t-full" in:fade={{ duration: 200 }}></div>
                            {/if}
                        </button>
                    {/each}
                </div>

                {#each modes as { id }}
                    {#if currentMode === id}
                        {@const section = homeState.getSection(id)}
                        {@const continueItems = homeState.getContinueItems(id)}

                        <div class="contents">
                            {#if section}
                                {#if continueItems.length > 0}
                                    <ContinueCarouselCarousel items={continueItems} mode={id} />
                                {/if}

                                {#if section.trending.length > 0}
                                    <ContentCardCarousel title={i18n.t("home.trending")} items={section.trending} />
                                {/if}

                                {#if section.popular.length > 0}
                                    <ContentCardCarousel title={i18n.t("home.popular")} items={section.popular} />
                                {/if}

                                {#if section.seasonal.length > 0}
                                    <ContentCardCarousel title={id === 'anime' ? i18n.t("home.simulcast") : i18n.t("home.latest")} items={section.seasonal} />
                                {/if}

                                {#if section.upcoming.length > 0}
                                    <ContentCardCarousel title={i18n.t("home.upcoming")} items={section.upcoming} />
                                {/if}

                                {#if section.recentlyFinished.length > 0}
                                    <ContentCardCarousel title={i18n.t("home.recently_finished")} items={section.recentlyFinished} />
                                {/if}

                                {#if section.topRated.length > 0}
                                    <ContentCardCarousel title={i18n.t("home.critically_acclaimed")} items={section.topRated} />
                                {/if}

                                {#if id === 'anime'}
                                    {#if section.topAction.length > 0}
                                        <ContentCardCarousel title={i18n.t("home.action")} items={section.topAction} />
                                    {/if}
                                {/if}
                            {/if}
                        </div>
                    {/if}
                {/each}
            </div>
        </div>
    {/if}
</div>