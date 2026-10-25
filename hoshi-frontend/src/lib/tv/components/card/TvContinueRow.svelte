<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { ContinueItem } from '@/api/progress/types.js';
    import TvRow, { TV_ROW_MAX } from '@/tv/components/TvRow.svelte';
    import TvEpisodeCard from '@/tv/components/card/TvEpisodeCard.svelte';
    import {
        continueProgress,
        continueUrl,
        displayTitle,
        episodeLabel,
        shouldBlurNsfw,
        useContinueItems,
        wideImage
    } from '@/utils/episode-rows.svelte.js';

    let { items }: { items: ContinueItem[] } = $props();

    const rows = useContinueItems(() => items, TV_ROW_MAX);
</script>

{#if rows.items.length > 0}
    <TvRow title={i18n.t('home.continue.continue_watching')}>
        {#each rows.items as item}
            {@const series = displayTitle(item.titleI18n, item.title)}
            {@const ep = episodeLabel(item.episode, item.unit?.title)}
            <TvEpisodeCard
                    href={continueUrl(item, '/tv')}
                    image={wideImage(item.unit?.thumbnailUrl || item.coverImage)}
                    alt={series}
                    blur={shouldBlurNsfw(item.nsfw)}
                    nsfw={!!item.nsfw}
                    {series}
                    episode={ep.number}
                    episodeTitle={ep.title}
                    progress={continueProgress(item)}
                    placeholder={String(item.episode ?? '?')}
            />
        {/each}
    </TvRow>
{/if}