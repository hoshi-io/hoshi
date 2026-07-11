<script lang="ts">
    import { fade } from "svelte/transition";

    import { i18n } from "@/stores/i18n.svelte.js";
    import { primaryMetadata } from "@/api/content/types";
    import Episodes from "@/components/content/Episodes.svelte";
    import Chapters from "@/components/content/Chapters.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import Relations from "@/components/content/Relations.svelte";
    import TrackerManager from "@/components/modals/TrackerManager.svelte";
    import ExtensionManager from '@/components/modals/ExtensionManager.svelte';
    import { appConfig } from "@/stores/config.svelte";

    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { AlertCircle } from "lucide-svelte";

    import { ContentDetailState } from "@/app/content.svelte";
    import { layoutState } from "@/stores/layout.svelte";
    import ContentHero from "@/components/hero/ContentHero.svelte";
    import ListEditorButton from "@/components/ListEditorButton.svelte";
    import { progressApi } from "@/api/progress/progress";
    import type { AnimeProgress, ChapterProgress } from "@/api/progress/types";
    import MpvLauncher from "@/components/mpv/MpvLauncher.svelte";
    import MergeContent from "@/components/modals/MergeContent.svelte";

    let mpvOpen = $state(false);
    let mpvEpNumber = $state(1);
    let mpvEpTitle = $state("");

    function handleWatchNowClick() {
        if (appConfig.data?.mpv?.useMpv){
            if (!detail.fullContent) return;

            const inProgress = animeProgress
                .filter(p => !p.completed && (p.timestampSeconds ?? 0) > 0)
                .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];

            if (inProgress) {
                mpvEpNumber = inProgress.episode;
            } else {
                const completedNums = new Set(animeProgress.filter(p => p.completed).map(p => p.episode));
                mpvEpNumber = animeProgress.length > 0
                    ? (animeProgress.filter(p => !completedNums.has(p.episode)).sort((a, b) => a.episode - b.episode)[0]?.episode ?? 1)
                    : 1;
            }

            const matchingUnit = detail.fullContent.contentUnits?.find(
                u => u.contentType === 'episode' && u.unitNumber === mpvEpNumber
            );

            mpvEpTitle = matchingUnit?.title
                ? i18n.t('watch.episode_with_title', { num: mpvEpNumber, title: matchingUnit.title })
                : i18n.t('watch.episode_number', { num: mpvEpNumber });

            mpvOpen = true;
        } else {
            return detail.watchNow();
        }
    }

    const detail = new ContentDetailState();

    let showTrackerModal = $state(false);
    let showExtensionModal = $state(false);
    let showMergeModal = $state(false);

    let animeProgress = $state<AnimeProgress[]>([]);
    let chapterProgress = $state<ChapterProgress[]>([]);
    let progressLoaded = $state(false);

    $effect(() => {
        layoutState.showBack = true;
        layoutState.backUrl = "/";
        layoutState.headerAction = headerAction;
    });

    $effect(() => {
        const cid = detail.fullContent?.content.cid;
        if (!cid || progressLoaded) return;

        progressLoaded = true;
        progressApi.getContentProgress(cid).then(res => {
            animeProgress = res.animeProgress ?? [];
            chapterProgress = res.chapterProgress ?? [];
        }).catch(() => {
            // Progress failing is non-fatal — degrade silently
        });
    });

    // Compute the Watch Now / Resume URL based on progress
    const watchUrl = $derived.by(() => {
        const cid = detail.fullContent?.content.cid;
        if (!cid) return null;

        // Find the most recently accessed in-progress episode
        const inProgress = animeProgress
            .filter(p => !p.completed && (p.timestampSeconds ?? 0) > 0)
            .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];

        if (inProgress) {
            return `/watch/${cid}/${inProgress.episode}?t=${inProgress.timestampSeconds}`;
        }

        // Otherwise first unwatched episode
        const completedNums = new Set(animeProgress.filter(p => p.completed).map(p => p.episode));
        const firstEp = animeProgress.length > 0
            ? (animeProgress.filter(p => !completedNums.has(p.episode)).sort((a, b) => a.episode - b.episode)[0]?.episode ?? 1)
            : 1;

        return `/watch/${cid}/${firstEp}`;
    });
</script>

