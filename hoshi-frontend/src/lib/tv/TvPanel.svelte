<script lang="ts">
    import { onMount, tick, type Snippet } from 'svelte';
    import { fade, fly } from 'svelte/transition';

    let {
        title,
        onclose,
        children,
        footer
    }: {
        title: string;
        onclose: () => void;
        children: Snippet;
        footer?: Snippet;
    } = $props();

    let panel: HTMLElement | null = $state(null);

    onMount(() => {
        // Remember who opened us so focus can go back there on close.
        const opener = document.activeElement as HTMLElement | null;

        tick().then(() => {
            const el =
                panel?.querySelector<HTMLElement>('[data-sn][data-autofocus]') ??
                panel?.querySelector<HTMLElement>('[data-sn]');
            el?.focus();
        });

        return () => {
            if (opener && document.contains(opener)) opener.focus();
        };
    });

    // Capture phase + stopPropagation: Back only closes the panel and never reaches the layout's handler.
    function onKeydown(e: KeyboardEvent) {
        const isBack =
            e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;
        if (!isBack) return;
        e.preventDefault();
        e.stopPropagation();
        onclose();
    }
</script>

<svelte:window onkeydowncapture={onKeydown} />

<div class="fixed inset-0 z-[60]" role="dialog" aria-modal="true" aria-label={title}>
    <div class="absolute inset-0 bg-black/60" transition:fade|global={{ duration: 150 }}></div>

    <!-- data-sn-trap limits D-pad navigation (spatialNav) to this panel -->
    <aside
            bind:this={panel}
            data-sn-trap
            class="absolute right-0 top-0 flex h-full w-[26rem] flex-col border-l border-border/20 bg-background"
            transition:fly|global={{ x: 420, duration: 180 }}
    >
        <header class="shrink-0 px-6 pb-3 pt-8">
            <h2 class="text-2xl font-black tracking-tight">{title}</h2>
        </header>

        <div class="min-h-0 flex-1 overflow-y-auto px-6 py-3 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
            {@render children()}
        </div>

        {#if footer}
            <div class="shrink-0 border-t border-border/20 px-6 py-4">
                {@render footer()}
            </div>
        {/if}
    </aside>
</div>