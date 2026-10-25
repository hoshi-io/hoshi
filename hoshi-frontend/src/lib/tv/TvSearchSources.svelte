<script lang="ts" module>
    export type Tracker = 'anilist' | 'mal' | 'kitsu' | 'simkl';

    export const TRACKERS: { id: Tracker; label: string; domain: string }[] = [
        { id: 'anilist', label: 'AniList', domain: 'anilist.co' },
        { id: 'mal', label: 'MyAnimeList', domain: 'myanimelist.net' },
        { id: 'kitsu', label: 'Kitsu', domain: 'kitsu.io' },
        { id: 'simkl', label: 'SIMKL', domain: 'simkl.com' }
    ];
</script>

<script lang="ts">
    import { Plug, Check, Database, Blocks } from 'lucide-svelte';
    import { searchState } from '@/app/search.svelte.js';

    let {
        availableExtensions = [],
        onSelectSource
    }: {
        availableExtensions: any[];
        onSelectSource: (mode: 'tracker' | 'extension', extId: string, tracker: Tracker) => void;
    } = $props();

    const sortedExtensions = $derived(
        [...availableExtensions].sort((a, b) => (a.name || '').localeCompare(b.name || ''))
    );

    // Same-named extensions get their source appended so they can be told apart.
    const displayNames = $derived.by(() => {
        const counts = new Map<string, number>();
        for (const ext of sortedExtensions) counts.set(ext.name, (counts.get(ext.name) ?? 0) + 1);

        const map = new Map<string, string>();
        for (const ext of sortedExtensions) {
            const dup = (counts.get(ext.name) ?? 0) > 1;
            map.set(ext.id, dup && ext.source ? `${ext.name} (${ext.source})` : ext.name);
        }
        return map;
    });

    const row =
        'flex h-12 w-full items-center gap-3 rounded-md px-4 text-left text-base font-medium outline-none ' +
        'transition-colors duration-150 focus:bg-primary focus:text-primary-foreground focus:ring-2 focus:ring-primary/60';
    const rowActive = 'bg-primary/10 text-primary';
    const rowIdle = 'text-foreground/80';

    const isTrackerActive = (id: Tracker) => searchState.searchMode === 'tracker' && searchState.tracker === id;
    const isExtActive = (id: string) => searchState.searchMode === 'extension' && searchState.selectedExtension === id;
</script>

<div class="space-y-6">
    <div class="space-y-1">
        <h3 class="mb-1 flex items-center gap-2 px-1 text-sm font-bold uppercase tracking-wider text-muted-foreground">
            <Database class="size-4" /> Trackers
        </h3>

        {#each TRACKERS as t (t.id)}
            <button
                    type="button"
                    data-sn
                    data-autofocus={isTrackerActive(t.id) ? '' : undefined}
                    onclick={() => onSelectSource('tracker', '', t.id)}
                    class="{row} {isTrackerActive(t.id) ? rowActive : rowIdle}"
            >
                <img
                        src="https://www.google.com/s2/favicons?domain={t.domain}&sz=64"
                        alt=""
                        class="size-6 shrink-0 rounded-sm object-contain"
                />
                <span class="flex-1 truncate">{t.label}</span>
                {#if isTrackerActive(t.id)}<Check class="size-5 shrink-0" />{/if}
            </button>
        {/each}
    </div>

    {#if sortedExtensions.length > 0}
        <div class="space-y-1">
            <h3 class="mb-1 flex items-center gap-2 px-1 text-sm font-bold uppercase tracking-wider text-muted-foreground">
                <Blocks class="size-4" /> Extensions
            </h3>

            {#each sortedExtensions as ext (ext.id)}
                <button
                        type="button"
                        data-sn
                        data-autofocus={isExtActive(ext.id) ? '' : undefined}
                        onclick={() => onSelectSource('extension', ext.id, 'anilist')}
                        class="{row} {isExtActive(ext.id) ? rowActive : rowIdle}"
                >
                    {#if ext.icon}
                        <img src={ext.icon} alt="" class="size-6 shrink-0 rounded-sm object-contain" />
                    {:else}
                        <Plug class="size-6 shrink-0" />
                    {/if}
                    <span class="flex-1 truncate">{displayNames.get(ext.id) ?? ext.name}</span>
                    {#if isExtActive(ext.id)}<Check class="size-5 shrink-0" />{/if}
                </button>
            {/each}
        </div>
    {/if}
</div>