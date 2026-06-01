<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { searchState } from "@/app/search.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        searchMode,
        tracker,
        status = $bindable(),
        genre = $bindable(),
        format = $bindable(),
        nsfw = $bindable(),
        extFiltersSchema,
        extFilterValues = $bindable(),
        onChange,
        onClear
    }: {
        searchMode: "tracker" | "extension";
        tracker: "anilist" | "mal" | "kitsu";
        status: string;
        genre: string;
        format: string;
        nsfw: boolean;
        extFiltersSchema: Record<string, any>;
        extFilterValues: Record<string, any>;
        onChange: () => void;
        onClear: () => void;
    } = $props();

    const MANGA_FORMATS = ["MANGA", "NOVEL", "LIGHT_NOVEL", "ONE_SHOT", "DOUJIN", "MANHWA", "MANHUA", "manga", "novel", "light_novel", "one_shot", "doujin", "manhwa", "manhua"];
    const ANIME_FORMATS = ["TV", "TV_SHORT", "MOVIE", "SPECIAL", "OVA", "ONA", "MUSIC", "tv", "movie", "ova", "ona", "special", "music"];

    const formatLabel = (key: string) => key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());

    const TRACKER_FILTERS = {
        anilist: {
            status: [
                { value: "FINISHED", label: "search.completed" },
                { value: "RELEASING", label: "search.ongoing" },
                { value: "NOT_YET_RELEASED", label: "search.planned" },
                { value: "CANCELLED", label: "search.cancelled" },
                { value: "HIATUS", label: "search.hiatus" },
            ],
            formats: [
                { value: "TV", label: "search.tv" },
                { value: "TV_SHORT", label: "search.tv_short" },
                { value: "MOVIE", label: "search.movie" },
                { value: "SPECIAL", label: "search.special" },
                { value: "OVA", label: "search.ova" },
                { value: "ONA", label: "search.ona" },
                { value: "MANGA", label: "search.manga" },
                { value: "NOVEL", label: "search.novel" },
                { value: "ONE_SHOT", label: "search.one_shot" },
            ],
            genres: [
                { value: "Action", label: "search.action" },
                { value: "Adventure", label: "search.adventure" },
                { value: "Comedy", label: "search.comedy" },
                { value: "Drama", label: "search.drama" },
                { value: "Ecchi", label: "search.ecchi" },
                { value: "Fantasy", label: "search.fantasy" },
                { value: "Horror", label: "search.horror" },
                { value: "Mahou Shoujo", label: "search.mahou_shoujo" },
                { value: "Mecha", label: "search.mecha" },
                { value: "Music", label: "search.music" },
                { value: "Mystery", label: "search.mystery" },
                { value: "Psychological", label: "search.psychological" },
                { value: "Romance", label: "search.romance" },
                { value: "Sci-Fi", label: "search.sci_fi" },
                { value: "Slice of Life", label: "search.slice_of_life" },
                { value: "Sports", label: "search.sports" },
                { value: "Supernatural", label: "search.supernatural" },
                { value: "Thriller", label: "search.thriller" },
            ],
        },
        mal: {
            status: [
                { value: "airing", label: "search.ongoing" },
                { value: "complete", label: "search.completed" },
                { value: "upcoming", label: "search.planned" },
            ],
            formats: [
                { value: "tv", label: "search.tv" },
                { value: "movie", label: "search.movie" },
                { value: "ova", label: "search.ova" },
                { value: "ona", label: "search.ona" },
                { value: "special", label: "search.special" },
                { value: "manga", label: "search.manga" },
                { value: "novel", label: "search.novel" },
                { value: "light_novel", label: "search.light_novel" },
                { value: "one_shot", label: "search.one_shot" },
                { value: "doujin", label: "search.doujin" },
                { value: "manhwa", label: "search.manhwa" },
                { value: "manhua", label: "search.manhua" },
            ],
            genres: [
                { value: "1", label: "search.action" },
                { value: "2", label: "search.adventure" },
                { value: "4", label: "search.comedy" },
                { value: "8", label: "search.drama" },
                { value: "10", label: "search.fantasy" },
                { value: "14", label: "search.horror" },
                { value: "7", label: "search.mystery" },
                { value: "37", label: "search.supernatural" },
                { value: "22", label: "search.romance" },
                { value: "24", label: "search.sci_fi" },
                { value: "36", label: "search.slice_of_life" },
                { value: "30", label: "search.sports" },
            ],
        },
        kitsu: {
            status: [
                { value: "current", label: "search.ongoing" },
                { value: "finished", label: "search.completed" },
                { value: "upcoming", label: "search.planned" },
            ],
            formats: [
                { value: "TV", label: "search.tv" },
                { value: "movie", label: "search.movie" },
                { value: "OVA", label: "search.ova" },
                { value: "manga", label: "search.manga" },
                { value: "novel", label: "search.novel" },
            ],
            genres: [
                { value: "action", label: "search.action" },
                { value: "adventure", label: "search.adventure" },
                { value: "comedy", label: "search.comedy" },
            ],
        }
    };


    let activeFilters = $derived(TRACKER_FILTERS[tracker] || TRACKER_FILTERS.anilist);

    let filteredFormats = $derived(
        activeFilters.formats.filter((f: any) => {
            if (searchState.contentType === "anime") {
                return !MANGA_FORMATS.includes(f.value);
            } else {
                return !ANIME_FORMATS.includes(f.value);
            }
        })
    );

    function handleFilterChange() {
        searchState.page = 1;
        onChange();
    }

    function handleFormatChange(newFormat: string) {
        format = newFormat;

        if (MANGA_FORMATS.includes(newFormat) && searchState.contentType !== "manga") {
            searchState.contentType = "manga";
        } else if (ANIME_FORMATS.includes(newFormat) && searchState.contentType !== "anime") {
            searchState.contentType = "anime";
        }

        handleFilterChange();
    }

    const toggleMultiSelect = (key: string, value: string) => {
        if (!extFilterValues[key]) extFilterValues[key] = [];
        const index = extFilterValues[key].indexOf(value);
        if (index > -1) {
            extFilterValues[key] = extFilterValues[key].filter((v: string) => v !== value);
        } else {
            extFilterValues[key] = [...extFilterValues[key], value];
        }
        handleFilterChange();
    };

    $effect(() => {
        const _t = tracker;
        status = "";
        genre = "";
        format = "";

        if (tracker !== "anilist") {
            nsfw = false;
        }
    });
