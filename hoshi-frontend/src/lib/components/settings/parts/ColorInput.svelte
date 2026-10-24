<script lang="ts">
    let {
        value = $bindable(),
        onchange,
        label
    }: {
        value: string,
        onchange?: () => void,
        label?: string
    } = $props();
</script>

<label class="group inline-flex items-center gap-3 cursor-pointer">
    <span
            class="relative size-9 rounded-full ring-1 ring-border shadow-sm overflow-hidden transition-transform duration-200 group-hover:scale-105 group-active:scale-95 group-focus-within:ring-2 group-focus-within:ring-primary/50"
            style="background-color: {value}"
    >
        <input
                type="color"
                bind:value
                {onchange}
                aria-label={label}
                class="absolute inset-0 size-full opacity-0 cursor-pointer"
        />
    </span>
    <span class="text-xs font-mono font-semibold uppercase tracking-wide text-muted-foreground tabular-nums">{value}</span>
</label>