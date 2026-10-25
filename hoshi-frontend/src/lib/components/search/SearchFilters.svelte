<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { searchState } from "@/app/search.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import {ANIME_FORMATS, formatLabel, MANGA_FORMATS, TRACKER_FILTERS} from "@/utils/search.js";

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
        tracker: "anilist" | "mal" | "kitsu" | "simkl";
        status: string;
        genre: string;
        format: string;
        nsfw: boolean;
        extFiltersSchema: Record<string, any>;
        extFilterValues: Record<string, any>;
        onChange: () => void;
        onClear: () => void;
    } = $props();


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

        if (tracker == "kitsu") {
            nsfw = false;
        }
    });
</script>

<div class="space-y-6 w-full">
    {#if searchMode === "tracker" && tracker != "simkl"}
        <div class="space-y-5">
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.status')}</Label>
                <ResponsiveSelect
                        bind:value={status}
                        items={[
            { value: "", label: i18n.t('search.any_status') },
            ...activeFilters.status.map((st: { value: any; label: string; }) => ({ value: st.value, label: i18n.t(st.label) }))
        ]}
                        onValueChange={handleFilterChange}
                        placeholder={i18n.t('search.any_status')}
                        class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                />
            </div>

            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.genre')}</Label>
                <ResponsiveSelect
                        bind:value={genre}
                        items={[
            { value: "", label: i18n.t('search.any_genre') },
            ...activeFilters.genres.map((gen: { value: any; label: string; }) => ({ value: gen.value, label: i18n.t(gen.label) }))
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
            ...filteredFormats.map((form: { value: any; label: string; }) => ({ value: form.value, label: i18n.t(form.label) }))
        ]}
                        onValueChange={handleFormatChange}
                        placeholder={i18n.t('search.any_format')}
                        class="bg-muted/20 border-none h-11 rounded-sm font-semibold"
                />
            </div>

            {#if tracker === 'anilist' || tracker === 'mal'}
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

                            <div class="flex flex-wrap gap-2 max-h-36 overflow-y-auto p-2 rounded-sm border border-border/40 bg-muted/5 [scrollbar-width:thin]">
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