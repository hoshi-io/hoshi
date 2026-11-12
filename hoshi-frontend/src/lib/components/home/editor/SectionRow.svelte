<script lang="ts">
    import { ArrowDown, ArrowUp, Eye, EyeOff, GripVertical, MoreHorizontal, Pencil, Trash2 } from 'lucide-svelte';
    import * as DropdownMenu from '@/components/ui/dropdown-menu';
    import SegmentedControl from '@/components/settings/parts/SegmentedControl.svelte';
    import { homeEditor } from '@/app/home-editor.svelte.js';
    import { labelText } from '@/app/home.svelte.js';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { cn } from '@/utils';
    import type { HomeSection, HomeTab, SectionLayout } from '@/api/home/types';
    import { TRACKER_NAMES } from './presets';

    let { tab, section, index }: { tab: HomeTab; section: HomeSection; index: number } = $props();

    const FIXED_LAYOUT = ['continueWatching', 'recentlyAired', 'extensions'];

    const layoutOptions = $derived([
        { value: 'carousel', label: i18n.t('home.edit.layout_carousel') },
        { value: 'ranked', label: i18n.t('home.edit.layout_ranked') },
        { value: 'strip', label: i18n.t('home.edit.layout_strip') },
    ]);

    const fixedLayout = $derived(section.source.kind === 'builtin' && FIXED_LAYOUT.includes(section.source.builtin.type));

    const meta = $derived.by(() => {
        const src = section.source;
        let source: string;
        if (src.kind === 'builtin') source = i18n.t('home.edit.source_builtin');
        else if (src.query.kind === 'tracker') source = TRACKER_NAMES[src.query.tracker] ?? src.query.tracker;
        else {
            const extId = src.query.extension;
            source = extensions.installed.find(e => e.id === extId)?.name ?? extId;
        }
        const parts = [source];
        if (section.contentType) parts.push(i18n.t(`home.${section.contentType}`));
        if (tab.hero === section.id) parts.push(i18n.t('home.edit.hero'));
        if (section.hidden) parts.push(i18n.t('home.edit.hidden'));
        return parts.join(' · ');
    });

    let renaming = $state(false);
    let name = $state('');
    let input = $state<HTMLInputElement | null>(null);

    function startRename() {
        name = labelText(section.title);
        renaming = true;
        queueMicrotask(() => { input?.focus(); input?.select(); });
    }

    function commitRename() {
        if (!renaming) return;
        if (name.trim() && name.trim() !== labelText(section.title)) homeEditor.renameSection(tab.id, section.id, name);
        renaming = false;
    }
</script>

<div class="section-row flex items-center gap-3 border-b border-border/40 bg-background py-4 last:border-b-0">
    <button
            type="button"
            data-drag-handle
            aria-label={i18n.t('home.edit.drag')}
            class="-ml-1 flex h-9 w-6 shrink-0 cursor-grab touch-none items-center justify-center text-muted-foreground/50 transition-colors hover:text-foreground active:cursor-grabbing"
    >
        <GripVertical class="size-4" />
    </button>

    <div class={cn('min-w-0 flex-1 transition-opacity', section.hidden && 'opacity-50')}>
        {#if renaming}
            <input
                    bind:this={input}
                    bind:value={name}
                    maxlength={64}
                    onblur={commitRename}
                    onkeydown={(e) => { if (e.key === 'Enter') commitRename(); if (e.key === 'Escape') renaming = false; }}
                    class="h-8 w-full rounded-md border border-input bg-background px-2 text-[15px] font-semibold outline-none focus:border-ring"
            />
        {:else}
            <p class="truncate text-base font-semibold md:text-[15px]">{labelText(section.title)}</p>
        {/if}
        <p class="mt-0.5 truncate text-sm text-muted-foreground">{meta}</p>
    </div>

    {#if !fixedLayout}
        <div class="hidden md:block">
            <SegmentedControl
                    value={section.layout}
                    options={layoutOptions}
                    label={i18n.t('home.edit.layout')}
                    onValueChange={(v) => homeEditor.setLayout(tab.id, section.id, v as SectionLayout)}
            />
        </div>
    {/if}

    <button
            type="button"
            title={i18n.t(section.hidden ? 'home.edit.show' : 'home.edit.hide')}
            aria-label={i18n.t(section.hidden ? 'home.edit.show' : 'home.edit.hide')}
            aria-pressed={section.hidden}
            onclick={() => homeEditor.toggleHidden(tab.id, section.id)}
            class="flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
    >
        {#if section.hidden}<EyeOff class="size-4" />{:else}<Eye class="size-4" />{/if}
    </button>

    <button
            type="button"
            title={i18n.t('home.edit.remove')}
            aria-label={i18n.t('home.edit.remove')}
            onclick={() => homeEditor.removeSection(tab.id, section.id)}
            class="flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
    >
        <Trash2 class="size-4" />
    </button>

    <DropdownMenu.DropdownMenu>
        <DropdownMenu.DropdownMenuTrigger>
            {#snippet child({ props })}
                <button {...props} type="button" aria-label={i18n.t('home.edit.more')}
                        class="flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground">
                    <MoreHorizontal class="size-4" />
                </button>
            {/snippet}
        </DropdownMenu.DropdownMenuTrigger>
        <DropdownMenu.DropdownMenuContent align="end" class="w-48">
            <DropdownMenu.DropdownMenuItem onclick={startRename}>
                <Pencil />{i18n.t('home.edit.rename')}
            </DropdownMenu.DropdownMenuItem>
            {#if !fixedLayout}
                <div class="md:hidden">
                    <DropdownMenu.DropdownMenuSeparator />
                    <DropdownMenu.DropdownMenuLabel class="text-xs font-normal text-muted-foreground">{i18n.t('home.edit.layout')}</DropdownMenu.DropdownMenuLabel>
                    <DropdownMenu.DropdownMenuRadioGroup
                            value={section.layout}
                            onValueChange={(v) => homeEditor.setLayout(tab.id, section.id, v as SectionLayout)}
                    >
                        {#each layoutOptions as l (l.value)}
                            <DropdownMenu.DropdownMenuRadioItem value={l.value}>{l.label}</DropdownMenu.DropdownMenuRadioItem>
                        {/each}
                    </DropdownMenu.DropdownMenuRadioGroup>
                </div>
            {/if}
            <DropdownMenu.DropdownMenuSeparator />
            <DropdownMenu.DropdownMenuItem disabled={index === 0} onclick={() => homeEditor.moveSection(tab.id, index, index - 1)}>
                <ArrowUp />{i18n.t('home.edit.move_up')}
            </DropdownMenu.DropdownMenuItem>
            <DropdownMenu.DropdownMenuItem disabled={index === tab.sections.length - 1} onclick={() => homeEditor.moveSection(tab.id, index, index + 1)}>
                <ArrowDown />{i18n.t('home.edit.move_down')}
            </DropdownMenu.DropdownMenuItem>
        </DropdownMenu.DropdownMenuContent>
    </DropdownMenu.DropdownMenu>
</div>

<style>
    :global([data-dragging]) .section-row {
        position: relative;
        z-index: 10;
        border-radius: 0.5rem;
        box-shadow: 0 8px 24px -10px rgb(0 0 0 / 0.45);
    }
</style>
