<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import * as Carousel from "@/components/ui/carousel";
    import EpisodeCard from "@/components/card/EpisodeCard.svelte";
    import {
        displayTitle,
        episodeLabel,
        relativeAired,
        shouldBlurNsfw,
        useRecentlyAired,
        watchUrl,
        wideImage
    } from "@/utils/episode-rows.svelte.js";

    const rows = useRecentlyAired();
</script>

{#if rows.items.length > 0}
    <div class="space-y-2 md:space-y-4">
        <h2 class="text-lg md:text-2xl font-bold tracking-tight text-foreground px-1">
            {i18n.t("home.recently_aired.title")}
        </h2>

        <Carousel.Root
                opts={{ align: "start", dragFree: true }}
                class="w-full px-1"
        >
            <Carousel.Content class="-ml-5 pt-2 pb-2 md:pb-6">
                {#each rows.items as item (item.trackerId + ':' + item.episode)}
                    {@const series = displayTitle(item.card.titleI18n, item.card.titleDefault)}
                    {@const ep = episodeLabel(item.episode, item.unitTitle)}

                    <Carousel.Item class="pl-5 basis-[275px] sm:basis-[360px]">
                        <EpisodeCard
                                href={watchUrl(item)}
                                image={wideImage(item.card.cover)}
                                alt={series}
                                blur={shouldBlurNsfw(item.card.nsfw)}
                                nsfw={!!item.card.nsfw}
                                {series}
                                episode={ep.number}
                                episodeTitle={ep.title}
                                badge={relativeAired(item)}
                                placeholder={String(item.episode ?? '?')}
                        />
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    </div>
{/if}