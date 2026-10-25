<script lang="ts" module>
    // Every card is a [data-sn] node that spatial nav measures on each keypress,
    // so keep rows short.
    export const TV_ROW_MAX = 20;
</script>

<script lang="ts">
    import type { Snippet } from 'svelte';

    let { title, children }: { title: string; children: Snippet } = $props();
</script>

<section>
    <h2 class="px-3 text-lg font-bold text-foreground">{title}</h2>

    <!-- px/py leave room for the focus ring and scale so overflow doesn't clip them -->
    <div
            class="flex gap-4 overflow-x-auto overflow-y-hidden scroll-smooth px-3 py-3
               [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    >
        {@render children()}
    </div>
</section>