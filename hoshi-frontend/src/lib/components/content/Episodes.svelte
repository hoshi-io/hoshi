<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import type { AnimeProgress } from "@/api/progress/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import ResponsiveSelect from "$lib/components/ResponsiveSelect.svelte";
    import {scrollIfResume, useEpisodeList} from "@/utils/episode-grid.svelte.js";
    import EpisodeCard from "@/components/card/EpisodeCard.svelte";
    import Check from 'phosphor-svelte/lib/CheckIcon';
    import type { ResumeTarget } from "@/app/content.svelte";

    let { cid, epsOrChapters, contentUnits = [], duration, progress = [], resume = $bindable(null) }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[],
        duration?: number | null,
        progress?: AnimeProgress[],
        resume?: ResumeTarget | null,
    } = $props();

    const list = useEpisodeList({
        cid: () => cid,
        units: () => contentUnits,
        total: () => epsOrChapters,
        duration: () => duration,
        progress: () => progress
    });

    // Without thumbnails (most extension-only shows) the cards would be empty boxes.
    const compact = $derived(!list.episodes.some(ep => ep.thumbnail));

    $effect(() => {
        resume = list.episodes.length
            ? { number: list.resumeEpisode, href: list.resumeHref, started: progress.length > 0 }
            : null;
    });

    const arcItems = $derived(
        list.arcs?.map((arc, i) => ({ value: String(i), label: arc.label })) ?? []
    );
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
        <h2 class="text-base md:text-lg font-bold tracking-tight">
            {i18n.t('content.episodes_title')}
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

    {#if compact}
    <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 xl:grid-cols-10 gap-2">
        {#each list.visible as row (row.number)}
            <a
                    href={row.href}
                    use:scrollIfResume={row.isResume}
                    title={row.title ?? row.label}
                    class="relative flex h-16 flex-col justify-center overflow-hidden rounded-md border px-3 transition-colors hover:bg-muted/50
                           {row.isResume ? 'border-primary/60 bg-primary/10' : 'border-border/40 bg-muted/20'}
                           {row.completed ? 'opacity-50' : ''}"
            >
                <span class="flex items-center gap-1.5 text-sm font-bold tabular-nums">
                    {row.number}
                    {#if row.completed}<Check class="size-3.5 text-primary" weight="bold" />{/if}
                </span>
                {#if row.title}
                    <span class="truncate text-xs text-muted-foreground">{row.title}</span>
                {:else if row.tag}
                    <span class="truncate text-xs font-semibold text-primary">{row.tag}</span>
                {/if}
                {#if row.pct}
                    <span class="absolute inset-x-0 bottom-0 h-0.5 bg-primary" style="width: {row.pct}%"></span>
                {/if}
            </a>
        {/each}
    </div>
    {:else}
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-3 gap-y-6">
        {#each list.visible as row (row.number)}
            <div use:scrollIfResume={row.isResume}>
                <EpisodeCard
                        stacked
                        nativeImg
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
    {/if}
</div>