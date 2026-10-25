<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import {
        AlertCircle, BookMarked, Building2, Calendar, Check, CheckCircle2, ChevronDown,
        ChevronUp, Clock, Play, Plus, RefreshCw, Star, Tv
    } from 'lucide-svelte';

    import { ContentDetailState } from '@/app/content.svelte';
    import { listStore } from '@/app/list.svelte.js';
    import { scheduleStore } from '@/app/schedule.svelte.js';
    import { listApi } from '@/api/list/list';
    import type { UpsertEntryBody } from '@/api/list/types';
    import { progressApi } from '@/api/progress/progress';
    import type { AnimeProgress } from '@/api/progress/types';
    import { primaryMetadata } from '@/api/content/types';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { appConfig } from '@/stores/config.svelte';
    import { formatScore } from '@/utils/normalize';
    import { Spinner } from '@/components/ui/spinner';
    import SmartImage from '@/components/SmartImage.svelte';
    import TvCard from '@/tv/components/card/TvCard.svelte';
    import ListEditorButton from "@/components/ListEditorButton.svelte";

    const WATCH = '/tv/watch';

    const detail = new ContentDetailState();

    const full = $derived(detail.fullContent);
    const cid = $derived(full?.content.cid ?? '');
    const meta = $derived<any>(
        full ? primaryMetadata(full, appConfig.data?.content?.preferredMetadataProvider) : null
    );
    const pref = $derived(appConfig.data?.ui?.titleLanguage || 'romaji');
    const displayTitle = $derived(meta?.titleI18n?.[pref] || meta?.title || '');
    const isAnime = $derived(full?.content.contentType === 'anime');
    const isMovie = $derived(meta?.subtype === 'MOVIE');
    const isAdult = $derived(
        !!full &&
        (full.content.nsfw ||
            meta?.genres?.some((g: string) => ['hentai', 'adult'].includes(g.toLowerCase())))
    );
    const shouldBlur = $derived(isAdult && (appConfig.data?.general?.blurAdultContent ?? true));
    const score = $derived(meta?.rating ? formatScore(meta.rating) : null);

    // Manga/novels are not supported on TV, so drop them from the related row.
    const relations = $derived(
        detail.relations.filter((r) => {
            const t = String((r.card as any)?.contentType ?? '').toUpperCase();
            return t !== 'MANGA' && t !== 'NOVEL';
        })
    );

    function revealTop(e: FocusEvent) {
        const main = (e.currentTarget as HTMLElement).closest('main');
        requestAnimationFrame(() => main?.scrollTo({ top: 0, behavior: 'smooth' }));
    }

    // ---------------------------------------------------------------- progress
    let animeProgress = $state<AnimeProgress[]>([]);
    let progressFor = '';

    $effect(() => {
        const id = cid;
        if (!id || progressFor === id) return;
        progressFor = id;
        progressApi
            .getContentProgress(id)
            .then((res) => (animeProgress = res.animeProgress ?? []))
            .catch(() => {
                // non-fatal: the page works without progress
            });
    });

    const progressMap = $derived(new Map(animeProgress.map((p) => [p.episode, p])));

    // ---------------------------------------------------------------- episodes
    const ARC_SIZE = 24; // bigger than the web's 12: fewer chips to scroll through on a remote

    const episodes = $derived.by(() => {
        if (!full || !isAnime || isMovie) return [];
        const enriched = (full.contentUnits ?? [])
            .filter((u) => u.contentType === 'episode' && u.title)
            .sort((a, b) => a.unitNumber - b.unitNumber);

        if (enriched.length > 0) {
            return enriched.map((u) => ({
                number: u.unitNumber,
                title: u.title as string | null,
                thumbnail: u.thumbnailUrl?.replace('_m.', '_w.') ?? '',
                duration: (meta?.episodeDuration ?? null) as number | null
            }));
        }

        const total = meta?.epsOrChapters && meta.epsOrChapters > 0 ? meta.epsOrChapters : 12;
        return Array.from({ length: total }, (_, i) => ({
            number: i + 1,
            title: null as string | null,
            thumbnail: '',
            duration: null as number | null
        }));
    });

    const resumeEpisode = $derived.by(() => {
        if (isMovie) return 1;
        const inProgress = animeProgress
            .filter((p) => !p.completed && p.timestampSeconds && p.timestampSeconds > 0)
            .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];
        if (inProgress) return inProgress.episode;

        const done = new Set(animeProgress.filter((p) => p.completed).map((p) => p.episode));
        return episodes.find((ep) => !done.has(ep.number))?.number ?? episodes[0]?.number ?? 1;
    });

    const resumeProgress = $derived(progressMap.get(resumeEpisode));
    const resumeInProgress = $derived(
        !!resumeProgress && !resumeProgress.completed && (resumeProgress.timestampSeconds ?? 0) > 0
    );

    function watchHref(ep: number, prog?: AnimeProgress) {
        const inProg = prog && !prog.completed && (prog.timestampSeconds ?? 0) > 0;
        return inProg ? `${WATCH}/${cid}/${ep}?t=${prog!.timestampSeconds}` : `${WATCH}/${cid}/${ep}`;
    }

    const arcs = $derived.by(() => {
        if (episodes.length <= ARC_SIZE) return null;
        const out: { start: number; end: number; label: string }[] = [];
        for (let i = 0; i < episodes.length; i += ARC_SIZE) {
            const slice = episodes.slice(i, i + ARC_SIZE);
            const start = slice[0].number;
            const end = slice[slice.length - 1].number;
            out.push({ start, end, label: `${start}–${end}` });
        }
        return out;
    });

    const resumeArcIndex = $derived.by(() => {
        if (!arcs) return 0;
        const idx = arcs.findIndex((a) => resumeEpisode >= a.start && resumeEpisode <= a.end);
        return idx >= 0 ? idx : 0;
    });

    let selectedArc = $state(0);
    $effect(() => {
        selectedArc = resumeArcIndex;
    });

    const visibleEpisodes = $derived.by(() => {
        if (!arcs) return episodes;
        const arc = arcs[selectedArc] ?? arcs[0];
        return episodes.filter((ep) => ep.number >= arc.start && ep.number <= arc.end);
    });

    const isRich = $derived(episodes.some((e) => e.title));

    // ---------------------------------------------------------------- list button
    const listEntry = $derived(listStore.entries.find((e) => e.cid === cid) ?? null);

    const STATUS_LABELS = $derived<Record<string, string>>({
        CURRENT: i18n.t('list.current'),
        COMPLETED: i18n.t('list.completed'),
        PLANNING: i18n.t('list.planning'),
        PAUSED: i18n.t('list.paused'),
        DROPPED: i18n.t('list.dropped')
    });

    const getMs = (ts: number) => (ts > 1e11 ? ts : ts * 1000);

    const airingEntry = $derived(
        cid ? scheduleStore.upcoming.find((e) => String(e.fullContent.content.cid) === cid) : undefined
    );

    const countdownText = $derived.by(() => {
        if (!airingEntry) return null;
        const diff = getMs(airingEntry.airingAt) - Date.now();
        if (diff <= 0) return null;
        const mins = Math.floor(diff / 60_000);
        const d = Math.floor(mins / 1440);
        const h = Math.floor((mins % 1440) / 60);
        const m = mins % 60;
        if (d > 0) return `${d}d ${h}h`;
        if (h > 0) return `${h}h ${m}m`;
        return `${m}m`;
    });

    // ---------------------------------------------------------------- misc helpers
    const releaseDate = $derived(
        meta?.releaseDate
            ? new Date(meta.releaseDate).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short' })
            : null
    );

    let synopsisEl = $state<HTMLElement | undefined>();
    let synopsisExpanded = $state(false);
    let canTruncate = $state(false);
    $effect(() => {
        meta?.synopsis;
        if (synopsisEl && !synopsisExpanded) {
            canTruncate = synopsisEl.scrollHeight > synopsisEl.clientHeight;
        }
    });

    const formatDuration = (minutes: number | null) => {
        if (!minutes || minutes <= 0) return '';
        if (minutes < 60) return `${minutes}m`;
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return m > 0 ? `${h}h ${m}m` : `${h}h`;
    };

    function formatRelationType(type: string) {
        if (!type) return '';
        const key = `relations.${type.toUpperCase()}` as any;
        const translated = i18n.t(key);
        if (translated === key) {
            return type.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, (l) => l.toUpperCase());
        }
        return translated;
    }

    // Content loads after navigation, so the layout's own autofocus has nothing to grab yet.
    // Take focus when the Play button appears, but never steal it from something the user picked.
    function takeFocus(node: HTMLElement) {
        const a = document.activeElement;
        if (!a || a === document.body || a.closest('[data-tv-sidebar]')) node.focus();
    }
