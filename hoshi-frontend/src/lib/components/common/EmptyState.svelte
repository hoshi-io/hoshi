<script lang="ts">
    import type { Snippet } from "svelte";
    import type PackageIcon from 'phosphor-svelte/lib/PackageIcon';

    let {
        icon,
        title,
        description = "",
        tone = "muted",
        size = "page",
        class: className = "",
        children
    }: {
        icon: typeof PackageIcon;
        title: string;
        description?: string;
        tone?: "muted" | "primary" | "error";
        /** page: list, schedule, search. inline: inside a section (chapters, settings). tv: 10-foot sizes. */
        size?: "page" | "inline" | "tv";
        class?: string;
        children?: Snippet;
    } = $props();

    const Icon = $derived(icon);
    const error = $derived(tone === "error");
</script>

<div role="status" class="flex flex-col items-center justify-center px-4 text-center {size === 'inline' ? 'py-8' : 'py-16'} {className}">
    <!-- A small stack of blank cards, the app's empty-state illustration. -->
    <div class="relative shrink-0 {size === 'inline' ? 'h-32 w-40' : size === 'tv' ? 'h-56 w-72' : 'h-48 w-60'}" aria-hidden="true">
        {#each [{ i: 0, side: 'left-3', r: '-9deg' }, { i: 1, side: 'right-3', r: '9deg' }] as back (back.i)}
            <div
                    class="ghost absolute bottom-0 {back.side} aspect-2/3 rounded-sm
                           {size === 'inline' ? 'w-[4.75rem]' : size === 'tv' ? 'w-32' : 'w-28'}
                           {error ? 'bg-destructive/5' : 'bg-muted/20'}"
                    style="--i: {back.i}; --r: {back.r}"
            ></div>
        {/each}
        <div
                class="ghost absolute bottom-2 left-1/2 z-10 flex aspect-2/3 flex-col justify-between rounded-sm
                       {size === 'inline' ? 'w-[4.75rem] p-1.5' : size === 'tv' ? 'w-32 p-3' : 'w-28 p-2.5'}
                       {error ? 'bg-destructive/10' : 'bg-muted/40'}"
                style="--i: 2; --x: -50%"
        >
            <div class="flex flex-1 items-center justify-center">
                <Icon class="{size === 'inline' ? 'size-6' : size === 'tv' ? 'size-10' : 'size-8'}
                             {error ? 'text-destructive/80' : tone === 'primary' ? 'text-primary/80' : 'text-muted-foreground'}" />
            </div>
            <div class="space-y-1.5">
                <div class="h-1.5 w-3/4 rounded-sm {error ? 'bg-destructive/20' : 'bg-foreground/10'}"></div>
                <div class="h-1.5 w-1/2 rounded-sm {error ? 'bg-destructive/10' : 'bg-foreground/5'}"></div>
            </div>
        </div>
    </div>

    <h2 class="font-bold tracking-tight {size === 'inline' ? 'mt-5 text-base' : size === 'tv' ? 'mt-8 text-2xl' : 'mt-8 text-xl'} {error ? 'text-destructive' : ''}">
        {title}
    </h2>
    {#if description}
        <p class="mt-2 max-w-sm font-medium leading-relaxed text-muted-foreground {size === 'inline' ? 'text-xs' : size === 'tv' ? 'max-w-md text-base' : 'text-sm'}">
            {description}
        </p>
    {/if}
    {#if children}
        <div class="flex flex-col items-center gap-2 {size === 'inline' ? 'mt-4' : 'mt-6'}">{@render children()}</div>
    {/if}
</div>

<style>
    .ghost {
        transform: translateX(var(--x, 0)) rotate(var(--r, 0deg));
        transform-origin: 50% 100%;
        animation: ghostIn 520ms cubic-bezier(0.22, 1, 0.36, 1) backwards;
        animation-delay: calc(var(--i) * 80ms);
    }

    @keyframes ghostIn {
        from {
            opacity: 0;
            transform: translateX(var(--x, 0)) translateY(16px) rotate(0deg) scale(0.94);
        }
    }
</style>
