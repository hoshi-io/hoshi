<script lang="ts">
    import type { Component } from "svelte";
    import SettingsControlRow from "@/components/player/settings/SettingsControlRow.svelte";

    let {
        icon,
        label,
        value,
        min,
        max,
        step,
        onInput
    }: {
        icon?: Component;
        label: string;
        value: number;
        min: number;
        max: number;
        step: number;
        onInput: (value: number) => void;
    } = $props();
</script>

<SettingsControlRow {icon} {label}>
    {#snippet below()}
        <input
                type="range"
                {min}
                {max}
                {step}
                {value}
                oninput={(e) => onInput(Number(e.currentTarget.value))}
                class="block w-full h-1 cursor-pointer accent-primary"
        />
    {/snippet}
</SettingsControlRow>