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
            if (item.timestampSeconds && item.timestampSeconds > 0) {
                return `/watch/${item.cid}/${item.episode}?t=${item.timestampSeconds}`;
            }
            return `/watch/${item.cid}/${item.episode}`;
        }

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

                    <!-- Equalized column basis bounds for structural parity across formats -->
                    <Carousel.Item class="pl-5 {mode === 'anime' ? 'basis-[275px] sm:basis-[360px]' : 'basis-[155px] sm:basis-[185px]'}">
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
                                        #{/if}

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

                            <!-- NEW PORTRAIT CARDS -->
                            <a href={getContinueUrl(item)}
                               class="reader-card group flex flex-col gap-3 w-full focus-visible:outline-none"
                            >
                                <div class="relative w-full aspect-[2/3] overflow-hidden bg-muted/15 border border-border/30 rounded-sm shadow-sm">
                                    {#if coverImg}
                                        <img
                                                src={coverImg}
                                                alt={displayTitle}
                                                class="w-full h-full object-cover transition-all duration-500 group-hover:scale-[1.04] group-hover:brightness-110 {isBlurred(item) ? 'blur-xl scale-110' : ''}"
                                        />
                                    {:else}
                                        <div class="w-full h-full flex items-center justify-center bg-muted/20">
                                            {#if mode === 'novel'}
                                                <FileText class="size-6 text-muted-foreground/30" />
                                            {:else}
                                                <BookOpen class="size-6 text-muted-foreground/30" />
                                            {/if}
                                        </div>
                                    {/if}

                                    <div class="absolute top-2 left-2 bg-background/80 backdrop-blur-md text-muted-foreground font-mono text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-sm border border-border/20 flex items-center gap-1">
                                        {#if mode === 'novel'}
                                            <FileText class="size-2.5 text-primary" />
                                            <span>LN</span>
                                        {:else}
                                            <BookOpen class="size-2.5 text-primary" />
                                            <span>MANGA</span>
                                        {/if}
                                    </div>

                                    {#if item.nsfw}
                                        <div class="absolute top-2 right-2 bg-destructive text-white text-[9px] font-black uppercase px-1.5 py-0.5 rounded-sm tracking-wider">
                                            18+
                                        </div>
                                    {/if}

                                    <div class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/80 via-black/40 to-transparent pointer-events-none flex items-end p-2">
                                        <p class="text-[10px] text-white/90 font-black tracking-wide uppercase line-clamp-1">
                                            {i18n.t('home.continue.chapters', { num: item.chapter })}
                                        </p>
                                    </div>
                                </div>

                                <div class="flex flex-col gap-0.5 px-0.5">
                                    <p class="font-black text-sm text-foreground line-clamp-1 group-hover:text-primary transition-colors duration-200">
                                        {displayTitle}
                                    </p>
                                    <span class="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-wider">
                                        {i18n.t('home.continue.continue_reading')} →
                                    </span>
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