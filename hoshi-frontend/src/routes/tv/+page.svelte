<script lang="ts">
    import { fade } from 'svelte/transition';
    import { Skeleton } from '@/components/ui/skeleton';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { auth } from '@/stores/auth.svelte.js';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import type { NormalizedCard } from '@/utils/normalize';
    import TvHero from '@/tv/components/TvHero.svelte';
    import TvRow, { TV_ROW_MAX } from '@/tv/components/TvRow.svelte';
    import TvCard from '@/tv/components/card/TvCard.svelte';
    import TvContinueRow from '@/tv/components/card/TvContinueRow.svelte';
    import TvRecentlyAiredRow from '@/tv/components/card/TvRecentlyAiredRow.svelte';

    // TV is anime-only for now, on the default layout.
    const tab = $derived(homeState.tabs.find(t => t.id === 'anime') ?? null);
    const sections = $derived(tab?.sections.filter(s => !s.hidden) ?? []);
    const continueItems = $derived(homeState.getContinueItems('anime'));
    const heroSection = $derived(homeState.heroSection(tab));
    const heroItems = $derived(
        heroSection?.source.kind === 'search' ? homeState.feed(heroSection.source.query).items.slice(0, 5) : []
    );

    $effect(() => {
        for (const s of sections) {
            if (s.source.kind === 'search') homeState.feed(s.source.query).load();
        }
    });

    const isSkeletonVisible = $derived(
        (auth.loading || !auth.initialized || homeState.loading) && !homeState.hasData
    );

    $effect(() => {
        if (auth.loading || !auth.initialized || !auth.user) return;

        if (homeState.hasData) {
            homeState.refreshContinueWatching();
        } else {
            homeState.load({ defaultOnly: true });
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
            {#each sections as s (s.id)}
                {#if s.source.kind === 'search'}
                    {@render cardRow(labelText(s.title), homeState.feed(s.source.query).items)}
                {:else if s.source.builtin.type === 'continueWatching'}
                    <TvContinueRow items={continueItems} />
                {:else if s.source.builtin.type === 'recentlyAired'}
                    <TvRecentlyAiredRow />
                {/if}
            {/each}
        </div>
    </div>
{/if}