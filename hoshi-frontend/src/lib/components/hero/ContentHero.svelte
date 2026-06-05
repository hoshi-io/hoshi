<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { appConfig } from "@/stores/config.svelte";
    import { Button } from "@/components/ui/button";
    import { Star, Calendar, Tv, BookMarked, Building2, Play, BookOpen, Link, Plug } from "lucide-svelte";
    import ListEditorButton from "@/components/ListEditorButton.svelte";
    import SmartImage from "@/components/SmartImage.svelte";

    let {
        fullContent,
        meta,
        displayTitle,
        isAnime,
        showTrackerModal = $bindable(),
        showExtensionModal = $bindable(),
        onWatchNow,
        headers
    } = $props<{
        fullContent: any;
        meta: any;
        displayTitle: string;
        isAnime: boolean;
        showTrackerModal: boolean;
        showExtensionModal: boolean;
        onWatchNow: () => void;
        headers?: any
    }>();

    let trackersExpanded = $state(false);
    const TRACKERS_LIMIT = 50;

    const score = $derived(meta?.rating ? Math.round(meta.rating * 10) : null);
    const isAdultContent = $derived(fullContent.content.nsfw || meta?.genres?.some((g: string) => ['hentai', 'adult'].includes(g.toLowerCase())));
    const shouldBlur = $derived(isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true));

    const trackers = $derived(fullContent.trackerMappings ?? []);
    const visibleTrackers = $derived(trackersExpanded ? trackers : trackers.slice(0, TRACKERS_LIMIT));

    function formatDate(dateStr?: string | null) {
        if (!dateStr) return null;
        return new Date(dateStr).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short' });
    }

    function getTrackerFavicon(name: string) {
        const domains: Record<string, string> = {
            anilist: 'anilist.co', myanimelist: 'myanimelist.net', mal: 'myanimelist.net',
            kitsu: 'kitsu.io', simkl: 'simkl.com', trakt: 'trakt.tv',
            anidb: 'anidb.net', animeplanet: 'anime-planet.com', shikimori: 'shikimori.one'
        };
        const domain = domains[name.toLowerCase()] || `${name}.com`;
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    function getTrackerLink(tracker: any) {
        if (tracker.trackerUrl) return tracker.trackerUrl;
        const id = tracker.trackerId;
        switch (tracker.trackerName.toLowerCase()) {
            case 'anilist': return `https://anilist.co/anime/${id}`;
            case 'anidb': return `https://anidb.net/anime/${id}`;
            case 'animeplanet': return `https://www.anime-planet.com/anime/${id}`;
            case 'myanimelist': return `https://myanimelist.net/anime/${id}`;
            default: return '#';
        }
    }
</script>

<div class="absolute top-0 inset-x-0 h-[62vh] md:h-[72vh] overflow-hidden pointer-events-none" in:fade={{ duration: 1000 }}>
    <SmartImage
            src={meta?.bannerImage || meta?.coverImage}
            alt={""}
            {shouldBlur}
            imageHeaders={headers}
            class="w-full h-full object-cover opacity-45"
    />
    <div class="absolute inset-0 bg-gradient-to-b from-transparent via-background/30 to-background"></div>
    <div class="absolute top-0 inset-x-0 h-24 bg-gradient-to-b from-background/60 to-transparent"></div>
</div>

<div class="relative z-10 w-full max-w-[2000px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 pt-56 md:pt-72 lg:pt-80" in:fade={{ delay: 100, duration: 400 }}>
    <div class="flex gap-4 sm:gap-6 md:gap-10 items-start">

        <div class="shrink-0 w-32 sm:w-40 md:w-48 lg:w-56" in:fly={{ y: 20, duration: 500, delay: 150 }}>
            <div class="rounded-sm overflow-hidden shadow-[0_8px_40px_rgba(0,0,0,0.6)] bg-muted aspect-[2/3] ring-1 ring-white/10">

                <SmartImage
                        src={meta?.coverImage}
                        alt={displayTitle}
                        {shouldBlur}
                        imageHeaders={headers}
                        class="w-full h-full object-cover"
                />
            </div>
        </div>

        <div class="flex-1 min-w-0 pb-2 md:pb-0 space-y-4 md:space-y-5" in:fly={{ y: 20, duration: 500, delay: 200 }}>
            <h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-black leading-tight tracking-tight line-clamp-3">
                {displayTitle}
            </h1>

            <div class="flex flex-wrap items-center gap-x-3 sm:gap-x-4 gap-y-1.5 text-[11px] sm:text-xs text-muted-foreground font-medium">
                {#if score}
        <span class="flex items-center gap-1 bg-green-500/15 text-green-400 border border-green-500/25 px-2 sm:px-2.5 py-1 rounded-lg font-bold">
            <Star class="w-2.5 h-2.5 sm:w-3 sm:h-3 fill-current" />{score}%
        </span>
                {/if}
                {#if meta?.releaseDate}
        <span class="flex items-center gap-1">
            <Calendar class="w-2.5 h-2.5 sm:w-3 sm:h-3 opacity-60" />{formatDate(meta.releaseDate)}
        </span>
                {/if}
                {#if meta?.subtype}
        <span class="flex items-center gap-1">
            <Tv class="w-2.5 h-2.5 sm:w-3 sm:h-3 opacity-60" />{meta.subtype}
        </span>
                {/if}
                {#if meta?.epsOrChapters}
        <span class="flex items-center gap-1">
            <BookMarked class="w-2.5 h-2.5 sm:w-3 sm:h-3 opacity-60" />
            {meta.epsOrChapters} {isAnime ? i18n.t('content.eps_short') : i18n.t('content.ch_short')}
        </span>
                {/if}
                {#if meta?.studio}
        <span class="flex items-center gap-1">
            <Building2 class="w-2.5 h-2.5 sm:w-3 sm:h-3 opacity-60" />{meta.studio}
        </span>
                {/if}
            </div>

            {#if meta?.genres?.length}
                <div class="hidden sm:flex flex-wrap gap-1.5">
                    {#each meta.genres.slice(0, 6) as genre}
            <span class="text-[11px] font-semibold px-2.5 py-0.5 rounded-full bg-muted/50 text-muted-foreground border border-border/30">
                {genre}
            </span>
                    {/each}
                </div>
            {/if}

            <div class="flex items-center gap-2 flex-wrap pt-1">
                <Button onclick={onWatchNow} class="rounded-sm px-6 h-10 font-bold shadow-lg gap-2">
                    {#if isAnime}
                        <Play class="w-4 h-4 fill-current" />{i18n.t('content.watch_now')}
                    {:else}
                        <BookOpen class="w-4 h-4 fill-current" />{i18n.t('content.read_now')}
                    {/if}
                </Button>

                <div class="hidden sm:block">
                    <ListEditorButton
                            cid={fullContent.content.cid}
                            title={displayTitle}
                            contentType={fullContent.content.contentType}
                            coverImage={meta?.coverImage}
                            size="icon"
                            class="h-10 w-10"
                    />
                </div>

                <Button size="icon" variant="secondary" class="rounded-sm w-10 h-10" onclick={() => showTrackerModal = true}>
                    <Link class="w-4 h-4" />
                </Button>
                <Button size="icon" variant="secondary" class="rounded-sm w-10 h-10" onclick={() => showExtensionModal = true}>
                    <Plug class="w-4 h-4" />
                </Button>
            </div>

            {#if trackers.length > 0}
                <div class="flex flex-wrap items-center gap-1 sm:gap-1.5 pt-0.5">
                    {#each visibleTrackers as tracker}
                        <a
                                href={getTrackerLink(tracker)}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="group flex items-center gap-0 sm:gap-1.5 px-1.5 sm:px-2.5 py-1 rounded-full bg-muted/20 border border-border/20 hover:bg-muted/40 transition-all"
                        >
                            <img
                                    src={getTrackerFavicon(tracker.trackerName)}
                                    alt={tracker.trackerName}
                                    class="w-3.5 h-3.5 sm:w-3 sm:h-3 rounded-sm opacity-70 group-hover:opacity-100 transition-opacity"
                            />
                            <span class="hidden sm:inline text-[10px] font-mono text-muted-foreground/35">
                    #{tracker.trackerId}
                </span>
                        </a>
                    {/each}

                    {#if trackers.length > TRACKERS_LIMIT}
                        <button
                                onclick={() => trackersExpanded = !trackersExpanded}
                                class="px-2 sm:px-2.5 py-1 rounded-full bg-muted/15 border border-border/15 text-[10px] sm:text-[11px] font-medium text-muted-foreground/50"
                        >
                            {trackersExpanded ? '−' : `+${trackers.length - TRACKERS_LIMIT}`}
                        </button>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
</div>