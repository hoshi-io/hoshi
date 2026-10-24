<script lang="ts">
    import type { Snippet } from "svelte";
    import { Label } from "@/components/ui/label";

    let {
        label,
        description,
        for: forId,
        disabled = false,
        inline = false,
        stacked = false,
        extra,
        class: className = "",
        children
    }: {
        label: string,
        description?: string,
        for?: string,
        disabled?: boolean,
        /** Keep the control on the same line as the text at every width (use for switches). */
        inline?: boolean,
        /** Put the control under the text at every width (use for wide controls: theme cards, swatches). */
        stacked?: boolean,
        /** Rendered inline right after the description (e.g. a "learn more" link). */
        extra?: Snippet,
        class?: string,
        children?: Snippet
    } = $props();

    const layout = $derived(
        stacked
            ? "flex-col items-stretch gap-4"
            : inline
                ? "flex-row items-center gap-4 sm:gap-8"
                : "flex-col sm:flex-row sm:items-center gap-3 sm:gap-8"
    );

    const textWidth = $derived(stacked ? "" : `${inline ? "flex-1" : ""} sm:max-w-[55%] sm:flex-none`);

    const controlWidth = $derived(
        stacked ? "w-full" : inline ? "sm:justify-end" : "w-full sm:w-auto sm:justify-end"
    );
</script>

<div
        class="flex justify-between py-5 border-b border-border/40 last:border-b-0 transition-opacity
               {layout} {disabled ? 'opacity-50' : ''} {className}"
>
    <div class="space-y-0.5 min-w-0 {textWidth}">
        <Label
                for={forId}
                class="text-[15px] font-semibold leading-snug {forId && !disabled ? 'cursor-pointer' : ''} {disabled ? 'cursor-not-allowed' : ''}"
        >
            {label}
        </Label>
        {#if description}
            <p class="text-sm text-muted-foreground leading-snug">{description}{#if extra}{" "}{@render extra()}{/if}</p>
        {/if}
    </div>

    {#if children}
        <!-- `inert` blocks focus and clicks while disabled, so wrapped controls (selects, inputs) need no disabled prop. -->
        <div class="{stacked ? '' : 'shrink-0'} flex {controlWidth}" inert={disabled || undefined}>
            {@render children()}
        </div>
    {/if}
</div>