<script lang="ts">
    import { page } from '$app/state';
    import { fly } from 'svelte/transition';

    let { routes }: { routes: Array<{ name: string, path: string, icon: any, key?: string }> } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<nav class="lg:hidden fixed bottom-0 z-50 w-full pb-safe pointer-events-none">
    <div class="mx-4 mb-4 rounded-3xl border border-border/40 bg-background/85 backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.2)] overflow-hidden pointer-events-auto">
        <div class="flex items-center justify-around h-15 px-2">
            {#each routes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}

                <a href={route.path}
                   class="relative flex flex-col items-center justify-center flex-1 h-full gap-1 select-none transition-colors duration-300
                   {active ? 'text-primary' : 'text-muted-foreground/50 active:text-muted-foreground'}"
                >
                    {#if active}
                        <div
                                class="absolute inset-x-1.5 inset-y-1 rounded-xl bg-primary/8 border border-primary/10"
                                in:fly={{ y: 2, duration: 180 }}
                        ></div>
                    {/if}

                    <div class="relative flex items-center justify-center transition-transform duration-300 {active ? 'scale-105 -translate-y-0.5' : 'scale-100'}">
                        <Icon
                                class="size-[21px] transition-all duration-300"
                                stroke-width={active ? 2.25 : 1.75}
                        />
                    </div>

                    <span
                            class="relative text-[9px] font-bold tracking-normal transition-all duration-300 long-press-none
                        {active ? 'opacity-100 scale-100 font-extrabold' : 'opacity-60 scale-95 font-medium text-muted-foreground/70'}"
                    >
                        {route.name}
                    </span>
                </a>
            {/each}
        </div>
    </div>
</nav>

<style>
    .pb-safe {
        padding-bottom: env(safe-area-inset-bottom, 0px);
    }
</style>