<script lang="ts">
    import { historyStore } from '@/stores/history.svelte';
    import { goto } from '$app/navigation';
    import { X, Clock } from 'lucide-svelte';

    let { onNavigate }: { onNavigate?: () => void } = $props();

    function open(id: string | number) {
        goto(`/c/${id}`);
        onNavigate?.();
    }
</script>

<div class="flex flex-col h-full max-h-[80vh]">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border/60">
        <div class="flex items-center gap-2">
            <Clock class="size-4 text-muted-foreground/80" />
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Recently viewed</span>
        </div>
        {#if historyStore.entries.length}
            <button
                    onclick={() => historyStore.clear()}
                    class="text-xs font-medium text-destructive/80 hover:text-destructive transition-colors px-2 py-1 rounded hover:bg-destructive/10"
            >
                Clear all
            </button>
        {/if}
    </div>

    <!-- Content Area -->
    <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        {#if historyStore.entries.length === 0}
            <div class="flex flex-col items-center justify-center py-8 px-4 text-center">
                <p class="text-sm text-muted-foreground/60">No recent history found.</p>
            </div>
        {:else}
            <div class="flex flex-col gap-1">
                {#each historyStore.entries as entry (entry.id)}
                    <div
                            role="button"
                            tabindex="0"
                            onclick={() => open(entry.id)}
                            onkeydown={(e) => e.key === 'Enter' && open(entry.id)}
                            class="group flex items-center gap-3 rounded-lg p-2 text-left hover:bg-muted/60 focus-visible:bg-muted/60 outline-none transition-all duration-150 cursor-pointer border border-transparent hover:border-border/40"
                    >
                        <!-- Cover Image / Fallback -->
                        {#if entry.coverImage}
                            <img
                                    src={entry.coverImage}
                                    alt=""
                                    class="size-9 shrink-0 rounded-md object-cover bg-muted ring-1 ring-border/50"
                            />
                        {:else}
                            <div class="size-9 shrink-0 rounded-md bg-muted flex items-center justify-center border border-border/50 text-muted-foreground/40">
                                <Clock class="size-4" />
                            </div>
                        {/if}

                        <!-- Title Text -->
                        <div class="flex flex-col overflow-hidden flex-1">
                            <span class="truncate text-sm font-medium text-foreground/90 group-hover:text-foreground">
                                {entry.title || 'Untitled Session'}
                            </span>
                        </div>

                        <!-- Action Button (X) -->
                        <button
                                onclick={(e) => {
                                e.stopPropagation();
                                historyStore.remove(entry.id);
                            }}
                                class="shrink-0 rounded-md p-1.5 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 hover:bg-background text-muted-foreground/60 hover:text-destructive transition-all"
                                aria-label="Remove from history"
                        >
                            <X class="size-3.5" />
                        </button>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

<style>
    /* Clean custom minimalist scrollbar for desktop */
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: var(--muted, #e4e4e7);
        border-radius: 2px;
    }
</style>