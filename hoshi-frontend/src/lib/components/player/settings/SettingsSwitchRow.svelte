<script lang="ts">
    import type { Component } from "svelte";
    import { Switch } from "@/components/ui/switch";
    import SettingsControlRow from "@/components/player/settings/SettingsControlRow.svelte";

    let {
        icon,
        label,
        checked,
        onCheckedChange,
        nested = false
    }: {
        icon?: Component;
        label: string;
        checked: boolean;
        onCheckedChange: (value: boolean) => void;
        nested?: boolean;
    } = $props();
</script>

<SettingsControlRow {icon} {label} {nested}>
    {#snippet control()}
        <Switch {checked} {onCheckedChange} class="shrink-0" />
    {/snippet}
</SettingsControlRow>