<script lang="ts">
    import { browser } from '$app/environment';
    import { type } from '@tauri-apps/plugin-os';
    import { Minus, Square, X } from 'lucide-svelte';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { themeManager } from '@/stores/theme.svelte.js';
    import MpvPopover from '@/components/mpv/MpvPopover.svelte';
    import HistoryMenu from "@/components/history/HistoryMenu.svelte";

    const isTauri = browser && '__TAURI__' in window;
    const osType = isTauri ? type() : null;
    const showTitlebar = isTauri && osType !== 'android' && osType !== 'ios';
    let winPromise: Promise<any> | null = null;

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
    <div class="absolute top-0 left-0 h-8 grid grid-cols-3 items-center bg-transparent select-none z-[60] w-full">

        <div data-tauri-drag-region class="h-full flex items-center gap-2.5 pl-5 overflow-hidden">
        </div>

        <div data-tauri-drag-region class="h-full w-full relative flex justify-center items-center">
            {#if layoutState.mpv}
                <MpvPopover />
            {/if}
        </div>

        <div class="flex h-full shrink-0 justify-end">
            <button onclick={minimize} class="h-full w-[42px] hover:bg-muted/20 text-muted-foreground/60 hover:text-foreground transition-colors inline-flex items-center justify-center" tabindex="-1">
                <Minus class="size-[12px] stroke-[2]" />
            </button>
            <button onclick={maximize} class="h-full w-[42px] hover:bg-muted/20 text-muted-foreground/60 hover:text-foreground transition-colors inline-flex items-center justify-center" tabindex="-1">
                <Square class="size-[10px] stroke-[2]" />
            </button>
            <button onclick={close} class="h-full w-[46px] hover:bg-destructive/90 text-muted-foreground/60 hover:text-destructive-foreground transition-colors inline-flex items-center justify-center" tabindex="-1">
                <X class="size-[12px] stroke-[2]" />
            </button>
        </div>
    </div>
{/if}