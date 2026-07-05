<script lang="ts">
    import * as ContextMenu from "$lib/components/ui/context-menu";
    import * as Drawer from "$lib/components/ui/drawer";
    import type { Snippet } from "svelte";
    import {layoutState} from "@/stores/layout.svelte";
    import { Info, Plus, Trash2 } from "lucide-svelte";

    interface Props {
        trigger: Snippet;
        onDetails: () => void;
        title: string;
        status: string;
        onIncrement?: () => void;
        onDelete?: () => void;
    }

    let { trigger, onDetails, title, status, onIncrement, onDelete }: Props = $props();

    function handleDelete() {
        if (confirm(`Remove "${title}" from your list?`)) {
            onDelete?.();
        }
    }

    let isMobile = $state(layoutState.isMobile);
    $effect(() => {
        const isMobileQuery = window.matchMedia("(max-width: 1024px)");
        isMobile = isMobileQuery.matches;

        const handler = (e: MediaQueryListEvent) => isMobile = e.matches;
        isMobileQuery.addEventListener("change", handler);
        return () => isMobileQuery.removeEventListener("change", handler);
    });

    let isDrawerOpen = $state(false);

    let pressTimer: ReturnType<typeof setTimeout> | null = null;

    function handleTouchStart(e: TouchEvent) {
        pressTimer = setTimeout(() => {
            isDrawerOpen = true;
            if (navigator.vibrate) navigator.vibrate(50);
        }, 600);
    }

    function handleTouchEnd() {
        if (pressTimer) clearTimeout(pressTimer);
    }
</script>

{#if isMobile}
    <div
            class="w-full h-full"
            ontouchstart={handleTouchStart}
            ontouchend={handleTouchEnd}
            ontouchcancel={handleTouchEnd}
    >
        {@render trigger()}
    </div>

    <Drawer.Root bind:open={isDrawerOpen}>
        <Drawer.Content class="p-6 pb-10 rounded-sm border-border/50 max-h-[50vh]">
            <div class="mt-4 space-y-2">
                <button
                        class="w-full text-left font-bold py-3.5 px-4 rounded-xl bg-muted/50 hover:bg-muted text-sm flex items-center gap-2 transition-colors"
                        onclick={() => {
                            isDrawerOpen = false;
                            onDetails();
                        }}
                >
                    <Info class="w-4 h-4" /> View Details
                </button>

                {#if onIncrement || onDelete}
                    <div class="h-px bg-border/50 my-2"></div>
                {/if}

                {#if onIncrement}
                    <button
                            class="w-full text-left font-bold py-3.5 px-4 rounded-xl bg-muted/50 hover:bg-muted text-sm flex items-center gap-2 transition-colors"
                            onclick={() => {
                                isDrawerOpen = false;
                                onIncrement();
                            }}
                    >
                        <Plus class="w-4 h-4" /> +1 Progress
                    </button>
                {/if}

                {#if onDelete}
                    <button
                            class="w-full text-left font-bold py-3.5 px-4 rounded-xl bg-destructive/10 hover:bg-destructive/20 text-destructive text-sm flex items-center gap-2 transition-colors"
                            onclick={() => {
                                isDrawerOpen = false;
                                handleDelete();
                            }}
                    >
                        <Trash2 class="w-4 h-4" /> Delete
                    </button>
                {/if}
            </div>
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <ContextMenu.Root>
        <ContextMenu.Trigger class="w-full h-full">
            {@render trigger()}
        </ContextMenu.Trigger>
        <ContextMenu.Content class="w-56 rounded-sm border border-border/60 bg-popover/95 backdrop-blur-md p-1 shadow-md">
            <ContextMenu.Item
                    class="font-medium rounded-sm text-sm px-3 py-2 cursor-pointer transition-colors flex items-center gap-2 focus:bg-primary focus:text-primary-foreground"
                    onSelect={onDetails} >
                <Info class="w-3.5 h-3.5" /> View Details
            </ContextMenu.Item>

            {#if onIncrement || onDelete}
                <ContextMenu.Separator class="h-px bg-border/50 my-1" />
            {/if}

            {#if onIncrement}
                <ContextMenu.Item
                        class="font-medium rounded-sm text-sm px-3 py-2 cursor-pointer transition-colors flex items-center gap-2 focus:bg-primary focus:text-primary-foreground"
                        onSelect={onIncrement} >
                    <Plus class="w-3.5 h-3.5" /> +1 Progress
                </ContextMenu.Item>
            {/if}

            {#if onDelete}
                <ContextMenu.Item
                        class="font-medium rounded-sm text-sm px-3 py-2 cursor-pointer transition-colors flex items-center gap-2 text-destructive focus:bg-destructive focus:text-destructive-foreground"
                        onSelect={handleDelete} >
                    <Trash2 class="w-3.5 h-3.5" /> Delete
                </ContextMenu.Item>
            {/if}
        </ContextMenu.Content>
    </ContextMenu.Root>
{/if}