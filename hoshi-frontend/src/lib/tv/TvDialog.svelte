<script lang="ts">
    import { onMount, type Snippet } from 'svelte';

    let {
        title,
        description,
        onclose,
        children
    }: {
        title: string;
        description?: string;
        onclose: () => void;
        children: Snippet;
    } = $props();

    let panel = $state<HTMLElement | undefined>();

    // Render on <body> so no ancestor (scroll container, transform) can clip or offset it.
    function portal(node: HTMLElement) {
        document.body.appendChild(node);
        return { destroy: () => node.remove() };
    }

    // Native listener (not a Svelte one) so stopPropagation also keeps the layout's global
    // Back handler from running while the dialog is open.
    function backCloses(node: HTMLElement) {
        const handler = (e: KeyboardEvent) => {
            const isBack =
                e.key === 'Escape' || e.key === 'GoBack' || e.key === 'BrowserBack' || e.keyCode === 4;
            if (!isBack) return;
            e.preventDefault();
            e.stopPropagation();
            onclose();
        };
        node.addEventListener('keydown', handler);
        return { destroy: () => node.removeEventListener('keydown', handler) };
    }

    onMount(() => {
        const returnTo = document.activeElement as HTMLElement | null;
        requestAnimationFrame(() => panel?.querySelector<HTMLElement>('[data-sn]')?.focus());
        return () => {
            if (returnTo && document.contains(returnTo)) returnTo.focus();
        };
    });
</script>

<div use:portal use:backCloses class="fixed inset-0 z-[100] flex items-center justify-center bg-black/75 p-8">
    <!-- data-sn-trap: spatialNav only moves between focusables inside this panel. -->
    <div
            bind:this={panel}
            data-sn-trap
            role="dialog"
            aria-modal="true"
            aria-label={title}
            class="max-h-full w-[34rem] max-w-full space-y-5 overflow-y-auto rounded-xl border border-border bg-card p-8 text-foreground shadow-2xl"
    >
        <header class="space-y-1.5">
            <h2 class="text-2xl font-black capitalize tracking-tight">{title}</h2>
            {#if description}
                <p class="text-base text-muted-foreground">{description}</p>
            {/if}
        </header>

        {@render children()}
    </div>
</div>