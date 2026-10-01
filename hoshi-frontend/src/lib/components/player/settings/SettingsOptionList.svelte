<script lang="ts">
    import { Check } from "lucide-svelte";

    let {
        options,
        activeId,
        onSelect
    }: {
        options: { id: string; label: string }[];
        activeId: string;
        onSelect: (id: string) => void;
    } = $props();
</script>

{#each options as option (option.id)}
    {@const isActive = option.id === activeId}
    <button
            onclick={() => onSelect(option.id)}
            class="flex items-center gap-3 w-full min-h-11 px-3 py-1.5 rounded-sm text-left outline-none transition-colors hover:bg-accent focus-visible:bg-accent {isActive ? 'text-primary bg-accent font-medium' : 'text-foreground/80'}"
    >
        <span class="flex items-center justify-center w-4 h-4 shrink-0">
            {#if isActive}
                <Check class="w-4 h-4 text-primary" />
            {/if}
        </span>
        <span class="flex-1 text-sm truncate">{option.label}</span>
    </button>
{/each}