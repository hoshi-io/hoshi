<script lang="ts">
    import {getCardScore, getCardScoreIsStars, type NormalizedCard} from '@/utils/normalize';
    import { getCardTitle, getCardShouldBlur, getCardTrailerUrl } from '@/utils/normalize';
    import { Button } from '@/components/ui/button';
    import { Play, Plus, Check } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import { i18n } from "@/stores/i18n.svelte.js";
    import { listStore } from '@/app/list.svelte.js';
    import ListEditorButton from "@/components/ListEditorButton.svelte";

    const YOUTUBE_REGEXP = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;

    let { items = [], animate = true }: { items: NormalizedCard[], animate?: boolean } = $props();

    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;

    let iframeReady = $state(false);

    let currentItem = $derived(items[currentIndex]);

    let title      = $derived(currentItem ? getCardTitle(currentItem) : "");
    let trailerUrl = $derived(currentItem ? getCardTrailerUrl(currentItem) : null);
    let shouldBlur = $derived(currentItem ? getCardShouldBlur(currentItem) : false);
    let score      = $derived(getCardScore(currentItem));

    function getYoutubeId(url: string | null | undefined): string | null {
        if (!url) return null;
        const match = url.match(YOUTUBE_REGEXP);
        return (match && match[2].length === 11) ? match[2] : null;
    }

    let trailerId    = $derived(getYoutubeId(trailerUrl));
    let thumbnailSrc = $derived(trailerId ? `https://i.ytimg.com/vi/${trailerId}/maxresdefault.jpg` : null);

    const startTimer = () => {
        if (items.length <= 1) return;
        clearInterval(timer);
        timer = setInterval(() => {
            currentIndex = (currentIndex + 1) % items.length;
        }, DURATION);
    };

    const pauseTimer  = () => clearInterval(timer);
    const resumeTimer = () => { if (items.length > 1) startTimer(); };
    const setSlide    = (index: number) => { currentIndex = index; startTimer(); };

    $effect(() => {
        if (items.length > 1) startTimer();
        return () => clearInterval(timer);
    });

    $effect(() => {
        currentIndex;
        iframeReady = false;
    });

    function onIframeLoad() {
        setTimeout(() => (iframeReady = true), 300);
    }
</script>

