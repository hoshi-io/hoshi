<script lang="ts">
    import type { Component } from "svelte";
    import SettingsControlRow from "@/components/player/settings/SettingsControlRow.svelte";

    let {
        icon,
        label,
        value,
        onInput,
        nested = false
    }: {
        icon?: Component;
        label: string;
        value: string;
        onInput: (value: string) => void;
        nested?: boolean;
    } = $props();
</script>

<SettingsControlRow {icon} {label} {nested}>
    {#snippet control()}
        <input
                type="color"
                {value}
                oninput={(e) => onInput(e.currentTarget.value)}
                class="w-7 h-7 p-0 rounded-sm border border-border bg-transparent cursor-pointer overflow-hidden
                       [&::-webkit-color-swatch-wrapper]:p-0 [&::-webkit-color-swatch]:border-0
                       [&::-moz-color-swatch]:border-0"
        />
    {/snippet}
</SettingsControlRow>