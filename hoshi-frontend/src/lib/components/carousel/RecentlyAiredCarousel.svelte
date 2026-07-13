<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";
    import * as Carousel from "@/components/ui/carousel";
    import MpvLauncher from "@/components/mpv/MpvLauncher.svelte";
    import {type RecentlyAiredItem, scheduleStore} from "@/app/schedule.svelte";

    $effect(() => {
        scheduleStore.load();
    });

    let mpvOpen = $state(false);
    let mpvItem = $state<RecentlyAiredItem | null>(null);
    let mpvEpTitle = $state("");

    let visibleItems = $derived(
        scheduleStore.recentlyAired.filter(item => {
            return !(item.card.nsfw && !appConfig.data?.general?.showAdultContent);
        })
    );

    function isBlurred(item: RecentlyAiredItem) {
        return item.card.nsfw && appConfig.data?.general?.blurAdultContent;
    }

    function getDisplayTitle(item: RecentlyAiredItem) {
        if (!appConfig.data) return item.card.titleDefault;
        const lang = appConfig.data.ui.titleLanguage || 'romaji';
        return item.card.titleI18n?.[lang] || item.card.titleI18n?.['romaji'] || item.card.titleDefault;
    }

    function processImageUrl(url: string | null | undefined) {
        if (!url) return null;
        return url.replace('_m.', '_w.');
    }

    function getWatchUrl(item: RecentlyAiredItem) {
        return `/watch/${item.card.cid}/${item.episode}`;
    }

    function handleClick(e: MouseEvent, item: RecentlyAiredItem) {
        if (appConfig.data?.mpv?.useMpv) {
            e.preventDefault();

            mpvItem = item;
            mpvEpTitle = item.unitTitle
                ? i18n.t('watch.episode_with_title', { num: item.episode, title: item.unitTitle })
                : i18n.t('watch.episode_number', { num: item.episode });

            mpvOpen = true;
        }
    }

    function getMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }

    function getRelativeAired(item: RecentlyAiredItem) {
        const diffMs = Date.now() - getMs(item.airingAt);
        const hours  = Math.floor(diffMs / 3_600_000);
        if (hours < 1)  return i18n.t("home.recently_aired.just_aired");
        if (hours < 24) return i18n.t("home.recently_aired.hours_ago", { count: hours });
        const days = Math.floor(hours / 24);
        return i18n.t("home.recently_aired.days_ago", { count: days });
    }
</script>

{#if visibleItems.length > 0}
    <div class="space-y-4">
        <h2 class="text-xl md:text-2xl font-black tracking-tight text-foreground px-1">
            {i18n.t("home.recently_aired.title")}
        </h2>

        <Carousel.Root
                opts={{ align: "start", dragFree: true }}
                class="w-full px-1"
        >
            <Carousel.Content class="-ml-5 pt-1 pb-4">
                {#each visibleItems as item (item.trackerId + ':' + item.episode)}
                    {@const imageUrl = processImageUrl(item.card.cover)}
                    {@const displayTitle = getDisplayTitle(item)}

                    <Carousel.Item class="pl-5 basis-[275px] sm:basis-[360px]">
                        <a href={getWatchUrl(item)}
                           onclick={(e) => handleClick(e, item)}
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

                                {#if item.card.nsfw}
                                    <div class="absolute top-2 right-2 bg-destructive text-white text-[10px] font-black uppercase px-2 py-0.5 rounded-sm tracking-wider">
                                        18+
                                    </div>
                                {/if}

                                <div class="absolute top-2 left-2 bg-background/80 backdrop-blur-md text-muted-foreground font-mono text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-sm border border-border/20">
                                    {getRelativeAired(item)}
                                </div>
                            </div>

                            <div class="flex flex-col gap-1 px-1">
                                <p class="text-[11px] font-bold text-muted-foreground/60 uppercase tracking-widest truncate">
                                    {displayTitle}
                                </p>
                                <div class="flex items-baseline gap-2 overflow-hidden">
                                    <span class="font-black text-base md:text-sm whitespace-nowrap group-hover:text-primary transition-colors">
                                        {item.unitTitle ? item.episode : i18n.t('watch.episode_number', { num: item.episode })}
                                    </span>
                                    {#if item.unitTitle}
                                        <span class="text-muted-foreground/40 text-xs">.</span>
                                        <span class="text-sm md:text-base font-semibold text-muted-foreground line-clamp-1">
                                            {item.unitTitle}
                                        </span>
                                    {/if}
                                </div>
                            </div>
                        </a>
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    </div>
{/if}

{#if mpvOpen && mpvItem}
    <MpvLauncher
            cid={mpvItem.card.cid}
            epNumber={mpvItem.episode}
            epTitle={mpvEpTitle}
            animeTitle={mpvItem.card.titleDefault}
            totalEpisodes={0}
            isNsfw={mpvItem.card.nsfw}
            coverImage={mpvItem.card.cover ?? undefined}
            startTime={0}
            bind:open={mpvOpen}
    />
{/if}