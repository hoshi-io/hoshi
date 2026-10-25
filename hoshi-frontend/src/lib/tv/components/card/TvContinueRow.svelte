<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { ContinueItem } from '@/api/progress/types.js';
    import TvRow, { TV_ROW_MAX } from '@/tv/components/TvRow.svelte';
    import TvEpisodeCard from '@/tv/components/card/TvEpisodeCard.svelte';
    import { wideImage, displayTitle, shouldHideNsfw, shouldBlurNsfw } from '@/tv/helpers.js';

    let { items }: { items: ContinueItem[] } = $props();

    const visibleItems = $derived(items.filter((item) => !shouldHideNsfw(item.nsfw)).slice(0, TV_ROW_MAX));

    function continueUrl(item: ContinueItem) {
        if (item.contentType === 'anime' && item.episode) {
            const base = `/tv/watch/${item.cid}/${item.episode}`;
            return item.timestampSeconds && item.timestampSeconds > 0 ? `${base}?t=${item.timestampSeconds}` : base;
        }
        return `/tv/c/${item.cid}`;
    }

    function progressPercent(item: ContinueItem) {
        if (!item.timestampSeconds || !item.episodeDurationSeconds) return 0;
        return Math.min((item.timestampSeconds / item.episodeDurationSeconds) * 100, 100);
    }
</script>

{#if visibleItems.length > 0}
    <TvRow title={i18n.t('home.continue.continue_watching')}>
        {#each visibleItems as item}
            {@const series = displayTitle(item.titleI18n, item.title)}
            <TvEpisodeCard
                    href={continueUrl(item)}
                    image={wideImage(item.unit?.thumbnailUrl || item.coverImage)}
                    alt={series}
                    blur={shouldBlurNsfw(item.nsfw)}
                    nsfw={!!item.nsfw}
                    {series}
                    episode={item.unit?.title ? String(item.episode) : i18n.t('watch.episode_number', { num: item.episode ?? '?' })}
                    episodeTitle={item.unit?.title ?? null}
                    progress={progressPercent(item)}
                    placeholder={String(item.episode ?? '?')}
            />
        {/each}
    </TvRow>
{/if}