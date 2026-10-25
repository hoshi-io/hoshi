<script lang="ts">
    import { Check } from 'lucide-svelte';

    let {
        label,
        pressed = false,
        onclick
    }: {
        label: string;
        pressed?: boolean;
        onclick: () => void;
    } = $props();
</script>

<!-- On/off pill for booleans and multi-select options. -->
<button
        type="button"
        role="switch"
        aria-checked={pressed}
        data-sn
        {onclick}
        class="flex h-10 items-center gap-2 rounded-md px-4 text-base font-semibold outline-none
           transition-[transform,background-color,color] duration-150
           focus:scale-105 focus:ring-4 focus:ring-primary/70
           {pressed ? 'bg-primary text-primary-foreground' : 'bg-muted/30 text-foreground/80'}"
>
    {#if pressed}<Check class="size-5 shrink-0" />{/if}
    {label}
</button>