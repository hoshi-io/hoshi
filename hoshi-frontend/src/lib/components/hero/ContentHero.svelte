<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { appConfig } from "@/stores/config.svelte";
    import { Button } from "@/components/ui/button";
    import { Star, Calendar, Tv, BookMarked, Building2, Play, BookOpen, Link, Plug, ChevronDown, ChevronUp, Clock } from "lucide-svelte";
    import ListEditorButton from "@/components/ListEditorButton.svelte";
    import SmartImage from "@/components/SmartImage.svelte";
    import { formatScore } from "@/utils/normalize";
    import RelationTreeDialog from "@/components/content/RelationTreeDialog.svelte";
    import { scheduleStore } from "@/app/schedule.svelte.js";

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
    const TRACKERS_LIMIT = 6;

    let synopsisExpanded = $state(false);
    let synopsisElement = $state<HTMLElement | undefined>();
    let canTruncateSynopsis = $state(false);

    $effect(() => {
        if (synopsisElement && !synopsisExpanded) {
            canTruncateSynopsis = synopsisElement.scrollHeight > synopsisElement.clientHeight;
        }
    });

    const score = $derived(meta?.rating ? formatScore(meta.rating) : null);
    const isAdultContent = $derived(fullContent.content.nsfw || meta?.genres?.some((g: string) => ['hentai', 'adult'].includes(g.toLowerCase())));
    const shouldBlur = $derived(isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true));

    const trackers = $derived(fullContent.trackerMappings ?? []);
    const visibleTrackers = $derived(trackersExpanded ? trackers : trackers.slice(0, TRACKERS_LIMIT));
    const cid = $derived(fullContent.content.cid)

    function getAiringMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }

    const airingEntry = $derived(
        scheduleStore.entries.find(e => String(e.trackerId) === fullContent.trackerMappings.find(e => e.trackerName === "anilist")?.trackerId)
    );

    const countdownText = $derived.by(() => {
        if (!airingEntry) return null;
        const diffMs = getAiringMs(airingEntry.airingAt) - Date.now();
        if (diffMs <= 0) return null;

        const totalMinutes = Math.floor(diffMs / 60_000);
        const days    = Math.floor(totalMinutes / 1440);
        const hours   = Math.floor((totalMinutes % 1440) / 60);
        const minutes = totalMinutes % 60;

        if (days > 0)  return `${days}d ${hours}h`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        return `${minutes}m`;
    });

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
        const mediaType = isAnime ? 'anime' : 'manga';

        switch (tracker.trackerName.toLowerCase()) {
            case 'anilist': return `https://anilist.co/${mediaType}/${id}`;
            case 'anidb': return `https://anidb.net/${mediaType}/${id}`;
            case 'animeplanet':
            case 'anime-planet': return `https://www.anime-planet.com/${mediaType}/${id}`;
            case 'myanimelist':
            case 'mal': return `https://myanimelist.net/${id.replace(':', '/')}`;
            case 'simkl': return `https://simkl.com/${mediaType}/${id}`;
            case 'kitsu': return `https://kitsu.io/${mediaType}/${id}`;
            case 'shikimori': return `https://shikimori.one/${mediaType}s/${id}`;
            default: return '#';
        }
    }
</script>

<div class="absolute top-0 inset-x-0 h-[55vh] md:h-[65vh] overflow-hidden pointer-events-none" in:fade={{ duration: 1000 }}>
    <SmartImage
            src={meta?.bannerImage || meta?.coverImage}
            alt={""}
            {shouldBlur}
            imageHeaders={headers}
            class="w-full h-full object-cover opacity-35 dark:opacity-30 scale-[1.01]"
    />
    <div class="absolute inset-0 bg-gradient-to-b from-background/5 via-background/40 to-background"></div>
    <div class="absolute top-0 inset-x-0 h-32 bg-gradient-to-b from-background/70 to-transparent"></div>
</div>

