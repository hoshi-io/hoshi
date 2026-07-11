<script lang="ts">
    import { auth } from '@/stores/auth.svelte.js';
    import { Users } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { page } from "$app/state";
    import HistoryMenu from "@/components/history/HistoryMenu.svelte";
    import { layoutState } from "@/stores/layout.svelte";

    let { mainRoutes, profileRoutes, showSwitchProfileModal = $bindable(false) } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<!-- Added 'group/sidebar' and conditional width/background handling for TV expansion -->
<aside
        class="{layoutState.isTV ? 'flex absolute top-0 left-0 bg-background/95 backdrop-blur-md shadow-2xl w-20 focus-within:w-56 hover:w-56 transition-all duration-300 ease-in-out' : 'hidden md:flex bg-transparent w-20'} flex-col h-full shrink-0 pt-8 pb-6 z-50 justify-between group/sidebar"
>

    <nav class="flex flex-col items-center space-y-4 w-full">
        <div>
            <div class="flex justify-center">
                <img
                        src="/128x128.png"
                        alt="App Logo"
                        class="size-8 opacity-80 select-none pointer-events-none rounded-lg"
                />
            </div>
        </div>

        {#each mainRoutes as route}
            {@const Icon = route.icon}
            {@const active = isActive(route.path)}
            <!-- Added outline-none and px-3 to handle focus rings cleanly -->
            <a href={route.path} class="flex w-full group outline-none px-3">
                <!-- Added focus-visible states for D-pad navigation -->
                <Button
                        variant="ghost"
                        class="w-full h-11 rounded-xl transition-all duration-300 {layoutState.isTV ? 'justify-start gap-4 overflow-hidden' : 'justify-center px-0'}
        {active ? 'text-primary bg-primary/5' : 'text-muted-foreground hover:bg-muted/30 focus-visible:bg-muted/30 focus-visible:ring-2 focus-visible:ring-primary hover:text-foreground focus-visible:text-foreground'}"
                >
                    <Icon class="shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'rotate-4 scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-8 group-hover:scale-105 group-hover:opacity-100 group-focus-visible:opacity-100 group-focus-visible:scale-105'}" />

                    <!-- Text label only rendered on TV and fades in on focus/hover -->
                    {#if layoutState.isTV}
                        <span class="whitespace-nowrap font-medium opacity-0 group-hover/sidebar:opacity-100 group-focus-within/sidebar:opacity-100 transition-opacity duration-300">
                            {route.name}
                        </span>
                    {/if}
                </Button>
            </a>
        {/each}
        <HistoryMenu />
    </nav>

    <div class="flex flex-col items-center space-y-4 px-3 w-full">
        <div class="w-full space-y-4">
            {#each profileRoutes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}
                <a href={route.path} class="flex w-full group outline-none" title={route.name}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-xl transition-all duration-300 {layoutState.isTV ? 'justify-start gap-4 overflow-hidden' : 'justify-center px-0'}
                        {active ? 'text-primary bg-primary/5' : 'text-muted-foreground hover:bg-muted/30 focus-visible:bg-muted/30 focus-visible:ring-2 focus-visible:ring-primary hover:text-foreground focus-visible:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'rotate-12 scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-4 group-hover:scale-105 group-hover:opacity-100 group-focus-visible:opacity-100'}" />

                        {#if layoutState.isTV}
                            <span class="whitespace-nowrap font-medium opacity-0 group-hover/sidebar:opacity-100 group-focus-within/sidebar:opacity-100 transition-opacity duration-300">
                                {route.name}
                            </span>
                        {/if}
                    </Button>
                </a>
            {/each}
        </div>

        {#if auth.user}
            <div class="w-full border-t border-border/10 pt-4 flex flex-col gap-4 items-center {layoutState.isTV ? 'group-hover/sidebar:flex-row group-focus-within/sidebar:flex-row group-hover/sidebar:px-2 group-focus-within/sidebar:px-2' : ''}">
                <div class="flex items-center justify-center outline-none focus-visible:ring-2 focus-visible:ring-primary rounded-full cursor-pointer" tabindex="0">
                    <Avatar.Root class="size-9 shrink-0 border border-border/20 shadow-sm transition-transform duration-300 hover:scale-105">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                </div>

                <Button
                        variant="ghost"
                        size="icon"
                        class="group size-9 rounded-xl text-muted-foreground hover:bg-muted/30 focus-visible:bg-muted/30 focus-visible:ring-2 focus-visible:ring-primary hover:text-foreground mx-auto"
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