<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import type { AnimeProgress } from "@/api/progress/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { CheckCircle2 } from "lucide-svelte";
    import ResponsiveSelect from "$lib/components/ResponsiveSelect.svelte";
    import MpvLauncher from "@/components/mpv/MpvLauncher.svelte";
    import {appConfig} from "@/stores/config.svelte";

    let { cid, epsOrChapters, contentUnits = [], duration, progress = [], animeTitle = "", isNsfw, coverImage }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[],
        duration?: number | null,
        animeTitle?: string,
        isNsfw?: boolean,
        coverImage?: string,
        progress?: AnimeProgress[],
    } = $props();

    const ARC_SIZE = 24;
    const progressMap = $derived(
        new Map(progress.map(p => [p.episode, p]))
    );

    const displayEpisodes = $derived.by(() => {
        const enrichedEpisodes = (contentUnits ?? [])
            .filter(u =>
                u.contentType === 'episode' &&
                u.title &&
                u.thumbnailUrl
            )
            .sort((a, b) => a.unitNumber - b.unitNumber);

        if (enrichedEpisodes.length > 0) {
            return enrichedEpisodes.map(u => ({
                number: u.unitNumber,
                title: u.title,
                description: u.description || null,
                thumbnail: u.thumbnailUrl.replace('_m.', '_w.'),
                enriched: true,
                duration: duration,
            }));
        }

        const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;
        return Array.from({ length: totalEpisodes }, (_, i) => ({
            number: i + 1,
            title: null,
            description: null,
            thumbnail: null,
            enriched: false,
            duration: null,
        }));
    });

    const resumeEpisode = $derived.by(() => {
        const inProgress = progress
            .filter(p => !p.completed && p.timestampSeconds && p.timestampSeconds > 0)
            .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];
        if (inProgress) return inProgress.episode;

        const completedNums = new Set(progress.filter(p => p.completed).map(p => p.episode));
        const firstUnwatched = displayEpisodes.find(ep => !completedNums.has(ep.number));
        return firstUnwatched?.number ?? displayEpisodes[0]?.number ?? 1;
    });

    const arcs = $derived.by(() => {
        if (displayEpisodes.length <= ARC_SIZE) return null;
        const chunks: { start: number; end: number; label: string }[] = [];
        for (let i = 0; i < displayEpisodes.length; i += ARC_SIZE) {
            const slice = displayEpisodes.slice(i, i + ARC_SIZE);
            const start = slice[0].number;
            const end = slice[slice.length - 1].number;
            chunks.push({ start, end, label: `${start}–${end}` });
        }
        return chunks;
    });

    const resumeArcIndex = $derived.by(() => {
        if (!arcs) return 0;
        const idx = arcs.findIndex(a => resumeEpisode >= a.start && resumeEpisode <= a.end);
        return idx >= 0 ? idx : 0;
    });

    let selectedArc = $state("0");

    $effect(() => {
        selectedArc = String(resumeArcIndex);
    });

    const arcItems = $derived(
        arcs?.map((arc, i) => ({
            value: String(i),
            label: arc.label,
        })) ?? []
    );

    const visibleEpisodes = $derived.by(() => {
        if (!arcs) return displayEpisodes;
        const arc = arcs[Number(selectedArc)];
        return displayEpisodes.filter(ep => ep.number >= arc.start && ep.number <= arc.end);
    });

    const formatDuration = (minutes: number | null) => {
        if (!minutes || minutes <= 0) return '';
        if (minutes < 60) return `${minutes}m`;
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return m > 0 ? `${h}h ${m}m` : `${h}h`;
    };

    const formatTimestamp = (seconds: number) => {
        const m = Math.floor(seconds / 60);
        const s = seconds % 60;
        return `${m}:${s.toString().padStart(2, '0')}`;
    };

    const isRich = $derived(displayEpisodes.some(e => e.enriched));

    function scrollIfResume(node: HTMLElement, isResume: boolean) {
        if (isResume) {
            requestAnimationFrame(() => {
                node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            });
        }
        return {
            update(newIsResume: boolean) {
                if (newIsResume) {
                    requestAnimationFrame(() => {
                        node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                    });
                }
            }
        };
    }

    // Launcher state
    let mpvOpen = $state(false);
    let mpvEpNumber = $state(1);
    let mpvEpTitle = $state("");

    function handleEpisodeClick(e: MouseEvent, ep: { number: number; title: string | null }) {
        if (appConfig.data?.mpv?.useMpv){
            e.preventDefault();
            mpvEpNumber = ep.number;
            mpvEpTitle = ep.title
                ? i18n.t('watch.episode_with_title', { num: ep.number, title: ep.title })
                : i18n.t('watch.episode_number', { num: ep.number });
            mpvOpen = true;
            return;
        }
    }
</script>

