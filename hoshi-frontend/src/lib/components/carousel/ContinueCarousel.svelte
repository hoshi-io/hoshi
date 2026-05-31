<script lang="ts">
    import { Play, BookOpen, FileText } from "lucide-svelte";
    import type { ContinueItem } from '@/api/progress/types';
    import type { ContentType } from '@/api/content/types';
    import { i18n } from "@/stores/i18n.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";
    import * as Carousel from "@/components/ui/carousel";
    import MpvLauncher from "@/components/mpv/MpvLauncher.svelte";

    let {
        items,
        mode
    }: {
        items: ContinueItem[];
        mode: ContentType;
    } = $props();

    let mpvOpen = $state(false);
    let mpvItem = $state<ContinueItem | null>(null);
    let mpvEpTitle = $state("");

    function handleAnimeClick(e: MouseEvent, item: ContinueItem) {
        if (!item.episode) return;
        if (appConfig.data?.mpv?.useMpv){
            e.preventDefault();

            mpvItem = item;
            mpvEpTitle = item.unit?.title
                ? i18n.t('watch.episode_with_title', { num: item.episode, title: item.unit.title })
                : i18n.t('watch.episode_number', { num: item.episode });

            mpvOpen = true;
        }
    }

    let visibleItems = $derived(items.filter(item => {
        return !(item.nsfw && !appConfig.data?.general?.showAdultContent);
    }));

    function isBlurred(item: ContinueItem) {
        return item.nsfw && appConfig.data?.general?.blurAdultContent;
    }

    function getDisplayTitle(item: ContinueItem) {
        if (!appConfig.data) return item.title;
        const lang = appConfig.data.ui.titleLanguage || 'romaji';
        return item.titleI18n?.[lang] || item.titleI18n?.['romaji'] || item.title;
    }

    function processImageUrl(url: string | null | undefined) {
        if (!url) return null;
        return url.replace('_m.', '_w.');
    }

    function getContinueUrl(item: ContinueItem) {
        if (item.contentType === 'anime' && item.episode) {
            const ratio = (item.episodeDurationSeconds && item.timestampSeconds)
                ? item.timestampSeconds / item.episodeDurationSeconds
                : 0;
            if (ratio >= 0.95) return `/watch/${item.cid}/${item.episode + 1}`;
            if (item.timestampSeconds && item.timestampSeconds > 0)
                return `/watch/${item.cid}/${item.episode}?t=${item.timestampSeconds}`;
            return `/watch/${item.cid}/${item.episode}`;
        }
        if (item.contentType === 'manga')
            return `/c/${item.cid}`;
        if (item.contentType === 'novel')
            return `/c/${item.cid}`;
        return `/c/${item.cid}`;
    }

    function getProgressPercent(item: ContinueItem) {
        if (!item.timestampSeconds || !item.episodeDurationSeconds) return 0;
        return Math.min((item.timestampSeconds / item.episodeDurationSeconds) * 100, 100);
    }
</script>

