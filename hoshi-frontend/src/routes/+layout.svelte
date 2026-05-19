<script lang="ts">
    import './layout.css';
    import { onMount } from 'svelte';
    import { afterNavigate } from '$app/navigation';
    import { page } from '$app/state';
    import { slide, fade } from 'svelte/transition';

    import {openUrl} from "@tauri-apps/plugin-opener";
    import { initApp, handleNavigation, handleDiscordActivity } from '$lib/app/app';
    import { auth } from '@/stores/auth.svelte.js';
    import { Toaster } from '$lib/components/ui/sonner';

    import DesktopTitlebar from '@/components/layout/DesktopTitlebar.svelte';
    import DesktopSidebar from '$lib/components/layout/DesktopSidebar.svelte';
    import MobileTop from '@/components/layout/MobileTop.svelte';
    import MobileBottom from '@/components/layout/MobileBottom.svelte';
    import SwitchProfile from '@/components/modals/SwitchProfile.svelte';
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { Search, Home, Calendar, Settings, List } from 'lucide-svelte';
    import { layoutState } from "@/stores/layout.svelte";
    import { setupImportListener } from "@/stores/importStatus.svelte";
    import ImportStatus from "@/components/ImportStatus.svelte";

    let { children } = $props();

    let innerWidth = $state(0);
    let isTouchDevice = $state(false);

    let isMobile = $derived(innerWidth < 1024 || isTouchDevice);

    $effect(() => {
        layoutState.isMobile = innerWidth < 1024 || isTouchDevice;
    });

    const mainRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/', icon: Home },
        { name: i18n.t('layout.search'), path: '/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('layout.settings'), path: '/settings', icon: Settings },
    ]);

    const mobileRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/', icon: Home },
        { name: i18n.t('layout.search'), path: '/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/schedule', icon: Calendar },
        { name: i18n.t('layout.settings'), path: '/settings', icon: Settings },
    ]);

    const pathname = $derived(page.url.pathname);

    const isViewer = $derived(
        pathname.startsWith('/watch/') ||
        pathname.startsWith('/read/') ||
        pathname.startsWith('/read-novel/') ||
        pathname.startsWith('/setup')
    );

    const showNav = $derived(
        auth.user !== null && !isViewer
    );

    let lastScrollY = $state(0);
    let isNavHidden = $state(false);
    let showSwitchProfileModal = $state(false);

    let mainElement: HTMLElement | null = $state(null);

    afterNavigate(() => {
        if (mainElement) {
            mainElement.scrollTo(0, 0);
        }
    });

    function handleScroll(e: Event) {
        if (!showNav || !isMobile) {
            if (isNavHidden) isNavHidden = false;
            return;
        }

        const target = e.target as HTMLElement;
        const currentScroll = target.scrollTop;

        if (currentScroll < 0) return;

        if (currentScroll > lastScrollY && currentScroll > 50) {
            isNavHidden = true;
        } else if (currentScroll < lastScrollY) {
            isNavHidden = false;
        }

        lastScrollY = currentScroll;
    }

    async function handleGlobalLinks(e: MouseEvent) {
        const target = e.target as HTMLElement;
        const anchor = target.closest('a');

        if (!anchor || !anchor.href) return;

        let url: URL;
        try {
            url = new URL(anchor.href);
        } catch (err) {
            return;
        }

        const isExternal = url.origin !== window.location.origin || url.protocol === 'mailto:';

        if (isExternal) {
            e.preventDefault();
            e.stopPropagation();

            try {
                await openUrl(anchor.href);
            } catch (err) {
                window.open(anchor.href, '_blank');
            }
        }
    }

    onMount(() => {
        initApp((v) => isTouchDevice = v);
        setupImportListener();
    });

    $effect(() => {
        handleNavigation(pathname);
    });

    function getPageLabel(path: string) {
        if (path === '/') return 'Home';
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

        const pageLabel = getPageLabel(pathname);

        handleDiscordActivity(!isViewer, pageLabel);
    });
</script>

<svelte:window bind:innerWidth />
<svelte:document onclickcapture={handleGlobalLinks} />

<div class="h-dvh w-full bg-background text-foreground flex flex-col overflow-hidden relative">

    <DesktopTitlebar />

    <div class="flex flex-1 overflow-hidden relative">

        {#if showNav}
            <div transition:slide={{axis: 'x', duration: 300}} class="absolute top-0 left-0 h-full z-50 hidden lg:block">
                <DesktopSidebar {mainRoutes} {profileRoutes} bind:showSwitchProfileModal />
            </div>
        {/if}

        <div class="flex-1 flex flex-col relative overflow-hidden bg-background">

            {#if showNav}
                <div class="w-full z-50 lg:hidden absolute top-0 left-0 transition-transform duration-300 ease-in-out {isNavHidden ? '-translate-y-full' : 'translate-y-0'}">
                    <MobileTop bind:showSwitchProfileModal />
                </div>
            {/if}

            <main
                    bind:this={mainElement}
                    class="flex-1 relative w-full h-full {isViewer ? 'overflow-hidden' : 'overflow-y-auto overflow-x-hidden touch-pan-y'} {showNav ? 'pt-24 pb-20 lg:pt-0 lg:pb-0' : ''}"
                    onscroll={handleScroll}
            >
                {#key pathname}
                    <div class="h-full" in:fade={{ duration: 150, delay: 50 }}>
                        {@render children()}
                    </div>
                {/key}
            </main>

            {#if showNav && !pathname.startsWith('/settings')}
                <div class="w-full z-50 lg:hidden absolute bottom-0 left-0 transition-transform duration-300 ease-in-out glass-panel {isNavHidden ? 'translate-y-full' : 'translate-y-0'}">
                    <MobileBottom routes={mobileRoutes} />
                </div>
            {/if}

        </div>
    </div>

    <ImportStatus />
    <Toaster />
    <SwitchProfile bind:open={showSwitchProfileModal} />

    {#if layoutState.listEditor}
        <ListEditor
                bind:open={layoutState.listEditorOpen}
                cid={layoutState.listEditor.cid}
                title={layoutState.listEditor.title}
                contentType={layoutState.listEditor.contentType}
                coverImage={layoutState.listEditor.coverImage}
        />
    {/if}
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