<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import type { AnimeProgress } from "@/api/progress/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import ResponsiveSelect from "$lib/components/ResponsiveSelect.svelte";
    import EpisodeCard from "@/components/card/EpisodeCard.svelte";
    import {scrollIfResume, useEpisodeList} from "@/utils/episode-grid.svelte.js";

    let { cid, epsOrChapters, contentUnits = [], duration, progress = [] }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[],
        duration?: number | null,
        progress?: AnimeProgress[],
    } = $props();

    const list = useEpisodeList({
        cid: () => cid,
        units: () => contentUnits,
        total: () => epsOrChapters,
        duration: () => duration,
        progress: () => progress
    });

    const arcItems = $derived(
        list.arcs?.map((arc, i) => ({ value: String(i), label: arc.label })) ?? []
    );
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
        <h2 class="text-base md:text-lg font-bold tracking-tight">
            {i18n.t('content.episodes_title', { default: 'Episodes' })}
            {#if epsOrChapters}
                <span class="text-muted-foreground/50 font-medium">· {epsOrChapters}</span>
            {/if}
        </h2>

        {#if list.arcs}
            <ResponsiveSelect
                    bind:value={() => String(list.selectedArc), (v) => (list.selectedArc = Number(v))}
                    items={arcItems}
                    placeholder="..."
                    class="w-auto min-w-[7rem] h-8 text-xs rounded-sm"
            />
        {/if}
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-3 gap-y-6">
        {#each list.visible as row (row.number)}
            <div use:scrollIfResume={row.isResume}>
                <EpisodeCard
                        stacked
                        playOnHover
                        href={row.href}
                        image={row.thumbnail}
                        alt={row.title ?? `EP ${row.number}`}
                        episode={row.label}
                        episodeTitle={row.title}
                        tag={row.tag}
                        subtext={row.timestamp}
                        badge={row.duration}
                        progress={row.pct ?? 0}
                        completed={row.completed}
                        highlight={row.isResume}
                        placeholder={String(row.number)}
                />
            </div>
        {/each}
    </div>
</div>