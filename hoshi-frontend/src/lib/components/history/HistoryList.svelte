<script lang="ts">
    import { historyStore } from '@/stores/history.svelte';
    import { i18n } from '@/stores/i18n.svelte.js';
    import XIcon from 'phosphor-svelte/lib/XIcon';

    let { onNavigate, compact = false }: { onNavigate?: () => void; compact?: boolean } = $props();
</script>

{#if historyStore.entries.length === 0}
    <p class="px-4 py-6 text-center text-sm text-muted-foreground">{i18n.t('history.empty')}</p>
{:else}
    <div class="flex min-h-0 flex-col">
        <ul class="min-h-0 flex-1 overflow-y-auto p-1.5">
            {#each historyStore.entries as entry (entry.id)}
                <li class="group relative">
                    <a
                            href="/c/{entry.id}"
                            draggable="false"
                            onclick={() => onNavigate?.()}
                            class="flex items-center gap-3 rounded-sm p-1.5 pr-9 outline-none transition-colors
                                   hover:bg-muted/40 focus-visible:bg-muted/40"
                    >
                        {#if entry.coverImage}
                            <img src={entry.coverImage} alt="" loading="lazy" class="aspect-2/3 w-8 shrink-0 rounded-sm bg-muted object-cover" />
                        {:else}
                            <div class="aspect-2/3 w-8 shrink-0 rounded-sm bg-muted/40"></div>
                        {/if}
                        <span class="line-clamp-2 text-sm font-medium leading-snug">{entry.title || '—'}</span>
                    </a>
                    <button
                            type="button"
                            onclick={() => historyStore.remove(entry.id)}
                            aria-label={i18n.t('history.remove')}
                            title={i18n.t('history.remove')}
                            class="absolute top-1/2 right-1.5 -translate-y-1/2 rounded-sm p-1.5 text-muted-foreground outline-none transition
                                   hover:bg-muted/60 hover:text-foreground focus-visible:bg-muted/60 focus-visible:opacity-100
                                   {compact ? 'opacity-0 group-hover:opacity-100' : ''}"
                    >
                        <XIcon class="size-3.5" />
                    </button>
                </li>
            {/each}
        </ul>
        <button
                type="button"
                onclick={() => historyStore.clear()}
                class="shrink-0 border-t border-border/40 px-3 py-2.5 text-xs font-semibold text-muted-foreground outline-none transition-colors
                       hover:bg-destructive/5 hover:text-destructive focus-visible:bg-muted/40"
        >
            {i18n.t('history.clear')}
        </button>
    </div>
{/if}
