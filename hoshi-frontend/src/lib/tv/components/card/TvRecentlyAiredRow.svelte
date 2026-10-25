<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import TvRow, { TV_ROW_MAX } from '@/tv/components/TvRow.svelte';
    import TvEpisodeCard from '@/tv/components/card/TvEpisodeCard.svelte';
    import {
        displayTitle,
        episodeLabel,
        relativeAired,
        shouldBlurNsfw,
        useRecentlyAired,
        watchUrl,
        wideImage
    } from '@/utils/episode-rows.svelte.js';

    const rows = useRecentlyAired(TV_ROW_MAX);
</script>

{#if rows.items.length > 0}
    <TvRow title={i18n.t('home.recently_aired.title')}>
        {#each rows.items as item (item.trackerId + ':' + item.episode)}
            {@const series = displayTitle(item.card.titleI18n, item.card.titleDefault)}
            {@const ep = episodeLabel(item.episode, item.unitTitle)}
            <TvEpisodeCard
                    href={watchUrl(item, '/tv')}
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
        {/each}
    </TvRow>
{/if}