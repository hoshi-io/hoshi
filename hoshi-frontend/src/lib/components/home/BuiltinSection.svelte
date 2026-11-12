<script lang="ts">
    import CardsSection from './CardsSection.svelte';
    import ExtensionsCarousel from './ExtensionsCarousel.svelte';
    import ContinueCarousel from '@/components/carousel/ContinueCarousel.svelte';
    import RecentlyAiredCarousel from '@/components/carousel/RecentlyAiredCarousel.svelte';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import { listStore } from '@/app/list.svelte.js';
    import { scheduleStore } from '@/app/schedule.svelte.js';
    import { homeApi } from '@/api/home/home';
    import type { BuiltinSection, HomeSection } from '@/api/home/types';
    import { normalizeFullContent, normalizeRelationCard, type NormalizedCard } from '@/utils/normalize';
    import { shouldHideNsfw } from '@/utils/episode-rows.svelte.js';

    const LIMIT = 20;

    let { section }: { section: HomeSection } = $props();

    const builtin = $derived((section.source as { builtin: BuiltinSection }).builtin);
    const ct = $derived(section.contentType);
    const title = $derived(labelText(section.title));
    const usesSchedule = $derived(builtin.type === 'airingToday' || builtin.type === 'newEpisodes');

    let sequels = $state<NormalizedCard[]>([]);
    let sequelsLoading = $state(false);
    let sequelsRequested = $state(false);

    $effect(() => {
        if (usesSchedule) scheduleStore.load();
    });

    function loadSequels() {
        if (sequelsRequested) return;
        sequelsRequested = true;
        sequelsLoading = true;
        homeApi.unstartedSequels(ct ?? 'anime', LIMIT)
            .then(res => { sequels = res.map(normalizeRelationCard); })
            .catch(err => console.error('Failed to load sequels', err))
            .finally(() => { sequelsLoading = false; });
    }

    const toMs = (ts: number) => (ts > 1e11 ? ts : ts * 1000);
    const byType = <T extends { contentType: string }>(x: T) => !ct || x.contentType === ct;
    const time = (s: string | null | undefined) => (s ? new Date(s).getTime() || 0 : 0);

    function listCards(status: string, sort: 'updated' | 'ended' | 'random'): NormalizedCard[] {
        const rows = listStore.normalized.filter(r => r.original.status === status && byType(r.original));
        if (sort === 'random') {
            const shuffled = [...rows];
            for (let i = shuffled.length - 1; i > 0; i--) {
                const j = Math.floor(Math.random() * (i + 1));
                [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
            }
            return shuffled.slice(0, LIMIT).map(r => r.card);
        }
        const key = sort === 'ended'
            ? (r: typeof rows[number]) => time(r.original.endDate) || time(r.original.updatedAt)
            : (r: typeof rows[number]) => time(r.original.updatedAt);
        return [...rows].sort((a, b) => key(b) - key(a)).slice(0, LIMIT).map(r => r.card);
    }

    function scheduleCards(): NormalizedCard[] {
        const startOfDay = new Date().setHours(0, 0, 0, 0);
        const endOfDay = startOfDay + 86_400_000;
        const seen = new Set<string>();
        const out: NormalizedCard[] = [];

        const entries = builtin.type === 'airingToday'
            ? scheduleStore.entries.filter(e => {
                const ms = toMs(e.airingAt);
                return ms >= startOfDay && ms < endOfDay;
            })
            : [...scheduleStore.aired]
                .sort((a, b) => b.airingAt - a.airingAt)
                .filter(e => listStore.entries.some(le => le.cid === e.fullContent.content.cid && le.status === 'CURRENT'));

        for (const e of entries) {
            const card = normalizeFullContent(e.fullContent);
            if (!card.cid || seen.has(card.cid) || shouldHideNsfw(card.nsfw)) continue;
            seen.add(card.cid);
            out.push(card);
        }
        return out.slice(0, LIMIT);
    }

    // Random picks are drawn once per mount so the row doesn't reshuffle on every list change.
    let randomPicks = $state<NormalizedCard[] | null>(null);
    $effect(() => {
        if (builtin.type === 'randomPlanning' && randomPicks === null && listStore.isInitialized) {
            randomPicks = listCards('PLANNING', 'random');
        }
    });

    const cards = $derived.by<NormalizedCard[]>(() => {
        switch (builtin.type) {
            case 'listStatus': return listCards(builtin.status, 'updated');
            case 'recentlyCompleted': return listCards('COMPLETED', 'ended');
            case 'randomPlanning': return randomPicks ?? [];
            case 'airingToday':
            case 'newEpisodes': return scheduleCards();
            case 'unstartedSequels': return sequels;
            default: return [];
        }
    });

    const loading = $derived(
        builtin.type === 'unstartedSequels' ? sequelsLoading
            : usesSchedule ? scheduleStore.isLoading
            : listStore.isLoading
    );
</script>

{#if builtin.type === 'continueWatching'}
    {@const items = homeState.getContinueItems(ct)}
    {#if items.length > 0}
        <ContinueCarousel {items} mode={ct ?? 'anime'} />
    {/if}
{:else if builtin.type === 'recentlyAired'}
    <RecentlyAiredCarousel />
{:else if builtin.type === 'extensions'}
    <ExtensionsCarousel {title} contentType={ct} />
{:else if cards.length > 0 || loading || (builtin.type === 'unstartedSequels' && !sequelsRequested)}
    <CardsSection
            {title}
            items={cards}
            layout={section.layout}
            {loading}
            onvisible={builtin.type === 'unstartedSequels' ? loadSequels : undefined}
    />
{/if}
