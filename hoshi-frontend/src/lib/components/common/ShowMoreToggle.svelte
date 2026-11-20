<script lang="ts">
    import CaretDown from 'phosphor-svelte/lib/CaretDownIcon';
    import { i18n } from "@/stores/i18n.svelte.js";

    let { expanded = $bindable(false), total }: { expanded?: boolean; total: number } = $props();
</script>

<button
        type="button"
        class="flex shrink-0 items-center gap-1.5 rounded-sm px-2 py-1 text-sm font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
        aria-expanded={expanded}
        onclick={() => (expanded = !expanded)}
>
    {#if expanded}
        {i18n.t('content.show_less')}
    {:else}
        {i18n.t('content.show_all')}
        <span class="tabular-nums text-muted-foreground/60">· {total}</span>
    {/if}
    <CaretDown class="size-3.5 transition-transform duration-200 {expanded ? 'rotate-180' : ''}" />
</button>
