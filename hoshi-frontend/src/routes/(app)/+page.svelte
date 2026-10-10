<script lang="ts">
    import { Skeleton } from '@/components/ui/skeleton';
    import { Tv, Book, BookText } from "lucide-svelte";
    import HomeHero from '@/components/hero/HomeHero.svelte';
    import ContentCardCarousel from '@/components/carousel/CardCarousel.svelte';
    import ContinueCarouselCarousel from '@/components/carousel/ContinueCarousel.svelte';
    import { crossfade, fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import type { Action } from 'svelte/action';
    import type { ContentType } from '@/api/content/types';
    import { appConfig } from "@/stores/config.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { auth } from '@/stores/auth.svelte.js';
    import { homeState } from '@/app/home.svelte.js';
    import RecentlyAiredCarousel from "@/components/carousel/RecentlyAiredCarousel.svelte";

    let currentMode = $state<ContentType>('anime');
    let initializedMode = $state(false);

    const modes = [
        { id: 'anime', label: 'Anime', icon: Tv },
        { id: 'manga', label: 'Manga', icon: Book },
        { id: 'novel', label: 'Novel', icon: BookText }
    ] as const;

    const prefersReducedMotion =
        typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

    const [send, receive] = crossfade({
        duration: prefersReducedMotion ? 0 : 300,
        easing: cubicOut,
        fallback: fade
    });

    const reveal: Action<HTMLElement, { delay?: number } | undefined> = (node, params) => {
        if (prefersReducedMotion) return;

        const delay = params?.delay ?? 0;
        node.style.opacity = '0';
        node.style.transform = 'translateY(16px)';
        node.style.transition = `opacity 600ms cubic-bezier(.22,1,.36,1) ${delay}ms, transform 600ms cubic-bezier(.22,1,.36,1) ${delay}ms`;
        void node.offsetHeight; // commit the start state so the transition actually runs

        const io = new IntersectionObserver(([entry]) => {
            if (!entry.isIntersecting) return;
            node.style.opacity = '1';
            node.style.transform = 'none';
            io.disconnect();
        }, { rootMargin: '0px 0px -8% 0px' });
        io.observe(node);

        return { destroy: () => io.disconnect() };
    };

    const isSkeletonVisible = $derived(
        (auth.loading || !auth.initialized || homeState.loading) && !homeState.hasData
    );

    $effect(() => {
        layoutState.title = i18n.t("home.title");
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => { layoutState.headerAction = undefined; };
    });

    $effect(() => {
        if (appConfig.data && !initializedMode) {
            currentMode = appConfig.data.ui.defaultHomeSection as ContentType;
            initializedMode = true;
        }
    });

    const currentSection = $derived(homeState.getSection(currentMode));
    const currentTrending = $derived(currentSection?.trending ?? []);
    const continueItems = $derived(homeState.getContinueItems(currentMode));

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
                    class="relative flex items-center justify-center px-3 sm:px-4 h-full rounded-lg text-xs font-bold transition-colors duration-300 {currentMode === id ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => currentMode = id}
                    aria-label={label}
                    aria-pressed={currentMode === id}
            >
                {#if currentMode === id}
                    <span
                            class="absolute inset-0 rounded-lg bg-background shadow-sm ring-1 ring-border/30"
                            in:receive={{ key: 'mode-pill' }}
                            out:send={{ key: 'mode-pill' }}
                    ></span>
                {/if}
                <Icon class="relative w-4.5 h-4.5 transition-transform duration-300 {currentMode === id ? 'scale-110' : ''} sm:mr-1.5" />
                <span class="relative hidden sm:inline-block tracking-wide">{label}</span>
            </button>
        {/each}
    </div>
{/snippet}

<div class="bg-background overflow-x-hidden relative -mt-24 lg:mt-0 pb-11">

    {#if isSkeletonVisible}
        <div class="w-full" in:fade={{ duration: 300 }}>
            <div class="w-full h-[75vh] md:h-[90vh] min-h-[550px] bg-card/30 flex items-end px-6 md:px-16 lg:pl-32 pb-16 md:pb-24">
                <div class="space-y-6 max-w-3xl w-full">
                    <Skeleton class="h-16 md:h-24 w-3/4 rounded-2xl" />
                    <Skeleton class="h-6 md:h-8 w-1/2 rounded-lg" />
                </div>
            </div>

            <div class="w-full px-4 md:px-12 lg:pl-32 pt-4 md:pt-6 -mt-8 md:-mt-16 relative z-20 space-y-8 md:space-y-10">
                {#each Array(2) as _}
                    <div class="space-y-3 md:space-y-4">
                        <Skeleton class="h-6 md:h-7 w-40 rounded-md" />
                        <div class="flex gap-3 md:gap-5 overflow-hidden">
                            {#each Array(7) as __}
                                <Skeleton class="shrink-0 aspect-[2/3] basis-[42%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 rounded-sm" />
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {:else}
        <div in:fade={{ duration: 500 }}>
            {#if currentTrending.length > 0}
                <div class="w-full relative">
                    <HomeHero items={currentTrending.slice(0, 5)}/>
                </div>
            {/if}

            <div class="w-full px-4 md:px-12 lg:pl-32 pt-4 md:pt-6 relative z-20 space-y-4 md:space-y-6 -mt-8 md:-mt-16 pb-safe">
                <div class="hidden md:flex items-center gap-8 border-b border-border/10 pb-4">
                    {#each modes as { id, label, icon: Icon }}
                        <button
                                class="group relative flex items-center gap-2.5 py-2 transition-all duration-300"
                                onclick={() => currentMode = id}
                                aria-pressed={currentMode === id}
                        >
                            <Icon class="size-5 transition-colors {currentMode === id ? 'text-primary' : 'text-muted-foreground group-hover:text-foreground'}" />
                            <span class="text-sm font-black uppercase tracking-widest transition-colors {currentMode === id ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'}">
                                {label}
                            </span>
                            {#if currentMode === id}
                                <div
                                        class="absolute -bottom-4 left-0 right-0 h-1 bg-primary rounded-t-full"
                                        in:receive={{ key: 'tab-underline' }}
                                        out:send={{ key: 'tab-underline' }}
                                ></div>
                            {/if}
                        </button>
                    {/each}
                </div>

                {#key currentMode}
                    {#if currentSection}
                        {@const section = currentSection}
                        <div class="space-y-2 md:space-y-4">
                            {#if continueItems.length > 0}
                                <div use:reveal={{ delay: 0 }}>
                                    <ContinueCarouselCarousel items={continueItems} mode={currentMode} />
                                </div>
                            {/if}

                            {#if section.trending.length > 0}
                                <div use:reveal={{ delay: 70 }}>
                                    <ContentCardCarousel title={i18n.t("home.trending")} items={section.trending} />
                                </div>
                            {/if}

                            {#if currentMode === 'anime'}
                                <div use:reveal={{ delay: 140 }}>
                                    <RecentlyAiredCarousel/>
                                </div>
                            {/if}

                            {#if section.popular.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={i18n.t("home.popular")} items={section.popular} />
                                </div>
                            {/if}

                            {#if section.seasonal.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={currentMode === 'anime' ? i18n.t("home.simulcast") : i18n.t("home.latest")} items={section.seasonal} />
                                </div>
                            {/if}

                            {#if section.upcoming.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={i18n.t("home.upcoming")} items={section.upcoming} />
                                </div>
                            {/if}

                            {#if section.recentlyFinished.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={i18n.t("home.recently_finished")} items={section.recentlyFinished} />
                                </div>
                            {/if}

                            {#if section.topRated.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={i18n.t("home.critically_acclaimed")} items={section.topRated} />
                                </div>
                            {/if}

                            {#if currentMode === 'anime' && section.topAction.length > 0}
                                <div use:reveal>
                                    <ContentCardCarousel title={i18n.t("home.action")} items={section.topAction} />
                                </div>
                            {/if}
                        </div>
                    {/if}
                {/key}
            </div>
        </div>
    {/if}
</div>