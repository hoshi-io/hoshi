<script lang="ts">
    import { Skeleton } from "@/components/ui/skeleton";
    import { i18n } from "@/stores/i18n.svelte.js";
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import { layoutState } from "@/stores/layout.svelte.js";
    import ShowMoreToggle from "@/components/common/ShowMoreToggle.svelte";
    import type {NormalizedRelation} from "@/utils/normalize";

    let { relations, loading = false }: {
        relations: NormalizedRelation[];
        loading?: boolean;
    } = $props();

    let showAll = $state(false);
    /** One row of the grid: 3 columns on mobile, 7 on desktop. */
    const INITIAL_LIMIT = $derived(layoutState.isMobile ? 3 : 7);

    const visibleRelations = $derived(showAll ? relations : relations.slice(0, INITIAL_LIMIT));

    function formatRelationType(type: string) {
        if (!type) return '';
        const key = `relations.${type.toUpperCase()}` as any;
        const translated = i18n.t(key);
        if (translated === key) {
            return type.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, l => l.toUpperCase());
        }
        return translated;
    }

    function relationKey(relation: NormalizedRelation) {
        return relation.targetCid ?? `${relation.targetTrackerName}:${relation.targetTrackerId}`;
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between gap-3">
        <h3 class="text-xl md:text-2xl font-bold tracking-tight">{i18n.t('content.related')}</h3>
        {#if !loading && relations.length > INITIAL_LIMIT}
            <ShowMoreToggle bind:expanded={showAll} total={relations.length} />
        {/if}
    </div>

    {#if loading}
        <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3 sm:gap-6">
            {#each Array(INITIAL_LIMIT) as _}
                <div class="flex flex-col gap-2.5 w-full">
                    <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                    <div class="space-y-2 px-1">
                        <Skeleton class="h-3 w-1/3 bg-muted" />
                        <Skeleton class="h-4 w-3/4 bg-muted" />
                    </div>
                </div>
            {/each}
        </div>
    {:else if relations.length === 0}
        <p class="text-muted-foreground">{i18n.t('content.no_related')}</p>
    {:else}
        <div class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-7 gap-3 sm:gap-6">
            {#each visibleRelations as relation (relationKey(relation))}
                <div class="relative group w-full h-full animate-in fade-in duration-300">
                    {#snippet relationOverlay()}
                        <div class="absolute top-2 left-2 z-20 bg-background/95 backdrop-blur-md px-2 py-1 rounded text-[9px] font-black uppercase tracking-widest border border-border/50 shadow-md text-foreground">
                            {formatRelationType(relation.relationType)}
                        </div>
                    {/snippet}

                    <CardWrapper
                            {...relation.card}
                            disablePreview={true}
                            overlay={relationOverlay}
                    />
                </div>
            {/each}
        </div>
    {/if}
</div>