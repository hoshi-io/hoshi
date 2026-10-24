<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        title,
        description,
        variant = "default",
        class: className = "",
        children
    }: {
        title: string,
        description?: string,
        /** "danger" tints the title and drops the divider; "bare" only drops the divider (e.g. when tabs follow). */
        variant?: "default" | "danger" | "bare",
        class?: string,
        children: Snippet
    } = $props();
</script>

<section class="w-full {className}">
    <header class="mb-2">
        <h2 class="font-bold tracking-tight {variant === 'danger' ? 'text-xl text-destructive' : 'text-2xl'}">{title}</h2>
        {#if description}
            <p class="text-sm text-muted-foreground mt-1">{description}</p>
        {/if}
    </header>

    <div class={variant === "danger" ? "pt-3" : variant === "bare" ? "pt-2" : "border-t border-border/40"}>
        {@render children()}
    </div>
</section>