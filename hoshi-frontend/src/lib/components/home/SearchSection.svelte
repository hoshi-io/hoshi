<script lang="ts">
    import CardsSection from './CardsSection.svelte';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import type { HomeSection } from '@/api/home/types';

    let { section }: { section: HomeSection } = $props();

    const feed = $derived(section.source.kind === 'search' ? homeState.feed(section.source.query) : null);
    const empty = $derived(!!feed && (feed.error !== null || (feed.loaded && feed.items.length === 0)));
</script>

{#if feed && !empty}
    <CardsSection
            title={labelText(section.title)}
            items={feed.items}
            layout={section.layout}
            loading={feed.loading}
            onvisible={() => feed.load()}
    />
{/if}
