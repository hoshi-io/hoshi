<script lang="ts">
    import { afterNavigate } from '$app/navigation';
    import { page } from '$app/state';
    import { slide, fade } from 'svelte/transition';
    import { Search, Home, Calendar, Settings, List } from 'lucide-svelte';

    import { handleDiscordActivity } from '$lib/app/app';
    import { auth } from '@/stores/auth.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { layoutState } from '@/stores/layout.svelte';

    import DesktopTitlebar from '@/components/layout/DesktopTitlebar.svelte';
    import DesktopSidebar from '$lib/components/layout/DesktopSidebar.svelte';
    import MobileTop from '@/components/layout/MobileTop.svelte';
    import MobileBottom from '@/components/layout/MobileBottom.svelte';

    let { children } = $props();

    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024 || layoutState.isTouchDevice);

    $effect(() => {
        layoutState.isMobile = isMobile;
    });

    const mainRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/', icon: Home },
        { name: i18n.t('layout.search'), path: '/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('layout.settings'), path: '/settings', icon: Settings }
    ]);

    const mobileRoutes = $derived([...mainRoutes, ...profileRoutes]);

    const pathname = $derived(page.url.pathname);

    const isViewer = $derived(
        pathname.startsWith('/watch/') ||
        pathname.startsWith('/read/') ||
        pathname.startsWith('/read-novel/') ||
        pathname.startsWith('/setup')
    );

    const showNav = $derived(auth.user !== null && !isViewer);

    let lastScrollY = $state(0);
    let isNavHidden = $state(false);
    let mainElement: HTMLElement | null = $state(null);
    let mobileTopHeight = $state(0);

    afterNavigate(() => {
        mainElement?.scrollTo(0, 0);
    });

    function handleScroll(e: Event) {
        if (!showNav || !isMobile) {
            if (isNavHidden) isNavHidden = false;
            return;
        }

        const currentScroll = (e.target as HTMLElement).scrollTop;
        if (currentScroll < 0) return;

        if (currentScroll > lastScrollY && currentScroll > 50) isNavHidden = true;
        else if (currentScroll < lastScrollY) isNavHidden = false;

        lastScrollY = currentScroll;
    }

    function getPageLabel(path: string) {
        if (path.startsWith('/search')) return 'Search';
        if (path.startsWith('/list')) return 'List';
        if (path.startsWith('/schedule')) return 'Schedule';
        if (path.startsWith('/settings')) return 'Settings';
        if (path.startsWith('/setup')) return 'Setup';
        if (path.startsWith('/c')) return 'Details';
        return 'Home';
    }

    $effect(() => {
        if (!auth.initialized || !auth.user) return;
        handleDiscordActivity(!isViewer, getPageLabel(pathname));
    });
</script>

<svelte:window bind:innerWidth />

<div
        class="h-dvh w-full {isViewer ? 'bg-transparent' : 'bg-background'} text-foreground flex flex-col overflow-hidden relative"
        style="--mobile-top-h: {mobileTopHeight || 96}px"
>
    <DesktopTitlebar />

    <div class="flex flex-1 overflow-hidden relative">
        {#if showNav}
            <div transition:slide={{ axis: 'x', duration: 300 }} class="absolute top-0 left-0 h-full z-50 hidden lg:block">
                <DesktopSidebar {mainRoutes} {profileRoutes} bind:showSwitchProfileModal={layoutState.switchProfileOpen} />
            </div>
        {/if}

        <div class="flex-1 flex flex-col relative overflow-hidden {isViewer ? 'bg-transparent' : 'bg-background'}">
            {#if showNav}
                <div
                        bind:clientHeight={mobileTopHeight}
                        class="w-full z-50 lg:hidden absolute top-0 left-0 transition-transform duration-300 ease-in-out {isNavHidden ? '-translate-y-full' : 'translate-y-0'}"
                >
                    <MobileTop bind:showSwitchProfileModal={layoutState.switchProfileOpen} />
                </div>
            {/if}

            <main
                    bind:this={mainElement}
                    class="flex-1 relative w-full h-full {isViewer ? 'overflow-hidden' : 'overflow-y-auto overflow-x-hidden touch-pan-y'}
                    {showNav ? 'pt-24 pb-20 lg:pt-0 lg:pb-0' : ''}"
                    onscroll={handleScroll}
            >
                {#key pathname}
                    <div class="h-full" in:fade={{ duration: 150, delay: 50 }}>
                        {@render children()}
                    </div>
                {/key}
            </main>

            {#if showNav && !pathname.startsWith('/settings')}
                <div class="w-full z-50 lg:hidden absolute bottom-0 left-0">
                    <MobileBottom routes={mobileRoutes} isScrolled={isNavHidden} />
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    :global(html, body) {
        overflow: hidden;
        height: 100%;
        width: 100%;
    }

    @media (hover: none) and (pointer: coarse) {
        :global(html, body) {
            touch-action: pan-x pan-y;
        }
        :global(main::-webkit-scrollbar) {
            display: none;
        }
        :global(main) {
            -ms-overflow-style: none;
            scrollbar-width: none;
        }
    }
</style>