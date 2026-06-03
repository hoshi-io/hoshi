<script lang="ts" generics="T">
    import { flip } from "svelte/animate";
    import { fade } from "svelte/transition";
    import { cubicInOut } from "svelte/easing";
    import type { Snippet } from "svelte";
    import { tick } from "svelte";

    let {
        items,
        keyFn,
        hasMore = false,
        isLoading = false,
        onLoadMore,
        cardContent,
    }: {
        items: T[];
        keyFn: (item: T) => string | number;
        hasMore?: boolean;
        isLoading?: boolean;
        onLoadMore?: () => void;
        cardContent: Snippet<[T]>;
    } = $props();

    let sentinel = $state<HTMLElement | null>(null);
    let isIntersecting = $state(false);

    $effect(() => {
        if (!sentinel || !hasMore) return;

        const observer = new IntersectionObserver((entries) => {
            isIntersecting = entries[0].isIntersecting;
        }, {
            rootMargin: "500px"
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
</script>

<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8 mb-10">
    {#each items as item (keyFn(item))}
        <div
                animate:flip={{ duration: 400, easing: cubicInOut }}
                in:fade={{ duration: 250 }}
                out:fade={{ duration: 150 }}
                class="group will-change-transform"
        >
            {@render cardContent(item)}
        </div>
    {/each}
</div>

{#if hasMore}
    <div bind:this={sentinel} class="h-10 w-full flex items-center justify-center mt-4">
        {#if isLoading}
            <div class="flex gap-1">
                <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:0ms]"></span>
                <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:150ms]"></span>
                <span class="w-1.5 h-1.5 rounded-full bg-muted-foreground/40 animate-bounce [animation-delay:300ms]"></span>
            </div>
        {/if}
    </div>
{/if}