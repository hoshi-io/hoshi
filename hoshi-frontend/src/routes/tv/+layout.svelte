<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { afterNavigate } from '$app/navigation';
    import { page } from '$app/state';
    import { fade } from 'svelte/transition';
    import { Search, Home, Calendar, Settings, List } from 'lucide-svelte';

    import { auth } from '@/stores/auth.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { enableSpatialNav, disableSpatialNav, focusFirstIn } from '@/tv/spatialNav';
    import TvSidebar from '@/tv/TvSidebar.svelte';

    let { children } = $props();

    const mainRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/tv', icon: Home },
        { name: i18n.t('layout.search'), path: '/tv/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/tv/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/tv/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('layout.settings'), path: '/tv/settings', icon: Settings }
    ]);

    const pathname = $derived(page.url.pathname);
    const isSetup = $derived(pathname.startsWith('/tv/setup'));
    const showNav = $derived(auth.user !== null && !isSetup);

    // Top-level screens: Back on these goes to the sidebar / exits, not to the previous page.
    const isSection = $derived([...mainRoutes, ...profileRoutes].some((r) => r.path === pathname));

    let mainElement: HTMLElement | null = $state(null);

    onMount(() => {
        enableSpatialNav();
        return disableSpatialNav;
    });

    function focusSidebar() {
        const el =
            document.querySelector<HTMLElement>('[data-tv-sidebar] [aria-current="page"]') ??
            document.querySelector<HTMLElement>('[data-tv-sidebar] [data-sn]');
        el?.focus();
    }

    function focusContent() {
        if (!mainElement) return;
        const preferred = mainElement.querySelector<HTMLElement>('[data-sn][data-autofocus]');
        if (preferred) preferred.focus();
        else focusFirstIn(mainElement);
    }

    afterNavigate(async () => {
        mainElement?.scrollTo(0, 0);
        await tick();
        // Only steal focus when the user was in the sidebar (or nowhere); never from content.
        const active = document.activeElement;
        if (!active || active === document.body || active.closest('[data-tv-sidebar]')) focusContent();
    });

    function onKeydown(e: KeyboardEvent) {
        if (isSetup) return; // the setup page handles its own Back (previous step / exit)
        const isBack =
            e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;
        if (!isBack) return;

        const inContent = mainElement?.contains(document.activeElement);

        if (!isSection) {
            e.preventDefault();
            history.back();
        } else if (inContent && showNav) {
            e.preventDefault();
            focusSidebar();
        }
        // else: already in sidebar on a top-level screen -> let the system exit the app
    }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="h-dvh w-full bg-background text-foreground relative overflow-hidden select-none">
    {#if showNav}
        <TvSidebar {mainRoutes} {profileRoutes} />
    {/if}

    <main
            bind:this={mainElement}
            class="h-full w-full overflow-y-auto overflow-x-hidden {showNav ? 'pl-20' : ''}"
    >
        {#key pathname}
            <!-- px/py = overscan-safe margins -->
            <div class="min-h-full px-12 py-8" in:fade={{ duration: 120 }}>
                {@render children()}
            </div>
        {/key}
    </main>
</div>

<style>
    :global(html, body) {
        overflow: hidden;
        height: 100%;
        width: 100%;
        cursor: none;
    }
    :global(main::-webkit-scrollbar) {
        display: none;
    }
</style>