<script lang="ts">
    import * as Carousel from '@/components/ui/carousel';
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import type { NormalizedCard } from "@/utils/normalize";
    import { Skeleton } from '@/components/ui/skeleton';
    import type { Snippet } from "svelte";

    let { title, items = [], ranked = false, loading = false, onvisible, actions }: {
        title: string;
        items: NormalizedCard[];
        ranked?: boolean;
        loading?: boolean;
        onvisible?: () => void;
        actions?: Snippet;
    } = $props();

    let el = $state<HTMLElement | null>(null);
    let visible = $state(false);

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

<section bind:this={el} class="space-y-1 md:space-y-0 group/section">
    <div class="flex items-center justify-between px-1">
        <h2 class="text-lg md:text-2xl font-bold tracking-tight text-foreground">{title}</h2>
        {@render actions?.()}
    </div>

    {#if visible && items.length > 0}
        <Carousel.Root
                opts={{ align: 'start', dragFree: true, skipSnaps: false, containScroll: 'trimSnaps' }}
                class="w-full relative group/carousel px-1 md:px-6 md:-mx-5"
        >
            <Carousel.Content class="-ml-3 md:-ml-5 py-2 md:py-6">
                {#each items as n, i (n.key)}
                    {#if ranked}
                        <Carousel.Item class="pl-3 md:pl-5 basis-[55%] sm:basis-[40%] md:basis-[30%] lg:basis-1/4 xl:basis-1/5 2xl:basis-[18%]">
                            <div class="flex items-end">
                                <span class="rank-number shrink-0 select-none font-black leading-none text-[5rem] md:text-[7rem] -mr-3 md:-mr-4" aria-hidden="true">{i + 1}</span>
                                <div class="min-w-0 flex-1"><CardWrapper {...n} /></div>
                            </div>
                        </Carousel.Item>
                    {:else}
                        <Carousel.Item class="pl-3 md:pl-5 basis-[42%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]">
                            <CardWrapper {...n} />
                        </Carousel.Item>
                    {/if}
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    {:else if !visible}
        <div class="h-40 md:h-56"></div>
    {:else if loading}
        <div class="overflow-hidden px-1 md:px-6 md:-mx-5">
            <div class="flex -ml-3 md:-ml-5 py-2 md:py-6">
                {#each Array(7) as _}
                    <div class="shrink-0 pl-3 md:pl-5 {ranked ? 'basis-[55%] sm:basis-[40%] md:basis-[30%] lg:basis-1/4 xl:basis-1/5 2xl:basis-[18%]' : 'basis-[42%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]'}">
                        <Skeleton class="w-full aspect-[2/3] rounded-sm" />
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</section>

<style>
    .rank-number {
        color: transparent;
        -webkit-text-stroke: 2px var(--muted-foreground);
        opacity: 0.6;
    }

    :global(.embla__container),
    :global(.embla__slide),
    :global([data-slot="carousel-content"]) {
        overflow: visible;
    }
</style>