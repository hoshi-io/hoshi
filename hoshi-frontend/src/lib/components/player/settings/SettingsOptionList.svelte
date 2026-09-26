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

{#each options as option}
    {@const isActive = option.id === activeId}
    <button
            onclick={() => onSelect(option.id)}
            class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-accent font-medium' : 'text-foreground/80 hover:bg-accent'}"
    >
        <div class="flex items-center justify-center w-5 h-5 shrink-0">
            {#if isActive}
                <Check class="w-4 h-4 text-primary" />
            {/if}
        </div>
        <span class="flex-1 text-sm truncate">{option.label}</span>
    </button>
{/each}