<script lang="ts">
    import SearchSection from './SearchSection.svelte';
    import BuiltinSection from './BuiltinSection.svelte';
    import type { HomeSection } from '@/api/home/types';

    let { section }: { section: HomeSection } = $props();
</script>

{#if section.source.kind === 'search'}
    <SearchSection {section} />
{:else}
    <BuiltinSection {section} />
{/if}
