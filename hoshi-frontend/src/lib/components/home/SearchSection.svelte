<script lang="ts">
    import CardsSection from './CardsSection.svelte';
    import EmptySection from './EmptySection.svelte';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { HomeSection } from '@/api/home/types';

    let { section }: { section: HomeSection } = $props();

    const feed = $derived(section.source.kind === 'search' ? homeState.feed(section.source.query) : null);
    const title = $derived(labelText(section.title));
</script>

{#if feed}
    {#if feed.error && feed.items.length === 0}
        <EmptySection {title} layout={section.layout} message={i18n.t('home.empty.error')} onretry={() => feed.retry()} />
    {:else if feed.loaded && feed.items.length === 0}
        <EmptySection {title} layout={section.layout} message={i18n.t('home.empty.nothing')} />
    {:else}
        <CardsSection
                {title}
                items={feed.items}
                layout={section.layout}
                loading={feed.loading}
                onvisible={() => feed.load()}
        />
    {/if}
{/if}
