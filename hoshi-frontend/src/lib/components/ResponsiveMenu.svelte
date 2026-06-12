<script lang="ts">
    import * as ContextMenu from "$lib/components/ui/context-menu";
    import * as Drawer from "$lib/components/ui/drawer";
    import type { Snippet } from "svelte";
    import {layoutState} from "@/stores/layout.svelte";

    interface Props {
        trigger: Snippet;
        onEdit: () => void;
        title: string;
        status: string;
    }

    let { trigger, onEdit, title, status }: Props = $props();

    // Check if device is mobile. You can replace this with your own responsive layout logic
    let isMobile = $state(layoutState.isMobile);
    $effect(() => {
        const isMobileQuery = window.matchMedia("(max-width: 1024px)");
        isMobile = isMobileQuery.matches;

        const handler = (e: MediaQueryListEvent) => isMobile = e.matches;
        isMobileQuery.addEventListener("change", handler);
        return () => isMobileQuery.removeEventListener("change", handler);
    });

    let isDrawerOpen = $state(false);

    // Mobile long-press handler
    let pressTimer: ReturnType<typeof setTimeout> | null = null;

    function handleTouchStart(e: TouchEvent) {
        pressTimer = setTimeout(() => {
            isDrawerOpen = true;
            // Vibrate if supported for haptic feedback
            if (navigator.vibrate) navigator.vibrate(50);
        }, 600); // 600ms hold triggers the menu
    }

    function handleTouchEnd() {
        if (pressTimer) clearTimeout(pressTimer);
    }
</script>

{#if isMobile}
    <div
            role="button"
            tabindex="0"
            ontouchstart={handleTouchStart}
            ontouchend={handleTouchEnd}
            ontouchmove={handleTouchEnd}
            class="w-full h-full select-none"
    >
        {@render trigger()}
    </div>

    <Drawer.Root bind:open={isDrawerOpen}>
        <Drawer.Content class="p-6 pb-10 rounded-t-3xl border-border/50 max-h-[50vh]">
            <Drawer.Header class="text-left px-0 pt-0">
                <Drawer.Title class="text-lg font-black tracking-tight line-clamp-1">{title}</Drawer.Title>
                <Drawer.Description class="text-xs font-semibold uppercase tracking-wider text-primary">{status}</Drawer.Description>
            </Drawer.Header>

            <div class="mt-4 space-y-2">
                <button
                        class="w-full text-left font-bold py-3.5 px-4 rounded-xl bg-muted/50 hover:bg-muted text-sm flex items-center transition-colors"
                        onclick={() => { isDrawerOpen = false; onEdit(); }}
                >
                    Edit Entry
                </button>
            </div>
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <ContextMenu.Root>
        <ContextMenu.Trigger class="w-full h-full">
            {@render trigger()}
        </ContextMenu.Trigger>
        <ContextMenu.Content class="w-56 rounded-xl border border-border/60 bg-popover/95 backdrop-blur-md p-1 shadow-md">
            <ContextMenu.Item
                    class="font-medium rounded-lg text-sm px-3 py-2 cursor-pointer transition-colors focus:bg-primary focus:text-primary-foreground"
                    onSelect={onEdit}
            >
                Edit Entry
            </ContextMenu.Item>
        </ContextMenu.Content>
    </ContextMenu.Root>
{/if}