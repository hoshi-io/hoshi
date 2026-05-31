<script lang="ts">
    import { browser } from '$app/environment';
    import { type } from '@tauri-apps/plugin-os';
    import { Minus, Square, X, MonitorPlay, ChevronDown } from 'lucide-svelte';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { themeManager } from '@/stores/theme.svelte.js';
    import MpvPopover from '@/components/mpv/MpvPopover.svelte';

    const isTauri = browser && '__TAURI__' in window;
    const osType = isTauri ? type() : null;
    const showTitlebar = isTauri && osType !== 'android' && osType !== 'ios';

    let winPromise: Promise<any> | null = null;
    let popoverOpen = $state(false);

    function getWin() {
        if (!showTitlebar) return null;
        if (!winPromise) {
            winPromise = import('@tauri-apps/api/window').then(m => m.getCurrentWindow());
        }
        return winPromise;
    }

    async function minimize() { const win = await getWin(); win?.minimize(); }
    async function maximize() {
        const win = await getWin();
        if (!win) return;
        const maximized = await win.isMaximized();
        maximized ? await win.unmaximize() : await win.maximize();
    }
    async function close() { const win = await getWin(); win?.close(); }
</script>

{#if showTitlebar}
    <div class="absolute top-0 left-0 h-7 flex justify-between items-center bg-transparent select-none z-[60] w-full">

        <div data-tauri-drag-region class="flex-1 h-full flex items-center gap-2 pl-3 overflow-hidden">
            <img src="/128x128.png" alt="App Logo" class="h-4 w-4 object-contain opacity-90 pointer-events-none rounded-md shrink-0" />

            <span class="text-[11px] font-medium tracking-wide line-clamp-1 pointer-events-none shrink-0
                {themeManager.theme === 'light' ? 'text-black/80' : 'text-white/80'}">
                {layoutState.title || ''}
            </span>
        </div>

        {#if layoutState.mpv}
            <div class="flex items-center pr-2 pointer-events-auto shrink-0" data-tauri-drag-region="false">
                <button
                        onclick={() => popoverOpen = !popoverOpen}
                        class="group flex items-center gap-1.5 h-5 px-2.5 rounded-full text-[10px] font-medium
                        bg-primary/10 text-primary border border-primary/20
                        hover:bg-primary/20 hover:border-primary/30 transition-all duration-200
                        {popoverOpen ? 'bg-primary/20 border-primary/30 shadow-sm' : ''}"
                >
                    <span class="relative flex h-1.5 w-1.5 shrink-0">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-primary"></span>
                    </span>

                    <span class="line-clamp-1 max-w-[180px] pt-[0.5px]">
                        {layoutState.mpv.animeTitle} · Ep {layoutState.mpv.epNumber}
                    </span>

                    <ChevronDown class="size-3 shrink-0 opacity-60 group-hover:opacity-100 {popoverOpen ? 'rotate-180 opacity-100' : ''} transition-all duration-300 ease-out" />
                </button>
            </div>
        {/if}

        <div class="flex h-full shrink-0">
            <button onclick={minimize} class="h-full w-[40px] hover:bg-muted/40 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center" tabindex="-1">
                <Minus class="size-[13px] stroke-[1.5]" />
            </button>
            <button onclick={maximize} class="h-full w-[40px] hover:bg-muted/40 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center" tabindex="-1">
                <Square class="size-[11px] stroke-[1.5]" />
            </button>
            <button onclick={close} class="h-full w-[40px] hover:bg-[#e81123] text-muted-foreground/80 hover:text-white transition-none inline-flex items-center justify-center" tabindex="-1">
                <X class="size-[13px] stroke-[1.5]" />
            </button>
        </div>

        {#if popoverOpen}
            <MpvPopover bind:open={popoverOpen} />
        {/if}
    </div>
{/if}