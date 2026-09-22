<script lang="ts">
    import { page } from '$app/state';

    let { routes, isScrolled = false }: {
        routes: Array<{ name: string, path: string, icon: any, key?: string }>,
        isScrolled?: boolean
    } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }

    let activeIndex = $derived(routes.findIndex(r => isActive(r.path)));
    let safeActiveIndex = $derived(activeIndex === -1 ? 0 : activeIndex);

    let bounce = $state(false);

    function handleNavClick() {
        bounce = true;
        setTimeout(() => bounce = false, 150);
    }
</script>

<nav class="lg:hidden fixed bottom-0 z-50 w-full pb-safe pointer-events-none flex justify-center">
    <div class="transition-all duration-300 ease-out overflow-hidden pointer-events-auto bg-background/85 backdrop-blur-2xl border border-border/40 shadow-[0_8px_32px_rgba(0,0,0,0.2)] transform-gpu rounded-full
        {bounce ? 'scale-[1.02]' : 'scale-100'}
        {isScrolled ? 'mb-3 w-[85%]' : 'mb-4 w-[calc(100%-2rem)]'}"
    >
        <div class="relative flex items-center justify-around transition-all duration-300 {isScrolled ? 'h-12' : 'h-[60px]'}">

            <!-- Sliding Pill Highlight (Uses unified rounded-full) -->
            <div class="absolute inset-y-0 left-0 flex items-center justify-center transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] z-0"
                 style="width: {100 / routes.length}%; transform: translateX({safeActiveIndex * 100}%);">
                <div class="w-[calc(100%-10px)] h-[calc(100%-10px)] bg-primary/15 rounded-full transition-all duration-300">
                </div>
            </div>

            {#each routes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}

                <a href={route.path}
                   onclick={handleNavClick}
                   class="relative z-10 flex flex-col items-center justify-center flex-1 h-full gap-0 select-none transition-colors duration-300
                   {active ? 'text-primary' : 'text-muted-foreground/50 active:text-muted-foreground'}"
                >
                    <div class="relative flex items-center justify-center transition-transform duration-300
                        {active ? (isScrolled ? 'scale-110' : 'scale-110 -translate-y-[1px]') : 'scale-100'}"
                    >
                        <Icon
                                class="transition-all duration-300 {isScrolled ? 'size-[18px]' : 'size-[20px]'}"
                                stroke-width={active ? 2.25 : 1.75}
                        />
                    </div>

                    <div class="grid transition-all duration-300 {isScrolled ? 'grid-rows-[0fr] opacity-0' : 'grid-rows-[1fr] opacity-100'}">
                        <span class="overflow-hidden text-[9px] font-bold tracking-normal transition-all duration-300 long-press-none
                            {active ? 'opacity-100 font-extrabold' : 'opacity-60 font-medium text-muted-foreground/70'}"
                        >
                            {route.name}
                        </span>
                    </div>
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