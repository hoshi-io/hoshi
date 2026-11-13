<script lang="ts">
    import { untrack } from 'svelte';
    import Check from 'phosphor-svelte/lib/CheckIcon';
    import Plus from 'phosphor-svelte/lib/PlusIcon';
    import SegmentedControl from '@/components/settings/parts/SegmentedControl.svelte';
    import { homeEditor, MAX_SECTIONS } from '@/app/home-editor.svelte.js';
    import { labelText } from '@/app/home.svelte.js';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { cn } from '@/utils';
    import type { ContentType } from '@/api/content/types';
    import type { HomeTab } from '@/api/home/types';
    import { buildPresets, sameSource, TRACKER_NAMES, type Preset, type PresetGroup } from './presets';

    let { tab, onadded }: { tab: HomeTab; onadded?: (id: string) => void } = $props();

    const GROUPS: PresetGroup[] = ['forYou', 'calendar', 'anilist', 'mangabaka', 'extensions'];

    /** The tab's most common content type, so the catalog starts on what the tab is about. */
    function dominantType(t: HomeTab): ContentType {
        const counts = new Map<ContentType, number>();
        for (const s of t.sections) if (s.contentType) counts.set(s.contentType, (counts.get(s.contentType) ?? 0) + 1);
        return [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ?? 'anime';
    }

    let contentType = $state<string>('anime');
    let filter = $state('');

    $effect(() => {
        tab.id;
        contentType = untrack(() => dominantType(tab));
    });

    const typeOptions = $derived(
        (['anime', 'manga', 'novel'] as const).map(t => ({ value: t, label: i18n.t(`home.${t}`) }))
    );

    const needle = $derived(filter.trim().toLowerCase());
    const presets = $derived(buildPresets(contentType as ContentType, extensions.installed));
    const visible = $derived(needle ? presets.filter(p => labelText(p.section.title).toLowerCase().includes(needle)) : presets);
    const full = $derived(tab.sections.length >= MAX_SECTIONS);

    function groupTitle(g: PresetGroup) {
        return TRACKER_NAMES[g] ?? i18n.t(`home.edit.group_${g}`);
    }

    function isAdded(p: Preset) {
        return !p.repeatable && tab.sections.some(s => sameSource(s, p.section));
    }

    function add(p: Preset) {
        const id = homeEditor.addSection(tab.id, p.section);
        if (id) onadded?.(id);
    }
</script>

<div class="flex h-full flex-col">
    <div class="space-y-3 pb-4">
        <input
                bind:value={filter}
                placeholder={i18n.t('home.edit.search_presets')}
                class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm outline-none transition-colors focus:border-ring"
        />
        <SegmentedControl bind:value={contentType} options={typeOptions} label={i18n.t('home.edit.content_type')} fluid />
        {#if full}
            <p class="text-xs text-destructive">{i18n.t('home.edit.tab_full')}</p>
        {/if}
    </div>

    <div class="-mr-2 flex-1 space-y-6 overflow-y-auto pr-2">
        {#each GROUPS as group (group)}
            {@const rows = visible.filter(p => p.group === group && !p.compact)}
            {@const chips = visible.filter(p => p.group === group && p.compact)}
            {#if rows.length > 0 || chips.length > 0}
                <section>
                    <h3 class="mb-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">{groupTitle(group)}</h3>

                    {#each rows as preset (preset.key)}
                        {@const added = isAdded(preset)}
                        <button
                                type="button"
                                disabled={added || full}
                                onclick={() => add(preset)}
                                class="flex w-full items-center justify-between gap-3 border-b border-border/40 py-2.5 text-left text-sm transition-colors last:border-b-0 hover:text-primary disabled:pointer-events-none"
                        >
                            <span class={cn('truncate', added && 'text-muted-foreground')}>{labelText(preset.section.title)}</span>
                            {#if added}
                                <Check class="size-4 shrink-0 text-muted-foreground" />
                            {:else}
                                <Plus class="size-4 shrink-0 text-muted-foreground" />
                            {/if}
                        </button>
                    {/each}

                    {#if chips.length > 0}
                        <p class="mb-2 mt-4 text-xs text-muted-foreground">{i18n.t('home.edit.genres')}</p>
                        <div class="flex flex-wrap gap-1.5">
                            {#each chips as preset (preset.key)}
                                {@const added = isAdded(preset)}
                                <button
                                        type="button"
                                        disabled={added || full}
                                        onclick={() => add(preset)}
                                        class={cn(
                                            'rounded-full border px-3 py-1 text-xs transition-colors disabled:pointer-events-none',
                                            added ? 'border-primary/40 text-primary' : 'border-border/60 text-muted-foreground hover:border-foreground/40 hover:text-foreground'
                                        )}
                                >
                                    {labelText(preset.section.title)}
                                </button>
                            {/each}
                        </div>
                    {/if}
                </section>
            {/if}
        {/each}
        {#if visible.length === 0}
            <p class="py-8 text-center text-sm text-muted-foreground">{i18n.t('home.edit.no_presets')}</p>
        {/if}
    </div>
</div>
