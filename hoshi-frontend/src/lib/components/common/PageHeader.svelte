<script lang="ts">
    import type { Snippet } from "svelte";

    let { title, description, actions }: {
        title: string;
        description?: string;
        actions?: Snippet;
    } = $props();
</script>

<header class="hidden md:flex items-start justify-between gap-6 w-full">
    <div class="min-w-0 space-y-1">
        <h1 class="text-3xl font-semibold tracking-tight truncate">{title}</h1>
        {#if description}
            <p class="text-sm text-muted-foreground">{description}</p>
        {/if}
    </div>
    {#if actions}
        <div class="flex shrink-0 items-center gap-1">{@render actions()}</div>
    {/if}
</header>