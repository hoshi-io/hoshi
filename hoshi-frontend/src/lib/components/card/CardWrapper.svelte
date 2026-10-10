<script lang="ts">
    import CardContainer from '@/components/card/CardContainer.svelte';
    import CardPreview from '@/components/card/CardPreview.svelte';
    import {getCardContentTypeLabel, getCardScore, getCardScoreIsStars, type NormalizedCard} from "@/utils/normalize";
    import { getCardTitle, getCardShouldBlur, getCardTrailerUrl } from "@/utils/normalize";
    import { layoutState } from "@/stores/layout.svelte";
    import { scale, fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
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

    const HOVER_DELAY = 220;
    let hoverTimer: ReturnType<typeof setTimeout> | undefined;

    function onEnter() {
        if (disablePreview || isMobile) return;
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => (isHovering = true), HOVER_DELAY);
    }

    function onLeave() {
        clearTimeout(hoverTimer);
        isHovering = false;
    }

    $effect(() => () => clearTimeout(hoverTimer));

    let contentTypeLabel = $derived(getCardContentTypeLabel(card));
</script>

<div class="card-ct group relative" class:no-preview={disablePreview} onmouseenter={onEnter} onmouseleave={onLeave}>
    <a
            href={card.href}
            class="card-base block w-full outline-none cursor-pointer h-full overflow-hidden"
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
        <div
                class="preview-anchor"
                in:scale={{ start: 0.92, duration: 220, easing: cubicOut }}
                out:fade={{ duration: 120 }}
        >
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
        top: 0;
        left: -12.5%;
        width: 125%;
        height: 100%;
        transform: scale(1.05);
        transform-origin: center;
    }

    .card-ct:not(:hover) .preview-anchor {
        pointer-events: none;
    }
</style>