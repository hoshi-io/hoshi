<script lang="ts">
    import * as Carousel from '@/components/ui/carousel';
    import { goto } from '$app/navigation';
    import { Puzzle } from 'lucide-svelte';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { searchState } from '@/app/search.svelte.js';
    import type { ContentType } from '@/api/content/types';
    import type { Extension } from '@/api/extensions/types';

    let { title, contentType }: { title: string; contentType: ContentType | null } = $props();

    const items = $derived(
        extensions.installed.filter(e =>
            e.ext_type !== 'torrent' && (!contentType || e.ext_type === contentType)
        )
    );

    function open(ext: Extension) {
        searchState.openExtension(ext.id, ext.ext_type as ContentType);
        goto('/search');
    }
</script>

{#if items.length > 0}
    <section class="space-y-2 md:space-y-3">
        <h2 class="text-lg md:text-2xl font-bold tracking-tight text-foreground px-1">{title}</h2>
        <Carousel.Root opts={{ align: 'start', dragFree: true, containScroll: 'trimSnaps' }} class="w-full px-1">
            <Carousel.Content class="-ml-3">
                {#each items as ext (ext.id)}
                    <Carousel.Item class="pl-3 basis-auto">
                        <button
                                type="button"
                                onclick={() => open(ext)}
                                class="flex w-28 flex-col items-center gap-2 rounded-lg border border-border/40 bg-card/50 p-3 transition-colors hover:bg-muted/60"
                        >
                            <div class="flex size-12 items-center justify-center overflow-hidden rounded-xl bg-muted">
                                {#if ext.icon}
                                    <img src={ext.icon} alt="" class="size-full object-cover" loading="lazy" />
                                {:else}
                                    <Puzzle class="size-6 text-muted-foreground" />
                                {/if}
                            </div>
                            <span class="w-full truncate text-center text-xs font-semibold">{ext.name}</span>
                            <span class="text-[10px] uppercase tracking-wider text-muted-foreground">{ext.ext_type}</span>
                        </button>
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    </section>
{/if}
