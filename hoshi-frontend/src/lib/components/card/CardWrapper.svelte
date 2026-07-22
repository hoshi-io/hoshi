<script lang="ts">
    import CardContainer from '@/components/card/CardContainer.svelte';
    import CardPreview from '@/components/card/CardPreview.svelte';
    import {getCardContentTypeLabel, getCardScore, getCardScoreIsStars, type NormalizedCard} from "@/utils/normalize";
    import { getCardTitle, getCardShouldBlur, getCardTrailerUrl } from "@/utils/normalize";
    import { layoutState } from "@/stores/layout.svelte";
    import type { Snippet } from "svelte";

    let {
        disablePreview = false,
        overlay,
        ...card
    }: NormalizedCard & {
        disablePreview?: boolean;
        overlay?: Snippet;
    } = $props();

    let title      = $derived(getCardTitle(card));
    let shouldBlur = $derived(getCardShouldBlur(card));
    let trailerUrl = $derived(getCardTrailerUrl(card));
    let isMobile   = $derived(layoutState.isMobile);
    let score      = $derived(getCardScore(card));
    let isStars    = $derived(getCardScoreIsStars(card));
    let isHovering = $state(false);


    let contentTypeLabel = $derived(getCardContentTypeLabel(card));
</script>

<div class="card-ct group relative" class:no-preview={disablePreview} onmouseenter={() => isHovering = true} onmouseleave={() => isHovering = false}>
    <a
            href={card.href}
            class="card-base block w-full outline-none cursor-pointer h-full overflow-hidden {disablePreview || isMobile ? '' : 'transition-opacity duration-300 group-hover:opacity-0'}"
    >
        <CardContainer
                {title}
                cover={card.cover}
                year={card.year}
                score={score}
                {shouldBlur}
                contentTypeLabel={contentTypeLabel}
                {overlay}
                imageHeaders={card.imageHeaders}
                isStars={isStars}
        />
    </a>

    {#if !disablePreview && !isMobile && isHovering}
        <div class="preview-anchor">
            <CardPreview
                    cid={card.cid}
                    {title}
                    cover={card.cover}
                    bannerImage={card.bannerImage}
                    {trailerUrl}
                    score={score}
                    status={card.status}
                    synopsis={card.synopsis}
                    episodeCount={card.episodeCount}
                    contentType={card.contentType}
                    contentTypeLabel={card.contentTypeLabel}
                    {shouldBlur}
                    href={card.href}
                    isStar={isStars}
            />
        </div>
    {/if}
</div>

<style>
    .card-ct {
        position: relative;
        z-index: 1;
        overflow: visible;
    }

    .card-ct:hover {
        z-index: 30;
    }

    .card-base {
        overflow: hidden;
        border-radius: inherit;
    }

    .preview-anchor {
        position: absolute;
        top: 50%;
        left: 50%;

        width: 125%;
        height: 100%;

        transform: translate(-50%, -50%) scale(1);
        opacity: 0;

        pointer-events: none;

        transition:
                opacity 0.2s ease,
                transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    }

    .card-ct:hover .preview-anchor {
        opacity: 1;
        pointer-events: auto;
        transform: translate(-50%, -50%) scale(1.05);
    }
</style>