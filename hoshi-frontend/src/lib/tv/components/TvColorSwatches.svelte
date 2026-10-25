<script lang="ts">
    import { Check } from 'lucide-svelte';

    const DEFAULT_PRESETS = [
        '#ffffff', '#ffff00', '#00ffff', '#00ff00', '#ffa500', '#ff0000', '#808080', '#000000'
    ];

    let {
        value,
        presets = DEFAULT_PRESETS,
        label,
        onchange
    }: {
        value: string;
        presets?: string[];
        label?: string;
        onchange: (color: string) => void;
    } = $props();

    const norm = (c: string) => (c ?? '').toLowerCase();

    // A saved colour that is not one of the presets stays visible (and selected) so nothing is lost.
    const swatches = $derived(
        value && !presets.some((p) => norm(p) === norm(value)) ? [...presets, value] : presets
    );

    function checkColor(hex: string) {
        const m = /^#?([0-9a-f]{6})/i.exec(hex ?? '');
        if (!m) return 'white';
        const n = parseInt(m[1], 16);
        const lum = 0.299 * ((n >> 16) & 255) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255);
        return lum > 150 ? 'black' : 'white';
    }
</script>

<div class="flex flex-wrap items-center gap-3 p-1" role="radiogroup" aria-label={label}>
    {#each swatches as color (color)}
        {@const selected = norm(color) === norm(value)}
        <button
                type="button"
                data-sn
                role="radio"
                aria-checked={selected}
                aria-label={color}
                onclick={() => onchange(color)}
                class="grid size-11 scroll-my-24 place-items-center rounded-full border-2 border-white/20 outline-none
                       transition-transform duration-150
                       focus:scale-110 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                       {selected ? 'ring-2 ring-foreground/60 ring-offset-2 ring-offset-background' : ''}"
                style="background-color: {color}"
        >
            {#if selected}
                <Check class="size-5" strokeWidth={3} style="color: {checkColor(color)}" />
            {/if}
        </button>
    {/each}
</div>