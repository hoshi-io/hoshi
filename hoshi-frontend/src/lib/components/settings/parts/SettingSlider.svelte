<script lang="ts">
    import { tick } from "svelte";
    import { Slider } from "@/components/ui/slider";

    let {
        value = $bindable(),
        min = 0,
        max,
        step = 1,
        format = (v: number) => String(v),
        oninput,
        oncommit,
        showValue = true,
        fluid = false,
        class: className = ""
    }: {
        value: number,
        min?: number,
        max: number,
        step?: number,
        format?: (v: number) => string,
        /** fires continuously while dragging (after `value` has been updated) */
        oninput?: () => void,
        /** fires once when the user releases the thumb / finishes a key press */
        oncommit?: (v: number) => void,
        /** show the formatted value next to the slider */
        showValue?: boolean,
        /** stretch to the full width of the parent instead of the fixed field width */
        fluid?: boolean,
        class?: string
    } = $props();

    function handleChange() {
        if (oninput) tick().then(() => oninput?.());
    }
</script>

<div class="flex items-center gap-4 w-full {fluid ? '' : 'sm:w-64'} {className}">
    <Slider
            type="single"
            bind:value
            {min}
            {max}
            {step}
            onValueChange={handleChange}
            onValueCommit={(v) => oncommit?.(v)}
            class="flex-1 min-w-0"
    />
    {#if showValue}
        <span class="w-12 shrink-0 text-right text-sm font-medium tabular-nums text-muted-foreground">{format(value)}</span>
    {/if}
</div>