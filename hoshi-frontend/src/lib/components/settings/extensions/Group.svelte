<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        title = "",
        count,
        skeletons = 0,
        children
    }: {
        title?: string;
        count?: number;
        skeletons?: number;
        children?: Snippet;
    } = $props();
</script>

<section class="space-y-3">
    {#if title}
        <div class="flex items-center gap-3">
            <h3 class="text-sm font-bold uppercase tracking-wider text-muted-foreground">{title}</h3>
            {#if count !== undefined}
                <span class="rounded-full border border-border/50 bg-muted px-2 py-0.5 text-[11px] font-semibold tabular-nums text-muted-foreground">
                    {count}
                </span>
            {/if}
            <div class="h-px flex-1 bg-border/50"></div>
        </div>
    {/if}

    <div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
        {#if skeletons > 0}
            {#each Array(skeletons) as _, i (i)}
                <div class="flex items-center gap-3 rounded-xl border border-border/50 bg-card p-3.5">
                    <div class="size-11 shrink-0 animate-pulse rounded-lg bg-muted"></div>
                    <div class="flex-1 space-y-2">
                        <div class="h-3.5 w-1/2 animate-pulse rounded bg-muted"></div>
                        <div class="h-3 w-1/3 animate-pulse rounded bg-muted/70"></div>
                    </div>
                    <div class="h-8 w-20 animate-pulse rounded-lg bg-muted"></div>
                </div>
            {/each}
        {:else}
            {@render children?.()}
        {/if}
    </div>
</section>