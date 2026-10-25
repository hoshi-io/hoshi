<script lang="ts">
    import { Check } from 'lucide-svelte';

    let {
        options,
        activeId,
        onselect
    }: {
        options: { id: string; label: string }[];
        activeId: string | null;
        onselect: (id: string) => void;
    } = $props();
</script>

<div role="radiogroup" class="flex flex-col gap-2">
    {#each options as option (option.id)}
        {@const selected = option.id === activeId}
        <button
                type="button"
                role="radio"
                aria-checked={selected}
                data-sn
                onclick={() => onselect(option.id)}
                class="flex min-h-14 items-center justify-between gap-3 rounded-lg px-5 py-3 text-left text-lg font-semibold outline-none
                       focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60
                       {selected ? 'bg-primary/25 text-white' : 'bg-white/5 text-white/90'}"
        >
            <span class="min-w-0 flex-1 truncate">{option.label}</span>
            {#if selected}<Check class="size-5 shrink-0" />{/if}
        </button>
    {/each}
</div>