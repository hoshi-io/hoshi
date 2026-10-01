<script lang="ts">
    import { ChevronRight } from "lucide-svelte";
    import type { Component } from "svelte";
    import SettingsIconTile from "@/components/player/settings/SettingsIconTile.svelte";

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
</script>

<button
        {onclick}
        class="group flex items-center justify-between gap-3 w-full min-h-11 px-3 py-1.5 rounded-sm hover:bg-accent focus-visible:bg-accent outline-none transition-colors text-left"
>
    <span class="flex items-center gap-3 min-w-0">
        <SettingsIconTile {icon} />
        <span class="text-sm font-medium text-foreground truncate">{label}</span>
    </span>
    <span class="flex items-center gap-2 shrink-0">
        {#if value}
            <span class="text-xs text-muted-foreground truncate max-w-[120px]">{value}</span>
        {/if}
        <ChevronRight class="w-4 h-4 text-muted-foreground/70 group-hover:text-foreground/70 transition-colors" />
    </span>
</button>