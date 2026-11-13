<script lang="ts">
    import CardsSection from './CardsSection.svelte';
    import EmptySection from './EmptySection.svelte';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { HomeSection } from '@/api/home/types';
    import ChevronRight from 'phosphor-svelte/lib/CaretRightIcon';
    import { goto } from '$app/navigation';
    import { searchState } from '@/app/search.svelte.js';

    let { section }: { section: HomeSection } = $props();

    const feed = $derived(section.source.kind === 'search' ? homeState.feed(section.source.query) : null);
    const title = $derived(labelText(section.title));

    function seeAll() {
        if (section.source.kind !== 'search') return;
        searchState.openQuery(section.source.query);
        goto('/search');
    }
</script>

{#snippet seeAllLink()}
    <button
            type="button"
            onclick={seeAll}
            class="flex shrink-0 items-center gap-0.5 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
    >
        {i18n.t('home.see_all')}
        <ChevronRight class="size-4" />
    </button>
{/snippet}

{#if feed}
    {#if feed.error && feed.items.length === 0}
        <EmptySection {title} layout={section.layout} message={i18n.t('home.empty.error')} onretry={() => feed.retry()} />
    {:else if !feed.loaded || feed.items.length > 0}
        <CardsSection
                {title}
                items={feed.items}
                layout={section.layout}
                loading={feed.loading}
                onvisible={() => feed.load()}
                actions={feed.items.length > 0 ? seeAllLink : undefined}
        />
    {/if}
{/if}
