<script lang="ts">
    import { fade } from 'svelte/transition';
    import {
        AlertCircle, BookMarked, Building2, Calendar, ChevronDown,
        ChevronUp, Clock, Play, RefreshCw, Star, Tv
    } from 'lucide-svelte';

    import { ContentDetailState } from '@/app/content.svelte';
    import { listStore } from '@/app/list.svelte.js';
    import { scheduleStore } from '@/app/schedule.svelte.js';
    import { progressApi } from '@/api/progress/progress';
    import type { AnimeProgress } from '@/api/progress/types';
    import { primaryMetadata } from '@/api/content/types';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { appConfig } from '@/stores/config.svelte';
    import { formatScore } from '@/utils/normalize';
    import { Spinner } from '@/components/ui/spinner';
    import SmartImage from '@/components/SmartImage.svelte';
    import TvCard from '@/tv/components/card/TvCard.svelte';
    import TvEpisodeCard from '@/tv/components/card/TvEpisodeCard.svelte';
    import ListEditorButton from "@/components/ListEditorButton.svelte";
    import {isInProgress, useEpisodeList} from "@/utils/episode-grid.svelte.js";

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

    const list = useEpisodeList({
        cid: () => cid,
        units: () => full?.contentUnits,
        total: () => meta?.epsOrChapters,
        duration: () => meta?.episodeDuration,
        progress: () => animeProgress,
        enabled: () => !!full && isAnime && !isMovie,
        arcSize: 24, // bigger than the web's 12: fewer chips to scroll through on a remote
        watchBase: '/tv'
    });

    const resumeEpisode = $derived(isMovie ? 1 : list.resumeEpisode);
    const resumeProgress = $derived(list.progressFor(resumeEpisode));
    const resumeInProgress = $derived(isInProgress(resumeProgress));

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
                                href={list.watchHref(resumeEpisode, resumeProgress)}
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
                    {i18n.t('content.episodes_title')}
                    {#if meta?.epsOrChapters}
                        <span class="font-medium text-muted-foreground/50">· {meta.epsOrChapters}</span>
                    {/if}
                </h2>

                {#if list.arcs}
                    <!-- One scrollable line; focus scrolls it sideways -->
                    <div class="-mx-3 flex gap-2 overflow-x-auto px-3 py-3 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                        {#each list.arcs as arc, i (arc.start)}
                            <button
                                    type="button"
                                    data-sn
                                    aria-pressed={list.selectedArc === i}
                                    onclick={() => (list.selectedArc = i)}
                                    class="h-11 shrink-0 scroll-my-24 rounded-lg border-2 px-4 text-sm font-bold tabular-nums outline-none
                                           transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                                           {list.selectedArc === i
                                    ? 'border-primary bg-primary text-primary-foreground'
                                    : 'border-border/60 bg-card text-foreground'}"
                            >
                                {arc.label}
                            </button>
                        {/each}
                    </div>
                {/if}

                <div class="grid grid-cols-4 gap-x-5 gap-y-7 p-3">
                    {#each list.visible as row (row.number)}
                        <TvEpisodeCard
                                stacked
                                nativeImg
                                href={row.href}
                                image={row.thumbnail}
                                alt={row.title ?? row.label}
                                episode={row.label}
                                episodeTitle={row.title}
                                tag={row.tag}
                                badge={row.duration}
                                progress={row.pct ?? 0}
                                completed={row.completed}
                                highlight={row.isResume}
                                placeholder={String(row.number)}
                        />
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