<div class="relative z-10 w-full max-w-[1800px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 pt-40 md:pt-48 lg:pt-52" in:fade={{ delay: 100, duration: 400 }}>
    <div class="flex flex-col sm:flex-row gap-6 md:gap-10 items-center sm:items-start text-center sm:text-left">

        <div class="shrink-0 w-36 sm:w-40 md:w-48 lg:w-52" in:fly={{ y: 25, duration: 500, delay: 150 }}>
            <div class="rounded-sm overflow-hidden shadow-[0_12px_50px_rgba(0,0,0,0.7)] bg-muted aspect-[2/3] ring-1 ring-white/10 group relative">
                <SmartImage
                        src={meta?.coverImage}
                        alt={displayTitle}
                        {shouldBlur}
                        imageHeaders={headers}
                        class="w-full h-full object-cover transition-transform duration-500 ease-out group-hover:scale-105 will-change-transform"
                />
            </div>
        </div>

        <div class="flex-1 min-w-0 space-y-4" in:fly={{ y: 20, duration: 500, delay: 200 }}>

            <h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-black leading-tight tracking-tight drop-shadow-[0_2px_8px_rgba(0,0,0,0.5)] line-clamp-2">
                {displayTitle || "Unknown title"}
            </h1>

            <div class="flex flex-wrap items-center justify-center sm:justify-start gap-x-4 gap-y-2 text-xs text-muted-foreground font-semibold">
                {#if score}
                    <span class="flex items-center gap-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded-md font-bold shadow-sm">
                        <Star class="w-3 h-3 fill-current" /> {score}
                    </span>
                {/if}
                {#if airingEntry && countdownText}
                    <span class="flex items-center gap-1 bg-primary/10 text-primary border border-primary/20 px-2 py-0.5 rounded-sm font-bold shadow-sm">
                        <Clock class="w-3 h-3" /> {i18n.t("content.episode_in", { num: airingEntry.episode, time: countdownText})}
                    </span>
                {/if}
                {#if meta?.releaseDate}
                    <span class="flex items-center gap-1.5 bg-muted/30 px-2 py-0.5 rounded-md border border-border/10">
                        <Calendar class="w-3 h-3 text-primary/70" />{formatDate(meta.releaseDate)}
                    </span>
                {/if}
                {#if meta?.subtype}
                    <span class="flex items-center gap-1.5 bg-muted/30 px-2 py-0.5 rounded-md border border-border/10 uppercase text-[10px] tracking-wider">
                        <Tv class="w-3 h-3 text-primary/70" />{meta.subtype}
                    </span>
                {/if}
                {#if meta?.epsOrChapters > 1}
                    <span class="flex items-center gap-1.5 bg-muted/30 px-2 py-0.5 rounded-md border border-border/10">
                        <BookMarked class="w-3 h-3 text-primary/70" />
                        {meta.epsOrChapters} {isAnime ? i18n.t('content.eps_short') : i18n.t('content.ch_short')}
                    </span>
                {/if}
                {#if meta?.studio}
                    <span class="flex items-center gap-1.5 text-foreground/80">
                        <Building2 class="w-3 h-3 text-muted-foreground/60" />{meta.studio}
                    </span>
                {/if}
            </div>

            {#if meta?.genres?.length}
                <div class="flex flex-wrap justify-center sm:justify-start gap-1.5">
                    {#each meta.genres.slice(0, 5) as genre}
                        <span class="text-[10px] font-bold tracking-wide uppercase px-2.5 py-0.5 rounded-md bg-muted/40 text-foreground/70 border border-border/20 backdrop-blur-sm">
                            {genre}
                        </span>
                    {/each}
                </div>
            {/if}

            <div class="flex flex-wrap items-center justify-center sm:justify-start gap-2 pt-2">
                <Button onclick={onWatchNow} class="rounded-sm px-6 h-10 font-bold shadow-lg gap-2">
                    {#if isAnime}
                        <Play class="w-4 h-4 fill-current" />{i18n.t('content.watch_now')}
                    {:else}
                        <BookOpen class="w-4 h-4 fill-current" />{i18n.t('content.read_now')}
                    {/if}
                </Button>
                <div class="flex items-center bg-muted/20 p-1 rounded-md border border-border/10 backdrop-blur-md gap-1">
                    <ListEditorButton
                            cid={fullContent.content.cid}
                            title={displayTitle}
                            contentType={fullContent.content.contentType}
                            coverImage={meta?.coverImage}
                            size="icon"
                            class="h-8 w-8 rounded-sm bg-transparent border-0 hover:bg-muted/40 text-foreground"
                    />

                    <div class="w-[1px] h-4 bg-border/40 mx-0.5"></div>

                    <Button size="icon" variant="ghost" class="rounded-sm w-8 h-8 hover:bg-muted/40 text-muted-foreground hover:text-foreground" onclick={() => showTrackerModal = true} title="Trackers">
                        <Link class="w-4 h-4" />
                    </Button>
                    <Button size="icon" variant="ghost" class="rounded-sm w-8 h-8 hover:bg-muted/40 text-muted-foreground hover:text-foreground" onclick={() => showExtensionModal = true} title="Extensions">
                        <Plug class="w-4 h-4" />
                    </Button>
                    <RelationTreeDialog cid={cid} />
                </div>

                {#if trackers.length > 0}
                    <div class="flex items-center gap-1.5 ml-0 sm:ml-3 bg-muted/10 px-2 py-1 rounded-md border border-border/5">
                        {#each visibleTrackers as tracker}
                            <a
                                    href={getTrackerLink(tracker)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="group flex items-center justify-center w-6 h-6 rounded-md hover:bg-muted/50 transition-all"
                                    title={`${tracker.trackerName}: #${tracker.trackerId}`}
                            >
                                <img
                                        src={getTrackerFavicon(tracker.trackerName)}
                                        alt={tracker.trackerName}
                                        class="w-4 h-4 rounded-sm opacity-60 group-hover:opacity-100 transition-opacity"
                                />
                            </a>
                        {/each}

                        {#if trackers.length > TRACKERS_LIMIT}
                            <button
                                    onclick={() => trackersExpanded = !trackersExpanded}
                                    class="w-6 h-6 flex items-center justify-center rounded-md hover:bg-muted/50 text-[10px] font-bold text-muted-foreground/60"
                            >
                                {trackersExpanded ? '−' : `+`}
                            </button>
                        {/if}
                    </div>
                {/if}
            </div>

            {#if meta?.synopsis}
                <div class="max-w-7xl space-y-1.5 mx-auto sm:mx-0 text-justify">
                    <p
                            bind:this={synopsisElement}
                            class="text-sm text-muted-foreground leading-relaxed {synopsisExpanded ? '' : 'line-clamp-3'}"
                    >
                        {@html meta.synopsis.replace(/<[^>]*>?/gm, '')}
                    </p>

                    {#if canTruncateSynopsis || synopsisExpanded}
                        <button
                                onclick={() => synopsisExpanded = !synopsisExpanded}
                                class="flex items-center gap-1 text-xs font-bold text-primary/80 hover:text-primary transition-colors mx-auto sm:mx-0"
                        >
                            {#if synopsisExpanded}
                                <ChevronUp class="w-3 h-3" />
                            {:else}
                                <ChevronDown class="w-3 h-3" />
                            {/if}
                        </button>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
</div>