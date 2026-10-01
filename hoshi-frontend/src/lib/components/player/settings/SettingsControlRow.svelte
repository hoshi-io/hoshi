<script lang="ts">
    import type { Component, Snippet } from "svelte";
    import SettingsIconTile from "@/components/player/settings/SettingsIconTile.svelte";

    // Shared shell for every non-navigating row (switch, slider, color).
    // Same height, padding, icon tile, label style and hover as SettingsRow.
    let {
        icon,
        label,
        nested = false,
        control,
        below
    }: {
        icon?: Component;
        label: string;
        /** Child setting: no icon, indented so its label lines up under the parent's label. */
        nested?: boolean;
        /** Right-aligned control (switch, swatch, ...). */
        control?: Snippet;
        /** Full-width content under the label (slider). */
        below?: Snippet;
    } = $props();
</script>

<label
        class="group flex flex-col justify-center w-full min-h-11 py-1.5 pr-3 rounded-sm hover:bg-accent transition-colors cursor-pointer {nested ? 'pl-[3.25rem]' : 'pl-3'}"
>
    <span class="flex items-center justify-between gap-3">
        <span class="flex items-center gap-3 min-w-0">
            {#if icon}
                <SettingsIconTile {icon} />
            {/if}
            <span class="text-sm font-medium truncate {nested ? 'text-muted-foreground' : 'text-foreground'}">{label}</span>
        </span>
        {#if control}
            <span class="flex items-center shrink-0">{@render control()}</span>
        {/if}
    </span>
    {#if below}
        <span class="block mt-1.5 pl-10">{@render below()}</span>
    {/if}
</label>