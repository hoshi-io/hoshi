<script lang="ts">
    import { Plug, Database, Blocks, ChevronRight } from "lucide-svelte";
    import { searchState } from "@/app/search.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import {extensionsApi} from "@/api/extensions/extensions";

    let {
        isMobile = false,
        availableExtensions = [],
        onSelectSource
    }: {
        isMobile?: boolean;
        availableExtensions: any[];
        onSelectSource: (mode: "tracker" | "extension", extId: string, tracker: "anilist" | "mal" | "kitsu" | "simkl", isMobile: boolean) => void;
    } = $props();

    let trackersOpen = $state(true);
    let extensionsOpen = $state(false);

    let sortedExtensions = $derived(
        [...availableExtensions].sort((a, b) =>
            (a.name || "").localeCompare(b.name || "")
        )
    );

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            'anilist': 'anilist.co',
            'mal': 'myanimelist.net',
            'kitsu': 'kitsu.io',
            'simkl': 'simkl.com'
        };

        const domain = domains[trackerName.toLowerCase()] || 'google.com';
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    const rowClasses = "flex items-center gap-3 px-2 py-1.5 w-full group outline-none rounded-md transition-colors text-sm font-medium";
    const activeRow = "bg-primary/10 text-primary";
    const inactiveRow = "text-foreground/80 hover:bg-muted/50 hover:text-foreground";
</script>

<div class="space-y-4 w-full select-none">
    <div class="space-y-1">
        <button
                type="button"
                onclick={() => trackersOpen = !trackersOpen}
                class="flex items-center justify-between w-full px-1 py-1.5 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors uppercase tracking-wider"
        >
            <span class="flex items-center gap-2">
                <Database class="w-3.5 h-3.5" /> Trackers
            </span>
            <ChevronRight class="w-4 h-4 transition-transform duration-200 {trackersOpen ? 'rotate-90' : ''}" />
        </button>

        {#if trackersOpen}
            <div class="space-y-0.5 animate-in slide-in-from-top-2 fade-in duration-200">
                <button
                        type="button"
                        onclick={() => onSelectSource('tracker', '', 'anilist', isMobile)}
                        class="{rowClasses} {searchState.searchMode === 'tracker' && searchState.tracker === 'anilist' ? activeRow : inactiveRow}"
                >
                    <img src={getTrackerFavicon('anilist')} alt="AniList" class="w-5 h-5 rounded-sm object-contain {searchState.searchMode === 'tracker' && searchState.tracker === 'anilist' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
                    <span class="truncate">AniList</span>
                </button>

                <button
                        type="button"
                        onclick={() => onSelectSource('tracker', '', 'mal', isMobile)}
                        class="{rowClasses} {searchState.searchMode === 'tracker' && searchState.tracker === 'mal' ? activeRow : inactiveRow}"
                >
                    <img src={getTrackerFavicon('mal')} alt="MAL" class="w-5 h-5 rounded-sm object-contain {searchState.searchMode === 'tracker' && searchState.tracker === 'mal' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
                    <span class="truncate">MAL</span>
                </button>

                <button
                        type="button"
                        onclick={() => onSelectSource('tracker', '', 'kitsu', isMobile)}
                        class="{rowClasses} {searchState.searchMode === 'tracker' && searchState.tracker === 'kitsu' ? activeRow : inactiveRow}"
                >
                    <img src={getTrackerFavicon('kitsu')} alt="Kitsu" class="w-5 h-5 rounded-sm object-contain {searchState.searchMode === 'tracker' && searchState.tracker === 'kitsu' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
                    <span class="truncate">Kitsu</span>
                </button>

                {#if searchState.contentType === "anime"}
                    <button
                            type="button"
                            onclick={() => onSelectSource('tracker', '', 'simkl', isMobile)}
                            class="{rowClasses} {searchState.searchMode === 'tracker' && searchState.tracker === 'simkl' ? activeRow : inactiveRow}"
                    >
                        <img src={getTrackerFavicon('simkl')} alt="SIMKL" class="w-5 h-5 rounded-sm object-contain {searchState.searchMode === 'tracker' && searchState.tracker === 'simkl' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
                        <span class="truncate">SIMKL</span>
                    </button>
                {/if}
            </div>
        {/if}
    </div>

    {#if availableExtensions.length > 0}
        {#if availableExtensions.length > 0}
            <div class="space-y-1">
                <button
                        type="button"
                        onclick={() => extensionsOpen = !extensionsOpen}
                        class="flex items-center justify-between w-full px-1 py-1.5 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors uppercase tracking-wider"
                >
                <span class="flex items-center gap-2">
                    <Blocks class="w-3.5 h-3.5" /> Extensions
                </span>
                    <ChevronRight class="w-4 h-4 transition-transform duration-200 {extensionsOpen ? 'rotate-90' : ''}" />
                </button>

                <!-- SearchSourceGrid.svelte -->

                {#if extensionsOpen}
                    <div class="space-y-0.5 max-h-[300px] overflow-y-auto scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent animate-in slide-in-from-top-2 fade-in duration-200">
                        {#each sortedExtensions as ext}
                            <!-- 1. Enforced a rigid layout row height (h-9 min-h-[36px] max-h-[36px]) so it never expands -->
                            <div class="{rowClasses} pr-1.5 justify-between h-9 min-h-[36px] max-h-[36px] {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? activeRow : inactiveRow}">

                                <button
                                        type="button"
                                        onclick={() => onSelectSource('extension', ext.id, 'anilist', isMobile)}
                                        class="flex items-center gap-3 flex-1 min-w-0 h-full outline-none text-left"
                                >
                                    {#if ext.icon}
                                        <img src={ext.icon} class="w-5 h-5 shrink-0 rounded-sm object-contain {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" alt={ext.name} />
                                    {:else}
                                        <Plug class="w-5 h-5 shrink-0 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? 'text-primary' : 'text-muted-foreground'}" />
                                    {/if}
                                    <span class="truncate">{ext.name}</span>
                                </button>

                                {#if ext.source === 'tachiyomi' && ext.settings}
                                    {@const langDef = ext.setting_definitions?.find(s => s.id === 'language' || s.key === 'language')}
                                    {#if langDef}
                                        <!-- 2. Replaced w-[85px] with strict height capping (!h-6) to ensure the select stays compact -->
                                        <div class="w-[85px] shrink-0 ml-2 h-6 flex items-center" onclick={(e) => e.stopPropagation()}>
                                            <ResponsiveSelect
                                                    value={ext.settings.language as string}
                                                    items={langDef.options || []}
                                                    placeholder="Lang"
                                                    data-size="sm"
                                                    class="!h-6 text-xs px-2 py-0 bg-transparent border-0 border-transparent shadow-none focus:ring-0 focus:ring-offset-0 hover:bg-muted/50 transition-colors"
                                                    onValueChange={(newVal) => {
                                        ext.settings.language = newVal;
                                        extensionsApi.updateSettings(ext.id, {
                                            ...ext.settings,
                                            language: newVal
                                        });
                                    }}
                                            />
                                        </div>
                                    {/if}
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    {/if}
</div>

<style>
    .scrollbar-thin::-webkit-scrollbar { width: 4px; }
    .scrollbar-thin::-webkit-scrollbar-track { background: transparent; }
    .scrollbar-thin::-webkit-scrollbar-thumb { background-color: hsl(var(--border) / 0.5); border-radius: 20px; }
    .scrollbar-thin:hover::-webkit-scrollbar-thumb { background-color: hsl(var(--border)); }
</style>