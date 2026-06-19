<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import type { AnimeProgress } from "@/api/progress/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { CheckCircle2, Play } from "lucide-svelte";
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

    const ARC_SIZE = 12;
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

<div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
        <h2 class="text-base md:text-lg font-bold tracking-tight">
            {i18n.t('content.episodes_title', { default: 'Episodes' })}
            {#if epsOrChapters}
                <span class="text-muted-foreground/50 font-medium">· {epsOrChapters}</span>
            {/if}
        </h2>

        {#if arcs}
            <ResponsiveSelect
                    bind:value={selectedArc}
                    items={arcItems}
                    placeholder="..."
                    class="w-auto min-w-[7rem] h-8 text-xs rounded-sm"
            />
        {/if}
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-x-3 gap-y-6">
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
                    class="group flex flex-col"
            >
                <div class="relative w-full aspect-video overflow-hidden rounded-sm bg-muted/20
                    {isResume ? 'ring-1 ring-primary/60' : ''}">
                    {#if ep.thumbnail}
                        <img
                                src={ep.thumbnail}
                                alt={ep.title ?? `EP ${ep.number}`}
                                class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105 {isCompleted ? 'opacity-40' : ''}"
                        />
                    {:else}
                        <div class="w-full h-full flex items-center justify-center bg-muted/10">
                            <span class="text-3xl font-black text-white/10">{ep.number}</span>
                        </div>
                    {/if}

                    <div class="absolute inset-x-0 top-0 h-12 bg-gradient-to-b from-black/70 to-transparent pointer-events-none"></div>

                    {#if ep.duration}
                        <div class="absolute top-2 left-2.5 text-[13px] font-bold text-white [text-shadow:0_1px_3px_rgba(0,0,0,0.8)]">
                            {formatDuration(ep.duration)}
                        </div>
                    {/if}

                    <div class="absolute inset-0 flex items-center justify-center bg-black/0 group-hover:bg-black/30 transition-colors duration-200">
                        <div class="w-9 h-9 rounded-full bg-white/90 flex items-center justify-center scale-75 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all duration-200">
                            <Play class="w-4 h-4 text-black fill-black ml-0.5" />
                        </div>
                    </div>

                    {#if isCompleted}
                        <div class="absolute inset-0 flex items-center justify-center">
                            <CheckCircle2 class="w-7 h-7 text-primary/80" />
                        </div>
                    {/if}

                    {#if progressPct !== null}
                        <div class="absolute bottom-0 inset-x-0 h-1 bg-white/15">
                            <div
                                    class="h-full bg-primary transition-all duration-300"
                                    style="width: {progressPct}%"
                            ></div>
                        </div>
                    {/if}
                </div>

                <div class="pt-2.5 space-y-0.5">
                    <div class="flex items-center gap-2">
                        <p class="text-[11px] font-bold uppercase tracking-wide text-muted-foreground/60">
                            {i18n.t('content.episode_title', { num: ep.number })}
                        </p>
                        {#if isResume && !isCompleted}
                            <span class="text-[10px] font-bold uppercase tracking-wide text-primary/80">
                                {isInProgress ? `· ${i18n.t('content.resume')}` : `· ${i18n.t('home.hero.watch')}`}
                            </span>
                        {/if}
                    </div>
                    {#if isRich && ep.title}
                        <p class="text-sm font-semibold leading-snug line-clamp-2 group-hover:text-primary transition-colors">
                            {ep.title}
                        </p>
                    {/if}
                    {#if isInProgress && prog?.timestampSeconds}
                        <p class="text-[10px] text-muted-foreground/40 font-mono tabular-nums pt-0.5">
                            {formatTimestamp(prog.timestampSeconds)}
                            {#if progressPct !== null}· {progressPct}%{/if}
                        </p>
                    {/if}
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