</script>

<div class="space-y-6 w-full">
    {#if searchMode === "tracker"}
        <div class="space-y-5">
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.status')}</Label>
                <ResponsiveSelect
                        bind:value={status}
                        items={[
            { value: "", label: i18n.t('search.any_status') },
            ...activeFilters.status.map(st => ({ value: st.value, label: i18n.t(st.label) }))
        ]}
                        onValueChange={handleFilterChange}
                        placeholder={i18n.t('search.any_status')}
                        class="bg-muted/20 border-none h-1 rounded-sm font-semibold"
                />
            </div>

            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.genre')}</Label>
                <ResponsiveSelect
                        bind:value={genre}
                        items={[
            { value: "", label: i18n.t('search.any_genre') },
            ...activeFilters.genres.map(gen => ({ value: gen.value, label: i18n.t(gen.label) }))
        ]}
                        onValueChange={handleFilterChange}
                        placeholder={i18n.t('search.any_genre')}
                        class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                />
            </div>

            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.format')}</Label>
                <ResponsiveSelect
                        bind:value={format}
                        items={[
            { value: "", label: i18n.t('search.any_format') },
            ...filteredFormats.map(form => ({ value: form.value, label: i18n.t(form.label) }))
        ]}
                        onValueChange={handleFormatChange}
                        placeholder={i18n.t('search.any_format')}
                        class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                />
            </div>

            {#if tracker === 'anilist'}
                <div class="flex items-center space-x-3 pt-2">
                    <Switch id="nsfw-mode" bind:checked={nsfw} onCheckedChange={handleFilterChange} />
                    <Label for="nsfw-mode" class="text-sm font-bold text-foreground/90">{i18n.t('search.nsfw_only')}</Label>
                </div>
            {/if}
        </div>

    {:else if searchMode === "extension" && Object.keys(extFiltersSchema).length > 0}
        <div class="space-y-4">
            {#each Object.entries(extFiltersSchema) as [key, filterDef]}
                <div class="space-y-2.5">
                    {#if filterDef.type === 'select'}
                        <Label class="text-sm font-bold text-foreground/90">
                            {filterDef.label || formatLabel(key)}
                        </Label>
                        <ResponsiveSelect
                                bind:value={extFilterValues[key]}
                                items={[
            { value: "", label: i18n.t('search.any_genre') },
            ...(filterDef.options || []).map((opt: any) => ({ value: opt.value, label: opt.label }))
        ]}
                                onValueChange={handleFilterChange}
                                placeholder={i18n.t('search.any_genre')}
                                class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                        />
                    {:else if filterDef.type === 'multiselect'}
                        <div class="space-y-3">
                            <Label class="text-sm font-bold text-foreground/90">
                                {filterDef.label || formatLabel(key)}
                            </Label>

                            <div class="flex flex-wrap gap-2">
                                {#each filterDef.options || [] as opt}
                                    {@const isSelected = extFilterValues[key]?.includes(opt.value)}
                                    <button
                                            type="button"
                                            class="px-3 py-1.5 rounded-sm text-xs font-bold transition-all border
                    {isSelected
                        ? 'bg-primary text-primary-foreground border-primary'
                        : 'bg-muted/20 text-foreground/70 border-transparent hover:border-border'}"
                                            onclick={() => toggleMultiSelect(key, opt.value)}
                                    >
                                        {opt.label}
                                        {#if isSelected}
                                            <span class="ml-1 opacity-70">✕</span>
                                        {/if}
                                    </button>
                                {/each}
                            </div>
                        </div>
                    {:else if filterDef.type === 'boolean'}
                        <div class="flex items-center space-x-3 pt-2">
                            <Switch id={`filter-${key}`} bind:checked={extFilterValues[key]} onCheckedChange={handleFilterChange} />
                            <Label for={`filter-${key}`} class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}

    <div class="pt-6 border-t border-border/40">
        <Button type="button" variant="secondary" class="w-full h-11 rounded-sm font-bold hover:bg-destructive hover:text-destructive-foreground transition-colors" onclick={onClear}>
            {i18n.t('search.clear_filters')}
        </Button>
    </div>
</div>