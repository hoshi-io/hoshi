<script lang="ts">
    import { fade } from 'svelte/transition';
    import { Skeleton } from '@/components/ui/skeleton';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { auth } from '@/stores/auth.svelte.js';
    import { homeState } from '@/app/home.svelte.js';
    import type { NormalizedCard } from '@/utils/normalize';
    import TvHero from '@/tv/components/TvHero.svelte';
    import TvRow, { TV_ROW_MAX } from '@/tv/components/TvRow.svelte';
    import TvCard from '@/tv/components/card/TvCard.svelte';
    import TvContinueRow from '@/tv/components/card/TvContinueRow.svelte';
    import TvRecentlyAiredRow from '@/tv/components/card/TvRecentlyAiredRow.svelte';

    // TV is anime-only for now.
    const section = $derived(homeState.getSection('anime'));
    const continueItems = $derived(homeState.getContinueItems('anime'));
    const heroItems = $derived(section?.trending.slice(0, 5) ?? []);

    const isSkeletonVisible = $derived(
        (auth.loading || !auth.initialized || homeState.loading) && !homeState.hasData
    );

    $effect(() => {
        if (auth.loading || !auth.initialized || !auth.user) return;

        if (homeState.hasData) {
            homeState.refreshContinueWatching();
        } else {
            homeState.load();
        }
    });
</script>

<svelte:head><title>{i18n.t('home.title')}</title></svelte:head>

{#snippet cardRow(title: string, items: NormalizedCard[])}
    {#if items.length > 0}
        <TvRow {title}>
            {#each items.slice(0, TV_ROW_MAX) as card}
                <TvCard {card} />
            {/each}
        </TvRow>
    {/if}
{/snippet}

{#if isSkeletonVisible}
    <div in:fade={{ duration: 200 }}>
        <Skeleton class="-mx-12 -mt-8 h-[60dvh] min-h-[300px] rounded-none" />
        <div class="relative -mt-6 space-y-6">
            {#each Array(2) as _}
                <div class="space-y-3 px-3">
                    <Skeleton class="h-6 w-40 rounded-md" />
                    <div class="flex gap-4 overflow-hidden">
                        {#each Array(6) as __}
                            <Skeleton class="aspect-[2/3] w-36 shrink-0 rounded-md" />
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </div>
{:else}
    <div in:fade={{ duration: 250 }}>
        <TvHero items={heroItems} />

        <div class="relative z-10 -mt-6 space-y-1 pb-12">
            <TvContinueRow items={continueItems} />

            {#if section}
                {@render cardRow(i18n.t('home.trending'), section.trending)}
                <TvRecentlyAiredRow />
                {@render cardRow(i18n.t('home.popular'), section.popular)}
                {@render cardRow(i18n.t('home.simulcast'), section.seasonal)}
                {@render cardRow(i18n.t('home.upcoming'), section.upcoming)}
                {@render cardRow(i18n.t('home.recently_finished'), section.recentlyFinished)}
                {@render cardRow(i18n.t('home.critically_acclaimed'), section.topRated)}
                {@render cardRow(i18n.t('home.action'), section.topAction)}
            {/if}
        </div>
    </div>
{/if}