{#if visibleItems.length > 0}
    <div class="space-y-4">
        <h2 class="text-xl md:text-2xl font-black tracking-tight text-foreground px-1">
            {mode === 'anime' ? i18n.t("home.continue.continue_watching") : i18n.t("home.continue.continue_reading")}
        </h2>

        <Carousel.Root
                opts={{ align: "start", dragFree: true }}
                class="w-full px-1"
        >
            <Carousel.Content class="-ml-5 pt-1 pb-4">
                {#each visibleItems as item}
                    {@const rawImg = item.unit?.thumbnailUrl || item.coverImage}
                    {@const imageUrl = processImageUrl(rawImg)}
                    {@const displayTitle = getDisplayTitle(item)}
                    {@const progressPercent = getProgressPercent(item)}

                    <Carousel.Item class="pl-5 {mode === 'anime' ? 'basis-[275px] sm:basis-[360px]' : 'basis-[220px] sm:basis-[260px]'}">
                        {#if mode === 'anime'}
                            <a href={getContinueUrl(item)}
                               onclick={(e) => handleAnimeClick(e, item)}
                               class="anime-card group flex flex-col gap-4 focus-visible:outline-none"
                            >
                            <div class="relative w-full aspect-video overflow-hidden bg-muted/20 border border-border/30 rounded-sm">
                                {#if imageUrl}
                                    <img
                                            src={imageUrl}
                                            alt={displayTitle}
                                            class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-[1.03] {isBlurred(item) ? 'blur-xl scale-110' : ''}"
                                    />
                                {:else}
                                    <div class="w-full h-full flex items-center justify-center">
                                            <span class="text-6xl font-black text-muted-foreground/10 select-none">
                                                {item.episode ?? '?'}
                                            </span>
                                    </div>
                                {/if}

                                <div class="absolute inset-x-0 bottom-0 h-1/3 bg-gradient-to-t from-black/60 to-transparent pointer-events-none"></div>

                                {#if item.nsfw}
                                    <div class="absolute top-2 right-2 bg-destructive text-white text-[10px] font-black uppercase px-2 py-0.5 rounded-sm tracking-wider">
                                        18+
                                    </div>
                                {/if}

                                {#if progressPercent > 0}
                                    <div class="absolute bottom-0 left-0 right-0 h-[4px] bg-white/10">
                                        <div class="h-full bg-primary transition-all shadow-[0_0_8px_rgba(var(--primary),0.6)]" style="width: {progressPercent}%"></div>
                                    </div>
                                {/if}
                            </div>

                            <div class="flex flex-col gap-1 px-1">
                                <p class="text-[11px] font-bold text-muted-foreground/60 uppercase tracking-widest truncate">
                                    {displayTitle}
                                </p>
                                <div class="flex items-baseline gap-2 overflow-hidden">
                                        <span class="font-black text-base md:text-sm whitespace-nowrap group-hover:text-primary transition-colors">
                                            {item.episode}
                                        </span>
                                    {#if item.unit?.title}
                                        <span class="text-muted-foreground/40 text-xs">.</span>
                                        <span class="text-sm md:text-base font-semibold text-muted-foreground line-clamp-1">
                                                {item.unit.title}
                                            </span>
                                    {/if}
                                </div>
                            </div>
                            </a>
                        {:else}
                            {@const coverImg = processImageUrl(item.coverImage)}

                            <a href={getContinueUrl(item)}
                            class="reader-card group flex gap-3 w-full p-2.5 border border-border/30 bg-card/60 hover:border-border/70 hover:bg-card transition-all duration-200 focus-visible:outline-none rounded-sm"
                            >
                            <div class="relative shrink-0 w-[60px] aspect-[2/3] overflow-hidden bg-muted/20 rounded-sm">
                                {#if coverImg}
                                    <img
                                            src={coverImg}
                                            alt={displayTitle}
                                            class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-[1.04] {isBlurred(item) ? 'blur-xl scale-110' : ''}"
                                    />
                                {:else}
                                    <div class="w-full h-full flex items-center justify-center bg-muted/30">
                                        {#if mode === 'novel'}
                                            <FileText class="size-5 text-muted-foreground/30" />
                                        {:else}
                                            <BookOpen class="size-5 text-muted-foreground/30" />
                                        {/if}
                                    </div>
                                {/if}
                            </div>

                            <div class="flex flex-col justify-between flex-1 min-w-0 py-0.5">
                                <div class="space-y-1">
                                    <p class="font-bold text-[12px] leading-snug line-clamp-2 group-hover:text-primary transition-colors">
                                        {displayTitle}
                                    </p>
                                    <p class="text-[10px] text-muted-foreground font-semibold uppercase tracking-wide">
                                        {i18n.t('home.continue.chapters', { num: item.chapter })}
                                    </p>
                                </div>

                                <div class="flex items-center gap-1.5 mt-2">
                                    <div class="w-5 h-5 rounded-sm bg-primary/10 flex items-center justify-center group-hover:bg-primary transition-colors duration-200">
                                        {#if mode === 'novel'}
                                            <FileText class="size-3 text-primary group-hover:text-primary-foreground transition-colors" />
                                        {:else}
                                            <BookOpen class="size-3 text-primary group-hover:text-primary-foreground transition-colors" />
                                        {/if}
                                    </div>
                                    <span class="text-[10px] font-bold text-muted-foreground group-hover:text-foreground transition-colors">
                                            {i18n.t('home.continue.continue_reading')}
                                        </span>
                                </div>
                            </div>
                            </a>
                        {/if}
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    </div>
{/if}

{#if mpvOpen && mpvItem}
    <MpvLauncher
            cid={mpvItem.cid}
            epNumber={mpvItem.episode ?? 1}
            epTitle={mpvEpTitle}
            animeTitle={mpvItem.title}
            totalEpisodes={0}
            isNsfw={mpvItem.nsfw}
            coverImage={mpvItem.coverImage ?? undefined}
            startTime={mpvItem.timestampSeconds ?? 0}
            bind:open={mpvOpen}
    />
{/if}