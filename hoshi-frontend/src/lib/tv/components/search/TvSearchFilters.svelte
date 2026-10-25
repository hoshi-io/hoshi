<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import { searchState } from '@/app/search.svelte.js';
    import TvChoice from '@/tv/TvChoice.svelte';
    import TvToggle from '@/tv/TvToggle.svelte';
    import {formatLabel, MANGA_FORMATS, TRACKER_FILTERS} from "@/utils/SearchFilters.js";

    let {
        extFiltersSchema,
        onChange
    }: {
        extFiltersSchema: Record<string, any>;
        /** Called after every change. The page decides when to actually search (on panel close). */
        onChange: () => void;
    } = $props();

    type TrackerFilters = (typeof TRACKER_FILTERS)['anilist'];
    const activeFilters = $derived<TrackerFilters>(
        (TRACKER_FILTERS as Record<string, TrackerFilters>)[searchState.tracker] ?? TRACKER_FILTERS.anilist
    );

    const anyOption = (key: string) => ({ value: '', label: i18n.t(key) });
    const mapOptions = (list: { value: string; label: string }[]) =>
        list.map((o) => ({ value: o.value, label: i18n.t(o.label) }));

    const statusItems = $derived([anyOption('search.any_status'), ...mapOptions(activeFilters.status)]);
    const genreItems = $derived([anyOption('search.any_genre'), ...mapOptions(activeFilters.genres)]);
    // TV is anime-only: drop every manga/novel format.
    const formatItems = $derived([
        anyOption('search.any_format'),
        ...mapOptions(activeFilters.formats.filter((f: { value: string }) => !MANGA_FORMATS.includes(f.value)))
    ]);

    const showTrackerFilters = $derived(searchState.searchMode === 'tracker' && searchState.tracker !== 'simkl');
    const showExtFilters = $derived(searchState.searchMode === 'extension' && Object.keys(extFiltersSchema).length > 0);

    function changed() {
        searchState.page = 1;
        onChange();
    }

    function toggleMulti(key: string, value: string) {
        const current: string[] = searchState.extFilterValues[key] ?? [];
        searchState.extFilterValues[key] = current.includes(value)
            ? current.filter((v) => v !== value)
            : [...current, value];
        changed();
    }

    const heading = 'mb-2 text-lg font-bold text-foreground';
</script>

<div class="space-y-6 pb-2">
    {#if showTrackerFilters}
        <div>
            <h3 class={heading}>{i18n.t('search.status')}</h3>
            <TvChoice size="sm" items={statusItems} bind:value={searchState.status} onselect={changed} />
        </div>

        <div>
            <h3 class={heading}>{i18n.t('search.format')}</h3>
            <TvChoice size="sm" items={formatItems} bind:value={searchState.format} onselect={changed} />
        </div>

        <div>
            <h3 class={heading}>{i18n.t('search.genre')}</h3>
            <TvChoice size="sm" items={genreItems} bind:value={searchState.genre} onselect={changed} />
        </div>

        {#if searchState.tracker === 'anilist' || searchState.tracker === 'mal'}
            <TvToggle
                    label={i18n.t('search.nsfw_only')}
                    pressed={searchState.nsfw}
                    onclick={() => {
                    searchState.nsfw = !searchState.nsfw;
                    changed();
                }}
            />
        {/if}
    {:else if showExtFilters}
        {#each Object.entries(extFiltersSchema) as [key, def] (key)}
            <div>
                {#if def.type === 'select'}
                    <h3 class={heading}>{def.label || formatLabel(key)}</h3>
                    <TvChoice
                            size="sm"
                            items={[anyOption('search.any_genre'), ...(def.options || []).map((o: any) => ({ value: o.value, label: o.label }))]}
                            value={searchState.extFilterValues[key] ?? ''}
                            onselect={(v) => {
                            searchState.extFilterValues[key] = v;
                            changed();
                        }}
                    />
                {:else if def.type === 'multiselect'}
                    <h3 class={heading}>{def.label || formatLabel(key)}</h3>
                    <div class="flex flex-wrap gap-2">
                        {#each def.options || [] as opt (opt.value)}
                            <TvToggle
                                    label={opt.label}
                                    pressed={!!searchState.extFilterValues[key]?.includes(opt.value)}
                                    onclick={() => toggleMulti(key, opt.value)}
                            />
                        {/each}
                    </div>
                {:else if def.type === 'boolean'}
                    <TvToggle
                            label={def.label || formatLabel(key)}
                            pressed={!!searchState.extFilterValues[key]}
                            onclick={() => {
                            searchState.extFilterValues[key] = !searchState.extFilterValues[key];
                            changed();
                        }}
                    />
                {/if}
            </div>
        {/each}
    {/if}
</div>