<script lang="ts">
    import { Skeleton } from '@/components/ui/skeleton';
    import HomeHero from '@/components/hero/HomeHero.svelte';
    import HomeSectionView from '@/components/home/HomeSectionView.svelte';
    import { tabIcon } from '@/components/home/icons';
    import { crossfade, fade, slide } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import type { Action } from 'svelte/action';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { auth } from '@/stores/auth.svelte.js';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import { homeEditor } from '@/app/home-editor.svelte.js';
    import HomeEditView from '@/components/home/editor/HomeEditView.svelte';
    import Pencil from 'phosphor-svelte/lib/PencilSimpleIcon';
    import { beforeNavigate } from '$app/navigation';

    const prefersReducedMotion =
        typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

    const [send, receive] = crossfade({
        duration: prefersReducedMotion ? 0 : 300,
        easing: cubicOut,
        fallback: (node) => fade(node, { duration: 300 })
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

    const activeTab = $derived(homeState.activeTab);
    const visibleSections = $derived(activeTab?.sections.filter(s => !s.hidden) ?? []);
    const heroSection = $derived(homeState.heroSection(activeTab));
    const heroFeed = $derived(
        heroSection?.source.kind === 'search' ? homeState.feed(heroSection.source.query) : null
    );

    // The slot stays while the hero loads, so items arriving don't cause a second jump.
    const hasHero = $derived(!!heroFeed && !heroFeed.error && (heroFeed.items.length > 0 || !heroFeed.loaded));
    const heroItems = $derived(heroFeed?.items.slice(0, 5) ?? []);

    $effect(() => {
        heroFeed?.load();
    });

    $effect(() => {
        layoutState.title = i18n.t("home.title");
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => { layoutState.headerAction = undefined; };
    });

    // Leaving home drops an unsaved draft. Not tied to this component's lifetime:
    // the layout can remount the page while navigating to it, which must not end editing.
    beforeNavigate(({ to }) => {
        if (homeEditor.editing && to?.url.pathname !== '/') homeEditor.cancel();
    });

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

{#snippet editButton(extra: string)}
    <button
            type="button"
            onclick={() => homeEditor.start()}
            title={i18n.t('home.edit.customize')}
            aria-label={i18n.t('home.edit.customize')}
            class="flex shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground {extra}"
    >
        <Pencil class="size-4" />
    </button>
{/snippet}

{#snippet mobileHeaderAction()}
    {#if !homeEditor.editing}
    <div class="flex items-center gap-1 mr-1" in:fade={{duration: 150}}>
    <div class="flex max-w-[55vw] items-center overflow-x-auto [scrollbar-width:none] bg-muted/40 p-1 rounded-xl border border-border/40 shadow-inner h-10">
        {#each homeState.tabs as tab (tab.id)}
            {@const Icon = tabIcon(tab.icon)}
            {@const label = labelText(tab.title)}
            <button
                    class="relative flex shrink-0 items-center justify-center px-3 sm:px-4 h-full rounded-lg text-xs font-bold transition-colors duration-300 {activeTab?.id === tab.id ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => homeState.activeTabId = tab.id}
                    aria-label={label}
                    aria-pressed={activeTab?.id === tab.id}
            >
                {#if activeTab?.id === tab.id}
                    <span
                            class="absolute inset-0 rounded-lg bg-background shadow-sm ring-1 ring-border/30"
                            in:receive={{ key: 'mode-pill' }}
                            out:send={{ key: 'mode-pill' }}
                    ></span>
                {/if}
                <Icon class="relative w-4.5 h-4.5 transition-transform duration-300 {activeTab?.id === tab.id ? 'scale-110' : ''} sm:mr-1.5" />
                <span class="relative hidden sm:inline-block tracking-wide">{label}</span>
            </button>
        {/each}
    </div>
    {@render editButton('size-10 md:hidden')}
    </div>
    {/if}
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
    {:else if homeEditor.editing}
        <div class="mx-auto w-full max-w-[2000px] px-4 pt-28 pb-safe md:px-8 lg:pl-32 lg:pr-12 lg:pt-6">
            <HomeEditView />
        </div>
    {:else if activeTab}
        <div in:fade={{ duration: 500 }}>
            {#if hasHero}
                <div class="w-full relative overflow-hidden" transition:slide={{ duration: prefersReducedMotion ? 0 : 450, easing: cubicOut }}>
                    {#key heroSection?.id}
                        <div in:fade={{ duration: 400 }}>
                            {#if heroItems.length > 0}
                                <HomeHero items={heroItems}/>
                            {:else}
                                <div class="w-full h-[75vh] md:h-[90vh] min-h-[550px] bg-card/30"></div>
                            {/if}
                        </div>
                    {/key}
                </div>
            {:else}
                <div class="h-24 lg:h-8" transition:slide={{ duration: prefersReducedMotion ? 0 : 450, easing: cubicOut }}></div>
            {/if}

            <div class="w-full px-4 md:px-12 lg:pl-32 pt-4 md:pt-6 relative z-20 space-y-4 md:space-y-6 pb-safe transition-[margin] duration-500 {hasHero ? '-mt-8 md:-mt-16' : ''}">
                <div class="hidden md:flex items-center gap-8 border-b border-border/10 pb-4 overflow-x-auto [scrollbar-width:none]">
                    {#each homeState.tabs as tab (tab.id)}
                        {@const Icon = tabIcon(tab.icon)}
                        <button
                                class="group relative flex shrink-0 items-center gap-2.5 py-2 transition-all duration-300"
                                onclick={() => homeState.activeTabId = tab.id}
                                aria-pressed={activeTab.id === tab.id}
                        >
                            <Icon class="size-5 transition-colors {activeTab.id === tab.id ? 'text-primary' : 'text-muted-foreground group-hover:text-foreground'}" />
                            <span class="text-sm font-black uppercase tracking-widest transition-colors {activeTab.id === tab.id ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'}">
                                {labelText(tab.title)}
                            </span>
                            {#if activeTab.id === tab.id}
                                <div
                                        class="absolute -bottom-4 left-0 right-0 h-1 bg-primary rounded-t-full"
                                        in:receive={{ key: 'tab-underline' }}
                                        out:send={{ key: 'tab-underline' }}
                                ></div>
                            {/if}
                        </button>
                    {/each}
                    <div class="ml-auto">{@render editButton('size-9')}</div>
                </div>

                {#key activeTab.id}
                    <div class="space-y-2 md:space-y-4">
                        {#each visibleSections as section, i (section.id)}
                            <div use:reveal={{ delay: Math.min(i, 2) * 70 }}>
                                <HomeSectionView {section} />
                            </div>
                        {/each}
                    </div>
                {/key}
            </div>
        </div>
    {/if}
</div>
