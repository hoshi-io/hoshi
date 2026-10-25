<script lang="ts">
    let {
        value = $bindable(),
        options,
        label,
        disabled = false,
        onchange
    }: {
        value: string;
        options: { value: string; label: string }[];
        label?: string;
        disabled?: boolean;
        onchange?: (value: string) => void;
    } = $props();

    function pick(v: string) {
        if (v === value) return;
        value = v;
        onchange?.(v);
    }
</script>

<div class="flex flex-wrap gap-2" role="radiogroup" aria-label={label}>
    {#each options as option (option.value)}
        <button
                type="button"
                data-sn
                role="radio"
                aria-checked={value === option.value}
                {disabled}
                onclick={() => pick(option.value)}
                class="h-11 scroll-my-24 whitespace-nowrap rounded-lg border-2 px-5 text-sm font-bold outline-none
                       transition-transform duration-150
                       focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                       {value === option.value
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border/60 bg-card text-foreground'}"
        >
            {option.label}
        </button>
    {/each}
</div>