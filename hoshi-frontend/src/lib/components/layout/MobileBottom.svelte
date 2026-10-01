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
    <div class="mb-3 relative overflow-hidden pointer-events-auto bg-background/85 backdrop-blur-xl border border-border/40 shadow-[0_8px_32px_rgba(0,0,0,0.2)] rounded-full
        transform-gpu will-change-[width,height]
        transition-[width,height,transform] duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] motion-reduce:transition-none
        {bounce ? 'scale-[1.02]' : 'scale-100'}
        {isScrolled ? 'w-[85%] h-12' : 'w-[calc(100%-2rem)] h-[60px]'}"
    >
        <div class="relative flex items-center justify-around h-full">

            <div class="absolute inset-y-0 left-0 flex items-center justify-center z-0 will-change-transform
                        transition-transform duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] motion-reduce:transition-none"
                 style="width: {100 / routes.length}%; transform: translateX({safeActiveIndex * 100}%);">
                <div class="w-[calc(100%-10px)] h-[calc(100%-10px)] bg-primary/15 rounded-full"></div>
            </div>

            {#each routes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}

                <a href={route.path}
                   onclick={handleNavClick}
                   class="relative z-10 flex items-center justify-center flex-1 h-full select-none transition-colors duration-300
                   {active ? 'text-primary' : 'text-muted-foreground/50 active:text-muted-foreground'}"
                >
                    <div class="flex items-center justify-center will-change-transform
                        transition-transform duration-300 ease-[cubic-bezier(0.32,0.72,0,1)] motion-reduce:transition-none
                        {active ? 'scale-110' : 'scale-100'}
                        {isScrolled ? 'translate-y-0' : '-translate-y-2'}"
                    >
                        <Icon class="size-[20px]" stroke-width={active ? 2.25 : 1.75} />
                    </div>

                    <span class="absolute bottom-[8px] inset-x-0 text-center leading-3 text-[9px] tracking-normal pointer-events-none long-press-none
                        transition-[opacity,transform] duration-300 ease-out motion-reduce:transition-none
                        {isScrolled ? 'opacity-0 translate-y-1' : (active ? 'opacity-100 font-extrabold' : 'opacity-60 font-medium text-muted-foreground/70')}
                        {active ? 'font-extrabold' : 'font-medium'}"
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