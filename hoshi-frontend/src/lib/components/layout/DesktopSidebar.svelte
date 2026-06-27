<script lang="ts">
    import { auth } from '@/stores/auth.svelte.js';
    import { Users } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { page } from "$app/state";
    import HistoryMenu from "@/components/history/HistoryMenu.svelte";

    let { mainRoutes, profileRoutes, showSwitchProfileModal = $bindable(false) } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<aside
        class="hidden md:flex flex-col h-full shrink-0 bg-transparent w-20 pt-8 pb-6 z-50 justify-between"
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
            <a href={route.path} class="flex justify-center w-full group">
                <Button
                        variant="ghost"
                        class="size-11 rounded-xl transition-all duration-300
        {active ? 'text-primary bg-primary/5' : 'text-muted-foreground hover:bg-muted/30 hover:text-foreground'}"
                >
                    <Icon class="shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'rotate-4 scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-8 group-hover:scale-105 group-hover:opacity-100'}" />
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
                <a href={route.path} class="block w-full relative group" title={route.name}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-xl transition-all duration-300 justify-center px-0
                        {active ? 'text-primary bg-primary/5' : 'text-muted-foreground hover:bg-muted/30 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5.5 transition-transform duration-300 ease-out
    {active
        ? 'rotate-12 scale-105 opacity-100'
        : 'opacity-70 group-hover:rotate-4 group-hover:scale-105 group-hover:opacity-100'}" />
                    </Button>
                </a>
            {/each}
        </div>

        {#if auth.user}
            <div class="w-full border-t border-border/10 pt-4 flex flex-col gap-4 items-center">
                <div class="flex items-center justify-center">
                    <Avatar.Root class="size-9 shrink-0 border border-border/20 shadow-sm transition-transform duration-300 hover:scale-105 cursor-pointer">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                </div>

                <Button
                        variant="ghost"
                        size="icon"
                        class="group size-9 rounded-xl text-muted-foreground hover:bg-muted/30 hover:text-foreground mx-auto"
                        onclick={(e) => {
                        e.stopPropagation();
                        showSwitchProfileModal = true;
                    }}
                        title={i18n.t('layout.switch_profile')}
                >
                    <Users class="size-4.5 shrink-0 transition-transform duration-300 ease-out group-hover:rotate-12" />
                </Button>
            </div>
        {/if}
    </div>
</aside>