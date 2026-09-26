<script lang="ts">
    import { ChevronRight } from "lucide-svelte";
    import type { Component } from "svelte";

    let {
        icon,
        label,
        value,
        onclick
    }: {
        icon: Component;
        label: string;
        value?: string;
        onclick: () => void;
    } = $props();

    const Icon = $derived(icon);
</script>

<button
        {onclick}
        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-accent transition text-left"
>
    <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-muted group-hover:bg-accent transition-colors">
            <Icon class="w-4 h-4 text-foreground/80 group-hover:text-foreground" />
        </div>
        <span class="text-sm font-medium text-foreground">{label}</span>
    </div>
    <div class="flex items-center gap-2">
        {#if value}
            <span class="text-xs text-muted-foreground truncate max-w-[90px]">{value}</span>
        {/if}
        <ChevronRight class="w-4 h-4 text-muted-foreground/70 group-hover:text-foreground/70 transition-colors" />
    </div>
</button>