</script>

<svelte:head>
    <title>{displayTitle || i18n.t(detail.isLoading ? 'content.loading' : 'content.details')}</title>
</svelte:head>

{#if detail.isLoading}
    <div class="flex h-[80vh] items-center justify-center">
        <Spinner class="size-12 text-muted-foreground/30" />
    </div>

{:else if detail.error}
    <div class="flex h-[80vh] flex-col items-center justify-center gap-6 text-center">
        <AlertCircle class="size-16 text-destructive/60" />
        <p class="text-2xl font-bold text-muted-foreground">{i18n.t(detail.error?.key || 'errors.error')}</p>
        <button
                type="button"
                data-sn
                use:takeFocus
                onclick={() => detail.retry()}
                class="flex items-center gap-2 rounded-lg border-2 border-border px-6 py-3 text-base font-bold outline-none
                       focus:border-primary focus:ring-4 focus:ring-primary/40"
        >
            <RefreshCw class="size-5" />
            {i18n.t('content.retry')}
        </button>
    </div>

{:else if full}
    <!-- data-sn-rows: row-based navigation (see spatialNav.ts), since this page mixes buttons,
         chips, an episode grid and a horizontal row. -->
    <div class="relative space-y-10 pb-16" data-sn-rows in:fade={{ duration: 200 }}>
        <!-- Backdrop. Starts at the content edge (not under the sidebar) and fades out. -->
        <div class="pointer-events-none absolute -top-8 left-0 -right-12 h-[420px] overflow-hidden">
            <SmartImage
                    src={meta?.bannerImage || meta?.coverImage}
                    alt=""
                    {shouldBlur}
                    imageHeaders={detail.headers}
                    class="size-full object-cover opacity-30"
            />
            <div class="absolute inset-0 bg-gradient-to-b from-background/10 via-background/60 to-background"></div>
            <div class="absolute inset-0 bg-gradient-to-r from-background via-transparent to-transparent"></div>
        </div>

        <!-- ------------------------------------------------------------ hero -->
        <section class="relative flex gap-8 pt-2" onfocusin={revealTop}>
            <div class="w-50 shrink-0">
                <div class="aspect-[2/3] overflow-hidden rounded-md bg-muted shadow-[0_12px_50px_rgba(0,0,0,0.7)] ring-1 ring-white/10">
                    <SmartImage
                            src={meta?.coverImage}
                            alt={displayTitle}
                            {shouldBlur}
                            imageHeaders={detail.headers}
                            class="size-full object-cover"
                    />
                </div>
            </div>

            <div class="min-w-0 flex-1 space-y-4">
                <h1 class="line-clamp-2 text-4xl font-black leading-tight tracking-tight drop-shadow-[0_2px_8px_rgba(0,0,0,0.5)]">
                    {displayTitle || 'Unknown title'}
                </h1>

                <div class="flex flex-wrap items-center gap-x-3 gap-y-2 text-sm font-semibold text-muted-foreground">
                    {#if score}
                        <span class="flex items-center gap-1.5 rounded-md border border-emerald-500/20 bg-emerald-500/10 px-2.5 py-1 font-bold text-emerald-400">
                            <Star class="size-4 fill-current" />{score}
                        </span>
                    {/if}
                    {#if airingEntry && countdownText}
                        <span class="flex items-center gap-1.5 rounded-md border border-primary/20 bg-primary/10 px-2.5 py-1 font-bold text-primary">
                            <Clock class="size-4" />
                            {i18n.t('content.episode_in', { num: airingEntry.episode, time: countdownText })}
                        </span>
                    {/if}
                    {#if releaseDate}
                        <span class="flex items-center gap-1.5 rounded-md bg-muted/40 px-2.5 py-1">
                            <Calendar class="size-4 text-primary/70" />{releaseDate}
                        </span>
                    {/if}
                    {#if meta?.subtype}
                        <span class="flex items-center gap-1.5 rounded-md bg-muted/40 px-2.5 py-1 uppercase tracking-wider">
                            <Tv class="size-4 text-primary/70" />{meta.subtype}
                        </span>
                    {/if}
                    {#if meta?.epsOrChapters > 1}
                        <span class="flex items-center gap-1.5 rounded-md bg-muted/40 px-2.5 py-1">
                            <BookMarked class="size-4 text-primary/70" />
                            {meta.epsOrChapters} {i18n.t('content.eps_short')}
                        </span>
                    {/if}
                    {#if meta?.studio}
                        <span class="flex items-center gap-1.5 text-foreground/80">
                            <Building2 class="size-4 text-muted-foreground/60" />{meta.studio}
                        </span>
                    {/if}
                </div>

                {#if meta?.genres?.length}
                    <div class="flex flex-wrap gap-2">
                        {#each meta.genres.slice(0, 5) as genre}
                            <span class="rounded-md border border-border/20 bg-muted/40 px-3 py-1 text-xs font-bold uppercase tracking-wide text-foreground/70">
                                {genre}
                            </span>
                        {/each}
                    </div>
                {/if}

                <!-- Actions -->
                <div class="flex flex-wrap items-center gap-3 py-1">
                    {#if isAnime}
                        <a
                                href={watchHref(resumeEpisode, resumeProgress)}
                                data-sn
                                use:takeFocus
                                class="flex h-14 scroll-my-24 items-center gap-3 rounded-lg bg-primary px-8 text-lg font-black text-primary-foreground outline-none
                                       transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/50 focus:ring-offset-2 focus:ring-offset-background"
                        >
                            <Play class="size-5 fill-current" />
                            {resumeInProgress ? i18n.t('content.resume') : i18n.t('content.watch_now')}
                            {#if !isMovie}
                                <span class="text-sm font-bold opacity-80">· {i18n.t('content.episode_title', { num: resumeEpisode })}</span>
                            {/if}
                        </a>
                    {/if}

                    <ListEditorButton
                            {cid}
                            title={displayTitle}
                            contentType={full.content.contentType}
                            coverImage={meta?.coverImage}
                            headers={detail.headers}
                            label={listEntry ? (STATUS_LABELS[listEntry.status] ?? listEntry.status) : i18n.t('list.planning')}
                    />
                </div>

                {#if meta?.synopsis}
                    <div class="mt-6 max-w-4xl space-y-1">
                        <p
                                bind:this={synopsisEl}
                                class="text-base leading-relaxed text-muted-foreground {synopsisExpanded ? '' : 'line-clamp-3'}"
                        >
                            {@html meta.synopsis.replace(/<[^>]*>?/gm, '')}
                        </p>
                        {#if canTruncate || synopsisExpanded}
                            <button
                                    type="button"
                                    data-sn
                                    onclick={() => (synopsisExpanded = !synopsisExpanded)}
                                    class="flex h-10 scroll-my-24 items-center rounded-lg px-4 text-primary outline-none focus:bg-secondary focus:ring-4 focus:ring-primary"
                            >
                                {#if synopsisExpanded}<ChevronUp class="size-5" />{:else}<ChevronDown class="size-5" />{/if}
                            </button>
                        {/if}
                    </div>
                {/if}
            </div>
        </section>

        <!-- ------------------------------------------------------------ episodes -->
        {#if isAnime && !isMovie}
            <section class="relative space-y-4">
                <h2 class="text-2xl font-black tracking-tight">
                    {i18n.t('content.episodes_title', { default: 'Episodes' })}
                    {#if meta?.epsOrChapters}
                        <span class="font-medium text-muted-foreground/50">· {meta.epsOrChapters}</span>
                    {/if}
                </h2>

                {#if arcs}
                    <!-- One scrollable line; focus scrolls it sideways -->
                    <div class="-mx-3 flex gap-2 overflow-x-auto px-3 py-3 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                        {#each arcs as arc, i (arc.start)}
                            <button
                                    type="button"
                                    data-sn
                                    aria-pressed={selectedArc === i}
                                    onclick={() => (selectedArc = i)}
                                    class="h-11 shrink-0 scroll-my-24 rounded-lg border-2 px-4 text-sm font-bold tabular-nums outline-none
                                           transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                                           {selectedArc === i
                                    ? 'border-primary bg-primary text-primary-foreground'
                                    : 'border-border/60 bg-card text-foreground'}"
                            >
                                {arc.label}
                            </button>
                        {/each}
                    </div>
                {/if}

                <div class="grid grid-cols-4 gap-x-5 gap-y-7 p-3">
                    {#each visibleEpisodes as ep (ep.number)}
                        {@const prog = progressMap.get(ep.number)}
                        {@const isCompleted = prog?.completed ?? false}
                        {@const isInProgress = !isCompleted && (prog?.timestampSeconds ?? 0) > 0}
                        {@const isResume = ep.number === resumeEpisode}
                        {@const pct =
                            isInProgress && prog?.episodeDurationSeconds && prog.episodeDurationSeconds > 0
                                ? Math.min(100, Math.round((prog.timestampSeconds! / prog.episodeDurationSeconds) * 100))
                                : null}

                        <a href={watchHref(ep.number, prog)} data-sn class="group block scroll-my-24 outline-none">
                            <div
                                    class="relative aspect-video overflow-hidden rounded-md bg-muted/20 transition-transform duration-150
                                           group-focus:scale-105 group-focus:ring-4 group-focus:ring-primary
                                           {isResume ? 'ring-2 ring-primary/50' : ''}"
                            >
                                {#if ep.thumbnail}
                                    <img
                                            src={ep.thumbnail}
                                            alt={ep.title ?? `EP ${ep.number}`}
                                            loading="lazy"
                                            class="size-full object-cover {isCompleted ? 'opacity-40' : ''}"
                                    />
                                {:else}
                                    <div class="flex size-full items-center justify-center bg-muted/10">
                                        <span class="text-4xl font-black text-white/10">{ep.number}</span>
                                    </div>
                                {/if}

                                {#if ep.duration}
                                    <div class="absolute left-2 top-1.5 text-xs font-bold text-white [text-shadow:0_1px_3px_rgba(0,0,0,0.9)]">
                                        {formatDuration(ep.duration)}
                                    </div>
                                {/if}

                                {#if isCompleted}
                                    <div class="absolute inset-0 flex items-center justify-center">
                                        <CheckCircle2 class="size-9 text-primary/80" />
                                    </div>
                                {/if}

                                {#if pct !== null}
                                    <div class="absolute inset-x-0 bottom-0 h-1.5 bg-white/20">
                                        <div class="h-full bg-primary" style="width: {pct}%"></div>
                                    </div>
                                {/if}
                            </div>

                            <div class="space-y-0.5 pt-2">
                                <p class="text-xs font-bold uppercase tracking-wide text-muted-foreground/70">
                                    {i18n.t('content.episode_title', { num: ep.number })}
                                    {#if isResume && !isCompleted}
                                        <span class="text-primary">· {isInProgress ? i18n.t('content.resume') : i18n.t('home.hero.watch')}</span>
                                    {/if}
                                </p>
                                {#if isRich && ep.title}
                                    <p class="line-clamp-2 text-sm font-semibold leading-snug group-focus:text-primary">{ep.title}</p>
                                {/if}
                            </div>
                        </a>
                    {/each}
                </div>
            </section>
        {/if}

        {#if relations.length > 0}
            <section class="relative space-y-3 border-t border-border/20 pt-6">
                <h2 class="text-2xl font-black tracking-tight">{i18n.t('content.related')}</h2>

                <div class="-mx-3 flex gap-5 overflow-x-auto px-3 py-3 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                    {#each relations.slice(0, 30) as relation (relation.targetCid ?? `${relation.targetTrackerName}:${relation.targetTrackerId}`)}
                        <TvCard card={relation.card}>
                            {#snippet overlay()}
                                <span class="block truncate text-[10px] font-black uppercase tracking-wider text-white">
                                    {formatRelationType(relation.relationType)}
                                </span>
                            {/snippet}
                        </TvCard>
                    {/each}
                </div>
            </section>
        {/if}
    </div>
{/if}