<script lang="ts">
    import type { Snippet } from "svelte";
    import type { PackageOpen } from "lucide-svelte";

    let {
        icon,
        title,
        description = "",
        tone = "muted",
        children
    }: {
        icon: typeof PackageOpen;
        title: string;
        description?: string;
        tone?: "muted" | "primary";
        children?: Snippet;
    } = $props();

    const Icon = $derived(icon);
</script>

<div class="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-border/40 bg-muted/5 px-4 py-14 text-center">
    <div class="mb-4 rounded-full border p-4 {tone === 'primary' ? 'border-primary/20 bg-primary/10 text-primary' : 'border-border/50 bg-muted text-muted-foreground'}">
        <Icon class="size-8" />
    </div>
    <h3 class="mb-1 text-lg font-bold text-foreground">{title}</h3>
    {#if description}
        <p class="max-w-sm text-sm text-muted-foreground">{description}</p>
    {/if}
    {#if children}
        <div class="mt-5">{@render children()}</div>
    {/if}
</div>