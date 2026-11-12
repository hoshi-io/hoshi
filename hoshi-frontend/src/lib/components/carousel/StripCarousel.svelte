<script lang="ts">
    import * as Carousel from '@/components/ui/carousel';
    import SmartImage from '@/components/SmartImage.svelte';
    import { Skeleton } from '@/components/ui/skeleton';
    import type { Snippet } from 'svelte';
    import { getCardContentTypeLabel, getCardScore, getCardShouldBlur, getCardTitle, type NormalizedCard } from '@/utils/normalize';

    const ROWS = 3;

    let { title, items = [], loading = false, onvisible, actions }: {
        title: string;
        items: NormalizedCard[];
        loading?: boolean;
        onvisible?: () => void;
        actions?: Snippet;
    } = $props();

    let el = $state<HTMLElement | null>(null);
    let visible = $state(false);

    const columns = $derived.by(() => {
        const cols: NormalizedCard[][] = [];
        for (let i = 0; i < items.length; i += ROWS) cols.push(items.slice(i, i + ROWS));
        return cols;
    });

    $effect(() => {
        if (!el) return;
        const observer = new IntersectionObserver(
            ([entry]) => { if (entry.isIntersecting) { visible = true; onvisible?.(); observer.disconnect(); } },
            { rootMargin: '800px' }
        );
        observer.observe(el);
        return () => observer.disconnect();
    });
</script>

<section bind:this={el} class="space-y-2 md:space-y-3">
    <div class="flex items-center justify-between px-1">
        <h2 class="text-lg md:text-2xl font-bold tracking-tight text-foreground">{title}</h2>
        {@render actions?.()}
    </div>

    {#if visible && items.length > 0}
        <Carousel.Root opts={{ align: 'start', dragFree: true, containScroll: 'trimSnaps' }} class="w-full px-1">
            <Carousel.Content class="-ml-3 md:-ml-4">
                {#each columns as col, ci (ci)}
                    <Carousel.Item class="pl-3 md:pl-4 basis-[85%] sm:basis-1/2 lg:basis-1/3 2xl:basis-1/4">
                        <div class="flex flex-col gap-2">
                            {#each col as card (card.key)}
                                {@const score = getCardScore(card)}
                                {@const label = getCardContentTypeLabel(card)}
                                <a href={card.href} class="flex items-center gap-3 rounded-md p-1.5 hover:bg-muted/50 transition-colors">
                                    <div class="w-12 shrink-0 aspect-[2/3] overflow-hidden rounded-sm bg-muted">
                                        <SmartImage src={card.cover} alt={getCardTitle(card)} class="w-full h-full object-cover" shouldBlur={getCardShouldBlur(card)} imageHeaders={card.imageHeaders} />
                                    </div>
                                    <div class="min-w-0">
                                        <p class="truncate text-sm font-semibold text-foreground">{getCardTitle(card)}</p>
                                        <p class="truncate text-xs text-muted-foreground">
                                            {[label, card.year, score].filter(Boolean).join(' · ')}
                                        </p>
                                    </div>
                                </a>
                            {/each}
                        </div>
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    {:else if !visible}
        <div class="h-48"></div>
    {:else if loading}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2 px-1">
            {#each Array(ROWS * 3) as _}
                <Skeleton class="h-[4.5rem] rounded-md" />
            {/each}
        </div>
    {/if}
</section>
