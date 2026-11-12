<script lang="ts">
    import CardCarousel from '@/components/carousel/CardCarousel.svelte';
    import StripCarousel from '@/components/carousel/StripCarousel.svelte';
    import type { SectionLayout } from '@/api/home/types';
    import type { NormalizedCard } from '@/utils/normalize';
    import type { Snippet } from 'svelte';

    let { title, items, layout, loading = false, onvisible, actions }: {
        title: string;
        items: NormalizedCard[];
        layout: SectionLayout;
        loading?: boolean;
        onvisible?: () => void;
        actions?: Snippet;
    } = $props();
</script>

{#if layout === 'strip'}
    <StripCarousel {title} {items} {loading} {onvisible} {actions} />
{:else}
    <CardCarousel {title} {items} {loading} {onvisible} {actions} ranked={layout === 'ranked'} />
{/if}
