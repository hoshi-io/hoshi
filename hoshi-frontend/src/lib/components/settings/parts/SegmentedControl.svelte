<script lang="ts">
    import { onMount, tick } from "svelte";
    import type { ComponentType } from "svelte";

    type Option = { value: string, label: string, icon?: ComponentType, class?: string };

    let {
        value = $bindable(),
        options,
        onValueChange,
        label,
        fluid = false,
        class: className = ""
    }: {
        value: string,
        options: Option[],
        onValueChange?: (value: string) => void,
        label?: string,
        /** fill the parent's width with equal-sized, taller segments */
        fluid?: boolean,
        class?: string
    } = $props();

    let container: HTMLDivElement | undefined = $state();
    let buttons: HTMLButtonElement[] = $state([]);
    let pill = $state({ x: 0, y: 0, w: 0, h: 0 });
    // The pill only animates after its first measurement, so it never "flies in" from 0,0.
    let ready = $state(false);

    function measure() {
        const i = options.findIndex((o) => o.value === value);
        const el = buttons[i];
        if (!el) return;
        pill = { x: el.offsetLeft, y: el.offsetTop, w: el.offsetWidth, h: el.offsetHeight };
    }

    $effect(() => {
        value; options;
        tick().then(measure);
    });

    onMount(() => {
        measure();
        requestAnimationFrame(() => (ready = true));

        // Re-measure when the layout changes (window resize, 2-col mobile wrap, language change).
        const ro = new ResizeObserver(measure);
        if (container) ro.observe(container);
        return () => ro.disconnect();
    });

    function select(v: string) {
        if (v === value) return;
        value = v;
        onValueChange?.(v);
    }

    function onKeydown(e: KeyboardEvent, i: number) {
        let next = -1;
        if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (i + 1) % options.length;
        else if (e.key === "ArrowLeft" || e.key === "ArrowUp") next = (i - 1 + options.length) % options.length;
        if (next < 0) return;
        e.preventDefault();
        select(options[next].value);
        buttons[next]?.focus();
    }
</script>

<div
        bind:this={container}
        role="radiogroup"
        aria-label={label}
        style={fluid || options.length <= 3 ? `grid-template-columns: repeat(${options.length}, minmax(0, 1fr))` : undefined}
        class="relative grid gap-1 rounded-lg bg-muted/50 p-1 w-full {fluid ? '' : 'grid-cols-2 sm:inline-flex sm:w-auto'} {className}"
>
    <span
            aria-hidden="true"
            class="absolute left-0 top-0 rounded-md bg-background shadow-sm ring-1 ring-border/60 pointer-events-none
                   {ready ? 'transition-[transform,width,height] duration-300 ease-[cubic-bezier(0.34,1.25,0.64,1)] motion-reduce:transition-none' : ''}"
            style="width: {pill.w}px; height: {pill.h}px; transform: translate({pill.x}px, {pill.y}px); opacity: {pill.w ? 1 : 0};"
    ></span>

    {#each options as opt, i (opt.value)}
        {@const active = opt.value === value}
        {@const Icon = opt.icon}
        <button
                bind:this={buttons[i]}
                type="button"
                role="radio"
                aria-checked={active}
                tabindex={active ? 0 : -1}
                onclick={() => select(opt.value)}
                onkeydown={(e) => onKeydown(e, i)}
                class="relative z-10 inline-flex items-center justify-center gap-2 {fluid ? 'h-10' : 'h-9'} px-3.5 rounded-md text-sm font-semibold whitespace-nowrap transition-colors duration-200
                       focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40
                       {active ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'} {opt.class ?? ''}"
        >
            {#if Icon}<Icon class="size-4 shrink-0" />{/if}
            {opt.label}
        </button>
    {/each}
</div>