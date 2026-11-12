<script lang="ts">
    import { flip } from 'svelte/animate';
    import { crossfade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { GripVertical, Plus } from 'lucide-svelte';
    import { homeEditor, MAX_TABS } from '@/app/home-editor.svelte.js';
    import { labelText } from '@/app/home.svelte.js';
    import { tabIcon } from '@/components/home/icons';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { sortable } from '@/utils/sortable';
    import { cn } from '@/utils';

    let { activeTabId = $bindable() }: { activeTabId: string | null } = $props();

    const [send, receive] = crossfade({ duration: 250, easing: cubicOut });

    const tabs = $derived(homeEditor.draft?.tabs ?? []);
    const currentId = $derived(activeTabId ?? tabs[0]?.id ?? null);

    function addTab() {
        const id = homeEditor.addTab(i18n.t('home.edit.new_tab'), 'compass');
        if (id) activeTabId = id;
    }

    const itemClass =
        'group relative flex w-full cursor-pointer select-none items-center justify-between rounded-md md:rounded-sm px-3 py-3 md:py-2.5 ' +
        'text-base md:text-[15px] font-semibold text-foreground/75 outline-none transition-colors duration-200 hover:text-foreground ' +
        'focus-visible:ring-2 focus-visible:ring-primary/40';
    const chipClass =
        'grid size-9 shrink-0 place-items-center rounded-lg bg-muted/60 text-muted-foreground transition-colors duration-200 ' +
        'group-hover:text-foreground md:size-8 md:bg-transparent';
</script>

<div class="w-full">
    <div class="px-3 pb-2 pt-1 text-[11px] font-bold uppercase tracking-[0.14em] text-muted-foreground/70">
        {i18n.t('home.edit.tabs')}
    </div>

    <div class="flex w-full flex-col rounded-xl border border-border/40 bg-muted/20 p-1 md:gap-0.5 md:border-0 md:bg-transparent md:p-0">
        <div class="flex flex-col md:gap-0.5" use:sortable={{ onmove: (from, to) => homeEditor.moveTab(from, to) }}>
            {#each tabs as tab, i (tab.id)}
                {@const Icon = tabIcon(tab.icon)}
                {@const active = currentId === tab.id}
                <div class="nav-item tab-item" style="--i: {i}" animate:flip={{ duration: 200, easing: cubicOut }}>
                    <div
                            role="button"
                            tabindex="0"
                            aria-pressed={active}
                            onclick={() => activeTabId = tab.id}
                            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activeTabId = tab.id; } }}
                            class={cn(itemClass, active && 'md:text-foreground', !active && 'md:hover:bg-muted/40 active:bg-muted/40')}
                    >
                        {#if active}
                            <span
                                    aria-hidden="true"
                                    class="absolute inset-0 hidden rounded-sm bg-primary/10 ring-1 ring-inset ring-primary/15 md:block"
                                    in:receive={{ key: 'home-edit-tab' }}
                                    out:send={{ key: 'home-edit-tab' }}
                            ></span>
                        {/if}
                        <span class="relative flex min-w-0 items-center gap-3.5">
                            <span class={cn(chipClass, active && 'text-primary md:text-foreground')}><Icon class="size-[1.1rem] md:size-4" /></span>
                            <span class="truncate">{labelText(tab.title)}</span>
                        </span>
                        <span class="relative flex items-center gap-2">
                            <span class="text-sm font-medium tabular-nums text-muted-foreground">{tab.sections.length}</span>
                            <span
                                    data-drag-handle
                                    aria-label={i18n.t('home.edit.drag')}
                                    class="flex h-8 w-6 cursor-grab touch-none items-center justify-center text-muted-foreground/40 transition-colors hover:text-foreground active:cursor-grabbing md:opacity-0 md:group-hover:opacity-100"
                            >
                                <GripVertical class="size-4" />
                            </span>
                        </span>
                    </div>
                </div>
            {/each}
        </div>

        <button
                type="button"
                onclick={addTab}
                disabled={tabs.length >= MAX_TABS}
                class={cn(itemClass, 'text-muted-foreground md:hover:bg-muted/40 disabled:pointer-events-none disabled:opacity-40')}
        >
            <span class="flex items-center gap-3.5">
                <span class={chipClass}><Plus class="size-[1.1rem] md:size-4" /></span>
                {i18n.t('home.edit.add_tab')}
            </span>
        </button>
    </div>
</div>

<style>
    .nav-item {
        animation: navIn 380ms cubic-bezier(0.22, 1, 0.36, 1) backwards;
        animation-delay: calc(var(--i) * 28ms);
    }

    @keyframes navIn {
        from { opacity: 0; transform: translateX(-8px); }
    }

    .tab-item:global([data-dragging]) {
        opacity: 0.6;
    }

    @media (prefers-reduced-motion: reduce) {
        .nav-item { animation: none; }
    }
</style>
