<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import { scheduleStore, type RecentlyAiredItem } from '@/app/schedule.svelte.js';
    import TvRow, { TV_ROW_MAX } from '@/tv/TvRow.svelte';
    import TvEpisodeCard from '@/tv/components/card/TvEpisodeCard.svelte';
    import { wideImage, displayTitle, shouldHideNsfw, shouldBlurNsfw } from '@/tv/helpers.ts';

    $effect(() => {
        scheduleStore.load();
    });

    const visibleItems = $derived(
        scheduleStore.recentlyAired
            .filter((item) => {
                if (shouldHideNsfw(item.card.nsfw)) return false;
                if (item.card.score == null || item.card.score < 6) return false;
                return true;
            })
            .slice(0, TV_ROW_MAX)
    );

    // NOTE: /watch is still the desktop player. Swap the prefix here once /tv/watch exists.
    function watchUrl(item: RecentlyAiredItem) {
        return `/watch/${item.card.cid}/${item.episode}`;
    }

    function toMs(ts: number) {
        return ts > 1e11 ? ts : ts * 1000;
    }

    function relativeAired(item: RecentlyAiredItem) {
        const hours = Math.floor((Date.now() - toMs(item.airingAt)) / 3_600_000);
        if (hours < 1) return i18n.t('home.recently_aired.just_aired');
        if (hours < 24) return i18n.t('home.recently_aired.hours_ago', { count: hours });
        return i18n.t('home.recently_aired.days_ago', { count: Math.floor(hours / 24) });
    }
</script>

{#if visibleItems.length > 0}
    <TvRow title={i18n.t('home.recently_aired.title')}>
        {#each visibleItems as item (item.trackerId + ':' + item.episode)}
            {@const series = displayTitle(item.card.titleI18n, item.card.titleDefault)}
            <TvEpisodeCard
                    href={watchUrl(item)}
                    image={wideImage(item.card.cover)}
                    alt={series}
                    blur={shouldBlurNsfw(item.card.nsfw)}
                    nsfw={!!item.card.nsfw}
                    {series}
                    episode={item.unitTitle ? String(item.episode) : i18n.t('watch.episode_number', { num: item.episode })}
                    episodeTitle={item.unitTitle ?? null}
                    badge={relativeAired(item)}
                    placeholder={String(item.episode ?? '?')}
            />
        {/each}
    </TvRow>
{/if}