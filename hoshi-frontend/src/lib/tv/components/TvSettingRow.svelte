<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        label,
        description,
        disabled = false,
        stacked = false,
        children
    }: {
        label: string;
        description?: string;
        disabled?: boolean;
        /** Put the control under the text instead of beside it (wide controls). */
        stacked?: boolean;
        children: Snippet;
    } = $props();
</script>

<div
        class="flex gap-6 border-b border-border/40 py-5
               {stacked ? 'flex-col' : 'items-center justify-between'}
               {disabled ? 'opacity-50' : ''}"
>
    <div class="min-w-0 space-y-1 {stacked ? '' : 'max-w-[60%]'}">
        <p class="text-lg font-bold leading-snug">{label}</p>
        {#if description}
            <p class="text-sm leading-snug text-muted-foreground">{description}</p>
        {/if}
    </div>
    <div class="shrink-0">
        {@render children()}
    </div>
</div>