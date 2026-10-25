<script lang="ts">
    import { Star } from 'lucide-svelte';
    import SmartImage from '@/components/SmartImage.svelte';
    import {
        getCardTitle,
        getCardShouldBlur,
        getCardScore,
        getCardScoreIsStars,
        getCardContentTypeLabel,
        type NormalizedCard
    } from '@/utils/normalize';

    let { card }: { card: NormalizedCard } = $props();

    const title = $derived(getCardTitle(card));
    const shouldBlur = $derived(getCardShouldBlur(card));
    const score = $derived(getCardScore(card));
    const isStars = $derived(getCardScoreIsStars(card));
    const typeLabel = $derived(getCardContentTypeLabel(card));
</script>

<!-- Focus is the "hover" of a TV: the <a> takes focus, the poster reacts via group-focus. -->
<a
        href={card.href}
        data-sn
        class="group block w-36 shrink-0 scroll-my-16 outline-none"
>
    <div
            class="relative aspect-[2/3] overflow-hidden rounded-md bg-secondary
               transition-transform duration-150
               group-focus:scale-105 group-focus:ring-4 group-focus:ring-primary"
    >
        <div class="absolute inset-0">
            <SmartImage
                    src={card.cover}
                    alt={title}
                    {shouldBlur}
                    imageHeaders={card.imageHeaders}
                    class="size-full object-cover"
            />
        </div>

        {#if score}
            <div class="absolute right-1.5 top-1.5 flex items-center gap-1 rounded bg-black/60 px-1.5 py-0.5 text-xs font-bold text-white">
                {#if !isStars}
                    <Star class="size-3 fill-yellow-400 text-yellow-400" />
                {/if}
                <span class={isStars ? 'text-amber-400' : ''}>{score}</span>
            </div>
        {/if}
    </div>

    <div class="mt-2 px-0.5">
        <div class="mb-0.5 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
            {typeLabel ?? ''}{card.year ? ` · ${card.year}` : ''}
        </div>
        <h3 class="line-clamp-2 text-sm font-bold leading-tight text-foreground group-focus:text-primary">
            {title}
        </h3>
    </div>
</a>