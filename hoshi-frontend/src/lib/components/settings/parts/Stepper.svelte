<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import Minus from 'phosphor-svelte/lib/MinusIcon';
    import Plus from 'phosphor-svelte/lib/PlusIcon';

    let {
        label,
        value,
        min,
        max,
        step,
        format = (v: number) => String(v),
        onChange,
    }: {
        label: string;
        value: number;
        min: number;
        max: number;
        step: number;
        format?: (v: number) => string;
        onChange: (v: number) => void;
    } = $props();

    function nudge(dir: 1 | -1) {
        const next = Math.round((value + dir * step) / step) * step;
        onChange(Number(Math.min(max, Math.max(min, next)).toFixed(2)));
    }
</script>

<div class="flex items-center justify-between gap-3">
    <span class="text-sm font-medium">{label}</span>
    <div class="flex shrink-0 items-center rounded-sm border border-border/60 bg-muted/30">
        <Button variant="ghost" size="icon" class="size-8 rounded-sm" aria-label="{label} −" disabled={value <= min} onclick={() => nudge(-1)}>
            <Minus class="size-3.5" />
        </Button>
        <span class="w-12 text-center font-mono text-sm font-semibold tabular-nums">{format(value)}</span>
        <Button variant="ghost" size="icon" class="size-8 rounded-sm" aria-label="{label} +" disabled={value >= max} onclick={() => nudge(1)}>
            <Plus class="size-3.5" />
        </Button>
    </div>
</div>