{#snippet headerAction()}
    {#if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}

        <div class="sm:hidden">
            <ListEditorButton
                    cid={detail.fullContent.content.cid}
                    title={displayTitle}
                    contentType={detail.fullContent.content.contentType}
                    coverImage={meta?.coverImage}
                    headers={detail.headers}
                    size="icon"
                    variant="ghost"
            />
        </div>
    {/if}
{/snippet}

<svelte:head>
    {#if detail.isLoading}
        <title>{i18n.t('content.loading')}</title>
    {:else if detail.error}
        <title>Error</title>
    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const title = meta?.titleI18n?.[pref] || meta?.title || i18n.t('content.details')}
        <title>{title}</title>
    {/if}
</svelte:head>

<div class="min-h-screen bg-background">
    {#if detail.isLoading}
        <div class="flex h-[85vh] w-full items-center justify-center">
            <Spinner class="w-10 h-10 text-muted-foreground/20" />
        </div>

    {:else if detail.error}
        <div class="flex h-[85vh] flex-col items-center justify-center gap-4">
            <AlertCircle class="w-12 h-12 text-destructive opacity-20" />
            <p class="text-lg text-muted-foreground font-medium">{i18n.t(detail.error?.key || 'errors.error')}</p>
            <Button variant="outline" class="rounded-full" onclick={() => detail.retry()}>
                {i18n.t('content.retry')}
            </Button>
        </div>

    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const isMovie = meta?.subtype === 'MOVIE'}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}
        {@const isAdultContent = detail.fullContent.content.nsfw || meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase()))}
        {@const isAnime = detail.fullContent.content.contentType === 'anime'}
        {@const hasCastOrStaff = (meta?.characters?.length ?? 0) > 0 || (meta?.staff?.length ?? 0) > 0}
        {@const hasRelations = detail.relations.length > 0 || detail.relationsLoading}

        <ContentHero
                fullContent={detail.fullContent}
                {meta}
                {displayTitle}
                {isAnime}
                bind:showTrackerModal
                bind:showExtensionModal
                bind:showMergeModal
                watchUrl={isAnime ? watchUrl : null}
                onWatchNow={handleWatchNowClick}
                headers={detail.headers}
        />

        <div class="relative z-10 w-full max-w-[1800px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 mt-10 pb-24 space-y-10" in:fade={{ delay: 250, duration: 400 }}>

            {#if !isMovie}
                <section class="space-y-4">
                    {#if isAnime}
                        <Episodes
                                cid={detail.fullContent.content.cid}
                                epsOrChapters={meta?.epsOrChapters}
                                contentUnits={detail.fullContent.contentUnits}
                                duration={meta?.episodeDuration}
                                progress={animeProgress}
                                animeTitle={meta?.titleI18n?.[pref] || meta?.title || ''}
                                isNsfw={detail.fullContent.content.nsfw || meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase()))}
                                coverImage={meta?.coverImage}
                        />
                    {:else}
                        <Chapters
                                cid={detail.fullContent.content.cid}
                                contentType={detail.fullContent.content.contentType}
                                progress={chapterProgress}
                                isDerived={detail.isDerived}
                                source={detail.source || detail.fullContent.metadata[0].sourceName}
                        />
                    {/if}
                </section>
            {/if}

            {#if hasCastOrStaff}
                <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
            {/if}

            {#if hasRelations}
                <div class="pt-2 border-t border-border/20">
                    <Relations relations={detail.relations} loading={detail.relationsLoading} />
                </div>
            {/if}

        </div>

        <TrackerManager bind:open={showTrackerModal} cid={detail.fullContent.content.cid} trackers={detail.fullContent.trackerMappings} metadata={meta} />
        <ExtensionManager bind:open={showExtensionModal} cid={detail.fullContent.content.cid} metadata={meta} isNsfw={isAdultContent} extensions={detail.fullContent.extensionSources} contentType={detail.fullContent.content.contentType} />
        {#if mpvOpen && isAnime}
            <MpvLauncher
                    cid={detail.fullContent.content.cid}
                    epNumber={mpvEpNumber}
                    epTitle={mpvEpTitle}
                    animeTitle={meta?.titleI18n?.[pref] || meta?.title || ''}
                    totalEpisodes={meta?.epsOrChapters ?? 0}
                    isNsfw={isAdultContent}
                    coverImage={meta?.coverImage}
                    startTime={animeProgress.find(p => p.episode === mpvEpNumber)?.timestampSeconds ?? 0}
                    bind:open={mpvOpen}
            />
        {/if}
        <MergeContent
                bind:open={showMergeModal}
                cid={detail.fullContent.content.cid}
                displayTitle={displayTitle}
                coverImage={meta?.coverImage}
                contentType={detail.fullContent.content.contentType}
        />
    {/if}
</div>