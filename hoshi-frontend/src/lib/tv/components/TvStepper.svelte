<script lang="ts">
    import { Minus, Plus } from 'lucide-svelte';

    let {
        value = $bindable(0),
        min = 0,
        max = Infinity,
        step = 1,
        label
    }: {
        value: number;
        min?: number;
        max?: number;
        step?: number;
        label?: string;
    } = $props();

    function nudge(dir: 1 | -1) {
        value = Math.min(max, Math.max(min, (Number(value) || 0) + dir * step));
    }

    function onKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowRight') {
            e.preventDefault();
            nudge(1);
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            nudge(-1);
        }
    }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
        role="spinbutton"
        tabindex="0"
        data-sn
        data-sn-lr
        aria-label={label}
        aria-valuemin={min}
        aria-valuemax={max === Infinity ? undefined : max}
        aria-valuenow={value}
        onkeydown={onKeydown}
        class="group flex w-48 scroll-my-24 items-center justify-between gap-3 rounded-lg px-3 py-2 outline-none
               focus:bg-secondary focus:ring-4 focus:ring-primary"
>
    <Minus class="size-5 shrink-0 text-muted-foreground opacity-30 group-focus:opacity-100" />
    <span class="text-xl font-bold tabular-nums">{value}</span>
    <Plus class="size-5 shrink-0 text-muted-foreground opacity-30 group-focus:opacity-100" />
</div>