<script lang="ts">
    import { ChevronLeft, ChevronRight } from 'lucide-svelte';

    let {
        value = $bindable(0),
        min,
        max,
        step = 1,
        disabled = false,
        label,
        format = (v: number) => String(v),
        oncommit
    }: {
        value: number;
        min: number;
        max: number;
        step?: number;
        disabled?: boolean;
        label?: string;
        format?: (v: number) => string;
        /** Fired once the user stops adjusting (debounced) or leaves the slider. */
        oncommit?: () => void;
    } = $props();

    const pct = $derived(max > min ? ((value - min) / (max - min)) * 100 : 0);

    let timer: ReturnType<typeof setTimeout> | null = null;

    function flush() {
        if (!timer) return;
        clearTimeout(timer);
        timer = null;
        oncommit?.();
    }

    function nudge(dir: 1 | -1) {
        const next = Math.min(max, Math.max(min, Number((value + dir * step).toFixed(4))));
        if (next === value) return;
        value = next;
        if (timer) clearTimeout(timer);
        timer = setTimeout(flush, 500);
    }

    function onKeydown(e: KeyboardEvent) {
        if (disabled) return;
        if (e.key === 'ArrowRight') {
            e.preventDefault();
            nudge(1);
        } else if (e.key === 'ArrowLeft') {
            e.preventDefault();
            nudge(-1);
        }
    }

    // Don't lose a pending save if the page goes away mid-adjustment.
    $effect(() => flush);
</script>

<!-- data-sn-lr: spatialNav leaves Left/Right to this element (Up/Down still move focus). -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
        role="slider"
        tabindex={disabled ? -1 : 0}
        data-sn={disabled ? undefined : ''}
        data-sn-lr
        aria-label={label}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        aria-disabled={disabled}
        onkeydown={onKeydown}
        onblur={flush}
        class="group flex w-72 scroll-my-24 items-center gap-3 rounded-lg px-3 py-2 outline-none
               focus:bg-secondary focus:ring-4 focus:ring-primary"
>
    <ChevronLeft class="size-5 shrink-0 text-muted-foreground opacity-30 group-focus:opacity-100" />
    <div class="h-2 flex-1 overflow-hidden rounded-full bg-muted">
        <div class="h-full rounded-full bg-primary" style="width: {pct}%"></div>
    </div>
    <ChevronRight class="size-5 shrink-0 text-muted-foreground opacity-30 group-focus:opacity-100" />
    <span class="w-16 shrink-0 text-right text-sm font-bold tabular-nums">{format(value)}</span>
</div>