<div class="flex flex-col h-full space-y-4">
    {#if arcs}
        <ResponsiveSelect
                bind:value={selectedArc}
                items={arcItems}
                placeholder="..."
                class="w-auto min-w-[7rem]"
        />
    {/if}

    <div class="flex-1 overflow-y-auto pr-2 space-y-3 hide-scrollbar">
        {#each visibleEpisodes as ep (ep.number)}
            {@const prog = progressMap.get(ep.number)}
            {@const isCompleted = prog?.completed ?? false}
            {@const isInProgress = !isCompleted && (prog?.timestampSeconds ?? 0) > 0}
            {@const isResume = ep.number === resumeEpisode}
            {@const progressPct = isInProgress && prog?.episodeDurationSeconds && prog.episodeDurationSeconds > 0
                ? Math.min(100, Math.round((prog.timestampSeconds! / prog.episodeDurationSeconds) * 100))
                : null}
            {@const href = isInProgress && prog?.timestampSeconds
                ? `/watch/${cid}/${ep.number}?t=${prog.timestampSeconds}`
                : `/watch/${cid}/${ep.number}`}

            <a
                    use:scrollIfResume={isResume}
                    {href}
                    onclick={(e) => handleEpisodeClick(e, ep)}
                    class="group relative flex gap-4 border transition-all duration-200
                        {isCompleted
                        ? 'border-white/5 bg-white/[0.02] opacity-40 hover:opacity-70 hover:bg-white/[0.04] hover:border-white/8'
                        : isResume
                            ? 'border-primary/30 bg-primary/5 hover:bg-primary/10 hover:border-primary/50 ring-1 ring-primary/20'
                            : 'border-white/5 bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/10'}"
            >
                <div class="relative shrink-0 w-48 aspect-video overflow-hidden bg-muted/20">
                    {#if ep.thumbnail}
                        <img
                                src={ep.thumbnail}
                                alt={ep.title ?? `EP ${ep.number}`}
                                class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                        />
                    {:else}
                        <div class="w-full h-full flex items-center justify-center">
                            <span class="text-4xl font-black text-white/5">{ep.number}</span>
                        </div>
                    {/if}

                    {#if ep.duration}
                        <div class="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/80 text-[10px] font-medium text-white rounded">
                            {formatDuration(ep.duration)}
                        </div>
                    {/if}

                    {#if isCompleted}
                        <div class="absolute inset-0 flex items-center justify-center bg-black/30">
                            <CheckCircle2 class="w-8 h-8 text-primary/70" />
                        </div>
                    {/if}

                    {#if progressPct !== null}
                        <div class="absolute bottom-0 inset-x-0 h-1 bg-white/10">
                            <div
                                    class="h-full bg-primary transition-all duration-300"
                                    style="width: {progressPct}%"
                            ></div>
                        </div>
                    {/if}
                </div>

                <div class="flex-1 min-w-0 py-3 pr-4 flex flex-col justify-between">
                    <div class="space-y-1">
                        <div class="flex justify-between items-start gap-2">
                            <p class="font-bold text-[15px] leading-tight line-clamp-1 group-hover:text-primary transition-colors">
                                {#if isRich && ep.title}
                                    {ep.number}. {ep.title}
                                {:else}
                                    {i18n.t('content.episode_title', { num: ep.number })}
                                {/if}
                            </p>
                        </div>
                        {#if ep.description}
                            <p class="text-[11px] text-muted-foreground/60 line-clamp-2 leading-relaxed">
                                {ep.description}
                            </p>
                        {/if}
                    </div>

                    <div class="flex items-center gap-2 mt-2">
                        {#if isResume && !isCompleted}
                            <span class="text-[10px] font-bold uppercase tracking-widest text-primary/80 bg-primary/10 px-2 py-0.5 rounded-sm">
                                {isInProgress ? i18n.t('content.resume') : i18n.t('home.hero.watch')}
                            </span>
                        {/if}
                        {#if isInProgress && prog?.timestampSeconds}
                            <span class="text-[10px] text-muted-foreground/40 font-mono tabular-nums">
                                {formatTimestamp(prog.timestampSeconds)}
                                {#if progressPct !== null}· {progressPct}%{/if}
                            </span>
                        {/if}
                    </div>
                </div>
            </a>
        {/each}
    </div>
</div>

{#if mpvOpen}
    <MpvLauncher
            {cid}
            epNumber={mpvEpNumber}
            epTitle={mpvEpTitle}
            {animeTitle}
            totalEpisodes={epsOrChapters ?? 0}
            {isNsfw}
            {coverImage}
            startTime={progressMap.get(mpvEpNumber)?.timestampSeconds ?? 0}
            bind:open={mpvOpen}
    />
{/if}

<style>
    .hide-scrollbar::-webkit-scrollbar { display: none; }
    .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>