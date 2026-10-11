<script lang="ts" generics="T">
    import { flip } from "svelte/animate";
    import { fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import type { Snippet } from "svelte";
    import { tick } from "svelte";
    import { Skeleton } from "$lib/components/ui/skeleton";

    let {
        items,
        keyFn,
        hasMore = false,
        isLoading = false,
        onLoadMore,
        cardContent,
        class: className = "space-y-6 mb-10",
        gridClass = "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8",
    }: {
        items: T[];
        keyFn: (item: T) => string | number;
        hasMore?: boolean;
        isLoading?: boolean;
        onLoadMore?: () => void;
        cardContent: Snippet<[T]>;
        class?: string;
        gridClass?: string;
    } = $props();

    let sentinel = $state<HTMLElement | null>(null);
    let isIntersecting = $state(false);

    $effect(() => {
        if (!sentinel || !hasMore) return;

        const observer = new IntersectionObserver((entries) => {
            isIntersecting = entries[0].isIntersecting;
        }, {
            rootMargin: "600px"
        });

        observer.observe(sentinel);
        return () => observer.disconnect();
    });

    let loadingMore = false;

    $effect(() => {
        if (isIntersecting && hasMore && !loadingMore) {
            loadingMore = true;
            onLoadMore?.();
            tick().then(() => {
                loadingMore = false;
            });
        }
    });

    // ---- Entrance ----------------------------------------------------------------------------

    const prefersReducedMotion =
        typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;

    const STAGGER_MS = 30;   // gap between consecutive cards
    const STAGGER_CAP = 14;  // ...but never more than ~420ms in total, so long lists don't crawl

    // Keys that were already on screen after the last render (plain variable on purpose: not reactive).
    let seen = new Set<string | number>();

    // Position of each *new* card within its batch: 0, 1, 2... Cards that were already there aren't listed.
    // So the first page staggers from 0, and a "load more" batch staggers from 0 as well
    // (instead of starting at card 31 and waiting a full second).
    const entrance = $derived.by(() => {
        const order = new Map<string | number, number>();
        let n = 0;
        for (const item of items) {
            const key = keyFn(item);
            if (!seen.has(key)) order.set(key, n++);
        }
        return order;
    });

    // Runs after the DOM is updated, so the next batch is measured against what is now on screen.
    $effect(() => {
        seen = new Set(items.map(keyFn));
    });

    function premiumPop(
        node: HTMLElement,
        { delay = 0, duration = 400 }: { delay?: number; duration?: number } = {}
    ) {
        return {
            delay: prefersReducedMotion ? 0 : delay,
            duration: prefersReducedMotion ? 0 : duration,
            css: (t: number) => {
                const eased = cubicOut(t);
                return `
                    opacity: ${eased};
                    transform: scale(${0.95 + eased * 0.05}) translateY(${(1 - eased) * 12}px);
                `;
            }
        };
    }
</script>

<div class={className}>
    <div class={gridClass}>
        {#each items as item (keyFn(item))}
            <div
                    animate:flip={{ duration: 350, easing: cubicOut }}
                    in:premiumPop|global={{ delay: Math.min(entrance.get(keyFn(item)) ?? 0, STAGGER_CAP) * STAGGER_MS, duration: 200 }}
                    out:fly={{ y: -10, duration: 150 }}
                    class="group will-change-transform"
            >
                {@render cardContent(item)}
            </div>
        {/each}

        {#if isLoading && items.length > 0}
            {#each Array(6) as _, i}
                <div
                        class="flex flex-col gap-3 w-full animate-in fade-in duration-300"
                        style="animation-delay: {i * 75}ms"
                >
                    <Skeleton class="aspect-[2/3] w-full rounded-sm bg-muted/20" />
                    <div class="space-y-2 px-1">
                        <Skeleton class="h-3 w-1/3 bg-muted/40" />
                        <Skeleton class="h-4 w-3/4 bg-muted/30" />
                    </div>
                </div>
            {/each}
        {/if}
    </div>

    {#if isLoading && items.length === 0}
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8">
            {#each Array(12) as _, i}
                <div
                        class="flex flex-col gap-3 w-full"
                >
                    <Skeleton class="aspect-[2/3] w-full rounded-sm bg-muted/20" />
                    <div class="space-y-2 px-1">
                        <Skeleton class="h-3 w-1/3 bg-muted/40" />
                        <Skeleton class="h-4 w-3/4 bg-muted/30" />
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    {#if hasMore}
        <div bind:this={sentinel} class="h-4 w-full pointer-events-none mt-2"></div>
    {/if}
</div>