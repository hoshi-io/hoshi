<script lang="ts">
    import './layout.css';
    import { untrack } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { openUrl } from '@tauri-apps/plugin-opener';
    import { check } from '@smbcloud/tauri-plugin-android-tv-check-api';

    import { initApp, handleNavigation } from '$lib/app/app';
    import { setupImportListener } from '@/stores/importStatus.svelte';
    import { layoutState } from '@/stores/layout.svelte';
    import { Toaster } from '$lib/components/ui/sonner';
    import ImportStatus from '@/components/ImportStatus.svelte';
    import SwitchProfile from '@/components/modals/SwitchProfile.svelte';
    import ListEditor from '@/components/modals/ListEditor.svelte';

    let { children } = $props();

    const pathname = $derived(page.url.pathname);
    const isIde = $derived(pathname.startsWith('/ide'));

    // "TV layout is showing" is just a function of the route, so the desktop
    // can open /tv manually and everything (spatial nav, etc.) follows.
    $effect(() => {
        layoutState.isTV = pathname.startsWith('/tv');
    });

    // Gate children until detection finished, so a TV box never flashes the desktop layout.
    let ready = $state(false);
    let started = false;

    $effect(() => {
        if (isIde) {
            ready = true;
            return;
        }
        if (started) return;
        started = true;
        untrack(boot);
        console.log(layoutState.isAndroidTv)
    });

    async function boot() {
        initApp((v) => (layoutState.isTouchDevice = v));
        setupImportListener();

        try {
            const result = await check();
            layoutState.isAndroidTv = result.isAndroidTv;
            if (result.isAndroidTv && !page.url.pathname.startsWith('/tv')) {
                await goto('/tv', { replaceState: true });
            }
        } catch {
            // plugin not available (desktop / iOS): not a TV
        }
        ready = true;
    }

    // Wait for `ready` so isAndroidTv is known before any setup/login redirect decides /setup vs /tv/setup.
    $effect(() => {
        if (isIde || !ready) return;
        handleNavigation(pathname);
    });

    async function handleGlobalLinks(e: MouseEvent) {
        if (isIde) return;
        const anchor = (e.target as HTMLElement).closest('a');
        if (!anchor?.href) return;

        let url: URL;
        try {
            url = new URL(anchor.href);
        } catch {
            return;
        }

        const isExternal = url.origin !== window.location.origin || url.protocol === 'mailto:';
        if (!isExternal) return;

        e.preventDefault();
        e.stopPropagation();
        try {
            await openUrl(anchor.href);
        } catch {
            window.open(anchor.href, '_blank');
        }
    }
</script>

<svelte:document onclickcapture={handleGlobalLinks} />

{#if ready}
    {@render children()}
{/if}

{#if !isIde}
    <ImportStatus />
    <Toaster />
    <SwitchProfile bind:open={layoutState.switchProfileOpen} />

    {#if layoutState.listEditor}
        <ListEditor
                bind:open={layoutState.listEditorOpen}
                cid={layoutState.listEditor.cid}
                title={layoutState.listEditor.title}
                contentType={layoutState.listEditor.contentType}
                coverImage={layoutState.listEditor.coverImage}
        />
    {/if}
{/if}