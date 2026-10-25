<script lang="ts">
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { Users, Monitor } from 'lucide-svelte';
    import * as Avatar from '$lib/components/ui/avatar';
    import { auth } from '@/stores/auth.svelte.js';
    import { layoutState } from '@/stores/layout.svelte';

    let { mainRoutes, profileRoutes } = $props();

    const pathname = $derived<string>(page.url.pathname);

    function isActive(path: string) {
        return path === '/tv' ? pathname === '/tv' : pathname.startsWith(path);
    }

    const item =
        'relative flex h-12 w-full items-center gap-4 rounded-md px-4 outline-none ' +
        'transition-colors duration-150 text-muted-foreground ' +
        'focus:bg-primary focus:text-primary-foreground focus:ring-2 focus:ring-primary/60';
    const itemActive = 'text-primary bg-primary/10';

    // Labels only appear while the rail is expanded (focus inside it).
    const label =
        'whitespace-nowrap font-medium opacity-0 transition-opacity duration-200 ' +
        'group-focus-within/rail:opacity-100 group-hover/rail:opacity-100';
</script>

<!-- Overlays the content (absolute) so expanding never reflows the page behind it. -->
<aside
        data-tv-sidebar
        class="group/rail absolute top-0 left-0 z-50 flex h-full w-20 flex-col justify-between
           border-r border-border/10 bg-background pt-8 pb-6
           transition-[width] duration-200 ease-out
           focus-within:w-64 hover:w-64 focus-within:shadow-2xl"
>
    <nav class="flex w-full flex-col items-stretch gap-3 px-3">
        <div class="mb-4 flex justify-center">
            <img src="/128x128.png" alt="" draggable="false" class="size-9 rounded-lg opacity-90" />
        </div>

        {#each mainRoutes as route}
            {@const Icon = route.icon}
            <a
                    href={route.path}
                    data-sn
                    aria-current={isActive(route.path) ? 'page' : undefined}
                    class="{item} {isActive(route.path) ? itemActive : ''}"
            >
                <Icon class="size-6 shrink-0" />
                <span class={label}>{route.name}</span>
            </a>
        {/each}
    </nav>

    <div class="flex w-full flex-col items-stretch gap-3 px-3">
        {#each profileRoutes as route}
            {@const Icon = route.icon}
            <a
                    href={route.path}
                    data-sn
                    aria-current={isActive(route.path) ? 'page' : undefined}
                    class="{item} {isActive(route.path) ? itemActive : ''}"
            >
                <Icon class="size-6 shrink-0" />
                <span class={label}>{route.name}</span>
            </a>
        {/each}

        <!-- Only offered when the TV layout is shown on a non-TV device (desktop testing/toggle) -->
        {#if !layoutState.isAndroidTv}
            <button data-sn class={item} onclick={() => goto('/')}>
                <Monitor class="size-6 shrink-0" />
                <span class={label}>Desktop layout</span>
            </button>
        {/if}

        {#if auth.user}
            <button data-sn class="{item} mt-1 border-t border-border/10 pt-0" onclick={() => (layoutState.switchProfileOpen = true)}>
                <Avatar.Root class="size-7 shrink-0">
                    <Avatar.Image src={auth.user.avatar} alt="" draggable="false" />
                    <Avatar.Fallback class="bg-primary/10 text-xs font-bold text-primary">
                        {auth.user.username[0].toUpperCase()}
                    </Avatar.Fallback>
                </Avatar.Root>
                <span class={label}>{auth.user.username}</span>
                <Users class="ml-auto size-5 shrink-0 opacity-0 group-focus-within/rail:opacity-70" />
            </button>
        {/if}
    </div>
</aside>