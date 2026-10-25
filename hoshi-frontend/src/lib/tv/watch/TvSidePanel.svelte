<script lang="ts">
    import { onMount, type Snippet } from 'svelte';
    import { fly } from 'svelte/transition';

    let {
        title,
        wide = false,
        children
    }: {
        title: string;
        wide?: boolean;
        children: Snippet;
    } = $props();

    let panel = $state<HTMLElement | undefined>();

    // Focus the current/selected entry (or the first control) and hand focus back on close.
    onMount(() => {
        const returnTo = document.activeElement as HTMLElement | null;
        requestAnimationFrame(() => {
            const target =
                panel?.querySelector<HTMLElement>('[aria-checked="true"], [data-current="true"]') ??
                panel?.querySelector<HTMLElement>('[data-sn]');
            target?.focus();
            target?.scrollIntoView({ block: 'center' });
        });
        return () => {
            if (returnTo && document.contains(returnTo)) returnTo.focus();
        };
    });
</script>

<!-- data-sn-trap: spatialNav only moves between focusables inside this panel. -->
<aside
        bind:this={panel}
        data-sn-trap
        role="dialog"
        aria-modal="true"
        aria-label={title}
        transition:fly={{ x: 60, duration: 200 }}
        class="absolute inset-y-0 right-0 z-[70] flex flex-col border-l border-white/10 bg-black/90 text-white
               {wide ? 'w-[36rem]' : 'w-[26rem]'}"
>
    <h2 class="shrink-0 px-8 pb-4 pt-8 text-2xl font-black tracking-tight">{title}</h2>
    <div class="min-h-0 flex-1 overflow-y-auto px-5 pb-8">
        {@render children()}
    </div>
</aside>