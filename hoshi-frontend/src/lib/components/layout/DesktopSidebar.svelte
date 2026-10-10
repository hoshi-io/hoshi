<script lang="ts">
    import { auth } from '@/stores/auth.svelte.js';
    import { Users, Code2 } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { page } from "$app/state";
    import HistoryMenu from "@/components/history/HistoryMenu.svelte";
    import { layoutState } from "@/stores/layout.svelte";
    import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
    import { crossfade, fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';

    let { mainRoutes, profileRoutes, showSwitchProfileModal = $bindable(false) } = $props();

    const prefersReducedMotion =
        typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

    const [send, receive] = crossfade({
        duration: prefersReducedMotion ? 0 : 260,
        easing: cubicOut,
        fallback: fade
    });

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }

    const itemBase =
        "relative flex h-11 w-full items-center rounded-sm outline-none cursor-default select-none " +
        "[-webkit-user-drag:none] transition-[color,background-color,transform] duration-200 " +
        "active:scale-[0.96] focus-visible:ring-2 focus-visible:ring-primary";
    const itemIdle =
        "text-muted-foreground hover:bg-muted/30 hover:text-foreground " +
        "focus-visible:bg-muted/30 focus-visible:text-foreground";
    const itemActive = "text-primary";
    const itemLayout = $derived(layoutState.isTV ? 'justify-start gap-4 overflow-hidden px-4' : 'justify-center');

    const tvLabel =
        "relative whitespace-nowrap font-medium opacity-0 group-hover/sidebar:opacity-100 " +
        "group-focus-within/sidebar:opacity-100 transition-opacity duration-300";

    async function openIde() {
        const existing = await WebviewWindow.getByLabel('extension-ide');
        if (existing) {
            await existing.setFocus();
            return;
        }

        const ideWindow = new WebviewWindow('extension-ide', {
            url: '/ide',
            title: 'Extension IDE',
            width: 1200,
            height: 800,
            decorations: false
        });

        ideWindow.once('tauri://error', (e) => {
            console.error('Failed to open IDE window:', e);
        });
    }
</script>

<aside
        class="{layoutState.isTV ? 'flex absolute top-0 left-0 bg-background/95 backdrop-blur-md shadow-2xl w-20 focus-within:w-56 hover:w-56 transition-all duration-300 ease-in-out' : 'hidden md:flex bg-transparent w-20'} flex-col h-full shrink-0 pt-8 pb-6 z-50 justify-between group/sidebar select-none"
>

    <nav class="flex flex-col items-center space-y-4 w-full">
        <div>
            <div class="flex justify-center">
                <img
                        src="/128x128.png"
                        alt="App Logo"
                        draggable="false"
                        class="size-8 opacity-80 select-none pointer-events-none rounded-lg"
                />
            </div>
        </div>
        {#each mainRoutes as route}
            {@const Icon = route.icon}
            {@const active = isActive(route.path)}
            <div class="relative w-full px-3">

                <a
                        href={route.path}
                        draggable="false"
                        title={layoutState.isTV ? undefined : route.name}
                        aria-current={active ? 'page' : undefined}
                        class="group {itemBase} {itemLayout} {active ? itemActive : itemIdle}"
                >
                    {#if active}
                        <span
                                aria-hidden="true"
                                class="absolute inset-0 rounded-sm bg-primary/10"
                                in:receive={{ key: 'rail-pill' }}
                                out:send={{ key: 'rail-pill' }}
                        ></span>
                    {/if}

                    <Icon class="relative shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-8 group-hover:scale-105 group-hover:opacity-100 group-focus-visible:opacity-100 group-focus-visible:scale-105'}" />

                    {#if layoutState.isTV}
                        <span class={tvLabel}>{route.name}</span>
                    {/if}
                </a>
            </div>
        {/each}
        <HistoryMenu />
    </nav>

    <div class="flex flex-col px-3 w-full">
        <div class="w-full space-y-4">
            {#each profileRoutes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}
                <div class="relative w-full">
                    {#if active}
                        <span
                                aria-hidden="true"
                                class="absolute -left-3 top-3 h-5 w-[3px] rounded-r-full bg-primary"
                                in:receive={{ key: 'rail-bar' }}
                                out:send={{ key: 'rail-bar' }}
                        ></span>
                    {/if}

                    <a
                            href={route.path}
                            draggable="false"
                            title={layoutState.isTV ? undefined : route.name}
                            aria-current={active ? 'page' : undefined}
                            class="group {itemBase} {itemLayout} {active ? itemActive : itemIdle}"
                    >
                        {#if active}
                            <span
                                    aria-hidden="true"
                                    class="absolute inset-0 rounded-sm bg-primary/10"
                                    in:receive={{ key: 'rail-pill' }}
                                    out:send={{ key: 'rail-pill' }}
                            ></span>
                        {/if}

                        <Icon class="relative shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-4 group-hover:scale-105 group-hover:opacity-100 group-focus-visible:opacity-100'}" />

                        {#if layoutState.isTV}
                            <span class={tvLabel}>{route.name}</span>
                        {/if}
                    </a>
                </div>
            {/each}

            <button
                    onclick={openIde}
                    title={layoutState.isTV ? undefined : 'Extension IDE'}
                    class="group {itemBase} {itemLayout} {itemIdle}"
            >
                <Code2 class="relative shrink-0 size-5.5 opacity-70 transition-transform duration-300 ease-out group-hover:rotate-4 group-hover:scale-105 group-hover:opacity-100 group-focus-visible:opacity-100" />

                {#if layoutState.isTV}
                    <span class={tvLabel}>Extension IDE</span>
                {/if}
            </button>
        </div>

        {#if auth.user}
            <div class="w-full border-t border-border/10 pt-4 flex flex-col gap-4 items-center {layoutState.isTV ? 'group-hover/sidebar:flex-row group-focus-within/sidebar:flex-row group-hover/sidebar:px-2 group-focus-within/sidebar:px-2' : ''}">
                <div class="flex items-center justify-center rounded-full">
                    <Avatar.Root class="size-9 shrink-0 border border-border/20 shadow-sm transition-transform duration-300 hover:scale-105">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} draggable="false" />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                </div>

                <Button
                        variant="ghost"
                        size="icon"
                        class="group size-9 rounded-sm cursor-default active:scale-[0.94] transition-[color,background-color,transform] duration-200 text-muted-foreground hover:bg-muted/30 focus-visible:bg-muted/30 focus-visible:ring-2 focus-visible:ring-primary hover:text-foreground focus-visible:text-foreground mx-auto"
                        onclick={(e) => {
                        e.stopPropagation();
                        showSwitchProfileModal = true;
                    }}
                        title={i18n.t('layout.switch_profile')}
                >
                    <Users class="size-4.5 shrink-0 transition-transform duration-300 ease-out group-hover:rotate-12 group-focus-visible:rotate-12" />
                </Button>
            </div>
        {/if}
    </div>
</aside>