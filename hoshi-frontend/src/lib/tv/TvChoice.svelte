<script lang="ts" generics="T extends string">
    import { Check } from 'lucide-svelte';

    type Item = { value: T; label: string };

    let {
        items,
        value = $bindable(),
        label,
        onselect,
        size = 'md'
    }: {
        items: Item[];
        value: T;
        label?: string;
        onselect?: (v: T) => void;
        /** 'sm' for long lists inside side panels */
        size?: 'md' | 'sm';
    } = $props();

    function pick(v: T) {
        value = v;
        onselect?.(v);
    }
</script>

<!-- A row of big buttons instead of a dropdown: Left/Right moves between options, OK selects. -->
<div role="radiogroup" aria-label={label} class="flex flex-wrap {size === 'sm' ? 'gap-2' : 'gap-3'}">
    {#each items as item (item.value)}
        {@const selected = item.value === value}
        <button
                type="button"
                role="radio"
                aria-checked={selected}
                data-sn
                onclick={() => pick(item.value)}
                class="flex items-center gap-2 rounded-md font-semibold outline-none
                   transition-[transform,background-color,color] duration-150
                   focus:scale-105 focus:ring-4 focus:ring-primary/70
                   {size === 'sm' ? 'h-10 px-4 text-base' : 'h-12 px-6 text-lg'}
                   {selected ? 'bg-primary text-primary-foreground' : 'bg-muted/30 text-foreground/80'}"
        >
            {#if selected}<Check class="size-5 shrink-0" />{/if}
            {item.label}
        </button>
    {/each}
</div>