{#if currentItem}
    {@const cid = currentItem.href.replace('/c/', '')}
    {@const hasEntry = listStore.hasCid(cid)}

    <div
            class="relative w-full h-[75vh] md:h-[90vh] min-h-[550px] overflow-hidden bg-background select-none"
            onmouseenter={pauseTimer}
            onmouseleave={resumeTimer}
            role="region"
            aria-roledescription="carousel"
    >
        {#key currentItem.href}
            <div
                    class="absolute inset-0 w-full h-full block-layer"
                    in:fade={{ duration: animate ? 800 : 0 }}
                    out:fade={{ duration: animate ? 600 : 0 }}
            >
                {#if trailerId}
                    <div class="absolute inset-0 w-full h-full pointer-events-none overflow-hidden flex items-center justify-center opacity-85 scale-up-bg">
                        <img
                                src={thumbnailSrc}
                                alt=""
                                class="absolute inset-0 w-full h-full object-cover transition-opacity duration-700 {shouldBlur ? 'blur-2xl scale-110' : ''} {iframeReady ? 'opacity-0' : 'opacity-100'}"
                        />
                        <iframe
                                src="https://www.youtube.com/embed/{trailerId}?autoplay=1&mute=1&controls=0&loop=1&playlist={trailerId}&enablejsapi=1&rel=0&modestbranding=1"
                                title="Trailer"
                                class="w-[115vw] h-[115vh] min-w-[1920px] min-h-[1080px] object-cover pointer-events-none {shouldBlur ? 'blur-2xl scale-110' : ''}"
                                frameborder="0"
                                allow="autoplay; fullscreen; picture-in-picture"
                                onload={onIframeLoad}
                        ></iframe>
                    </div>
                {:else if currentItem.bannerImage}
                    <div class="absolute inset-0 w-full h-full scale-up-bg">
                        <img
                                src={currentItem.bannerImage}
                                alt={title}
                                class="w-full h-full object-cover object-center opacity-75 {shouldBlur ? 'blur-2xl scale-110' : ''}"
                        />
                    </div>
                {:else if currentItem.cover}
                    <div class="absolute inset-0 w-full h-full scale-up-bg">
                        <img
                                src={currentItem.cover}
                                alt={title}
                                class="w-full h-full object-cover object-center opacity-55 blur-xl scale-110"
                        />
                    </div>
                {/if}

                <div class="absolute inset-0 z-10 ambient-overlay"></div>

                <div class="absolute inset-0 z-20 w-full h-full max-w-[2000px] mx-auto px-6 md:px-16 lg:pl-32 flex flex-col justify-end pb-16 md:pb-24 pt-44 pointer-events-none">
                    <div class="max-w-3xl flex flex-col items-start text-layout pointer-events-auto">

                        <div class="flex flex-wrap items-center gap-2.5 text-xs font-bold text-foreground mb-4 item-stagger" style="--delay: 1">
                            {#if currentItem.contentTypeLabel}
                                <span class="bg-primary/10 text-primary px-2.5 py-0.5 rounded-md uppercase tracking-wider text-[11px] border border-primary/20 backdrop-blur-md">
                                    {currentItem.contentTypeLabel}
                                </span>
                            {/if}
                            {#if score}
                                <span class="bg-emerald-500/10 text-emerald-400 px-2.5 py-0.5 rounded-md text-[11px] border border-emerald-500/20 backdrop-blur-md font-black">
                                    {score}
                                </span>
                            {/if}
                            {#if currentItem.year}
                                <span class="px-1.5 text-foreground/80 font-medium">{currentItem.year}</span>
                            {/if}
                            {#if currentItem.episodeCount}
                                <span class="text-muted-foreground/90 font-medium">
                                    • {currentItem.contentType === 'anime' ? i18n.t('home.hero.eps', {count: currentItem.episodeCount}) : i18n.t('home.hero.chapters', {count: currentItem.episodeCount})}
                                </span>
                            {/if}
                        </div>

                        <div class="w-full flex flex-col gap-4">

                            <div class="w-full item-stagger" style="--delay: 2">
                                {#key currentIndex}
                                    <a href={currentItem.href} class="block group/title max-w-max no-underline">
                                        <h1
                                                in:fly={{ y: 8, duration: 400, delay: 50 }}
                                                class="font-black text-foreground tracking-tight fluid-hero-title line-clamp-2 cursor-pointer transition-opacity duration-200 group-hover/title:opacity-85"
                                        >
                                            {title}
                                        </h1>
                                    </a>
                                {/key}
                            </div>

                            <div class="w-full item-stagger mb-2" style="--delay: 4">
                                {#key currentIndex}
                                    <p
                                            in:fly={{ y: 8, duration: 400, delay: 180 }}
                                            class="text-muted-foreground/95 text-sm md:text-base font-medium leading-relaxed tracking-normal w-full text-justify text-justify-inter-word line-clamp-4 md:line-clamp-5"
                                    >
                                        {currentItem.synopsis || i18n.t('home.hero.no_synopsis')}
                                    </p>
                                {/key}
                            </div>

                        </div>

                        <div class="flex items-center gap-3 w-full sm:w-auto item-stagger" style="--delay: 3">
                            <a
                                    href={currentItem.href}
                                    class="flex-1 sm:flex-initial bg-primary hover:bg-primary/90 text-primary-foreground font-bold px-8 h-11 sm:h-12 rounded-lg flex items-center justify-center gap-2.5 transition-all active:scale-[0.98] shadow-md hover:shadow-lg shadow-primary/10 hover:shadow-primary/20 text-sm tracking-wide group"
                            >
                                <Play class="w-4 h-4 fill-current transition-transform group-hover:scale-110" />
                                {currentItem.contentType === 'anime' ? i18n.t('home.hero.watch') : i18n.t('home.hero.read')}
                            </a>

                            <ListEditorButton
                                    cid={cid}
                                    title={title}
                                    contentType={currentItem.contentType}
                                    coverImage={currentItem.cover ?? undefined}
                                    size="icon"
                                    class="h-11 w-11 sm:h-12 sm:w-12 rounded-lg bg-secondary/40 border border-border/40 hover:bg-secondary/70 text-foreground transition-all active:scale-[0.98] hover:border-border/80"
                            />
                        </div>
                    </div>
                </div>
            </div>
        {/key}

        {#if items.length > 1}
            <div class="absolute bottom-8 right-6 md:right-16 z-30 flex gap-2 items-center">
                {#each items as _, i}
                    <button
                            aria-label={`Ir a diapositiva ${i + 1}`}
                            class="h-1.5 rounded-full transition-all duration-300 {i === currentIndex
                        ? 'w-6 bg-primary'
                        : 'w-1.5 bg-foreground/30 hover:bg-foreground/60'}"
                            onclick={() => setSlide(i)}
                    ></button>
                {/each}
            </div>
        {/if}
    </div>
{/if}

<style>
    .fluid-hero-title {
        font-size: clamp(1.75rem, 4vw + 0.5rem, 3.25rem);
        line-height: 1.1;
    }

    .text-justify-inter-word {
        text-justify: inter-word;
        text-align: justify;
    }

    .scale-up-bg {
        animation: subtleScale 9s cubic-bezier(0.25, 1, 0.5, 1) forwards;
    }

    @keyframes subtleScale {
        from { transform: scale(1.03); }
        to { transform: scale(1); }
    }

    .item-stagger {
        opacity: 0;
        transform: translateY(10px);
        animation: smoothFloatIn 0.5s cubic-bezier(0.215, 0.610, 0.355, 1) forwards;
        animation-delay: calc(var(--delay) * 75ms);
    }

    @keyframes smoothFloatIn {
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .ambient-overlay {
        background: linear-gradient(
                to right,
                var(--color-background) 0%,
                color-mix(in srgb, var(--color-background) 40%, transparent) 40%,
                transparent 80%
        ),
        linear-gradient(
                to top,
                var(--color-background) 0%,
                transparent 35%
        );
    }
</style>