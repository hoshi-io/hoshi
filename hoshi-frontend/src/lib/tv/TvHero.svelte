<script lang="ts">
    import { fade } from 'svelte/transition';
    import { Play, Star } from 'lucide-svelte';
    import { i18n } from '@/stores/i18n.svelte.js';
    import {
        getCardTitle,
        getCardShouldBlur,
        getCardScore,
        type NormalizedCard
    } from '@/utils/normalize';
    import ListEditorButton from "@/components/ListEditorButton.svelte";

    let { items = [] }: { items: NormalizedCard[] } = $props();

    const DURATION = 10_000;

    let index = $state(0);
    const current = $derived(items.length ? items[index % items.length] : undefined);

    const title = $derived(current ? getCardTitle(current) : '');
    const shouldBlur = $derived(current ? getCardShouldBlur(current) : false);
    const score = $derived(current ? getCardScore(current) : null);

    $effect(() => {
        if (items.length <= 1) return;
        const t = setInterval(() => (index = (index + 1) % items.length), DURATION);
        return () => clearInterval(t);
    });

    // Bring the whole hero back into view when its button gets focus.
    function revealTop(e: FocusEvent) {
        (e.currentTarget as HTMLElement).closest('main')?.scrollTo({ top: 0, behavior: 'smooth' });
    }
</script>

{#if current}
    <!-- -mx/-mt cancel the layout's overscan padding so the backdrop bleeds to the edges.
         No trailer iframe, no blur filters: too heavy for TV SoCs. -->
    <div class="relative -mx-12 -mt-8 h-[60dvh] min-h-[300px] select-none overflow-hidden bg-background">
        {#key current.href}
            <div class="absolute inset-0" in:fade={{ duration: 500 }} out:fade={{ duration: 400 }}>
                {#if current.bannerImage}
                    <img
                            src={current.bannerImage}
                            alt=""
                            decoding="async"
                            class="size-full object-cover object-center opacity-75 {shouldBlur ? 'blur-2xl scale-110' : ''}"
                    />
                {:else if current.cover}
                    <img
                            src={current.cover}
                            alt=""
                            decoding="async"
                            class="size-full scale-110 object-cover object-center opacity-50 blur-xl"
                    />
                {/if}

                <div class="ambient-overlay absolute inset-0"></div>

                <div class="absolute inset-0 flex flex-col justify-end px-12 pb-10">
                    <div class="flex max-w-xl flex-col items-start">
                        <div class="mb-3 flex flex-wrap items-center gap-2 text-xs font-bold">
                            {#if current.contentTypeLabel}
                                <span class="rounded bg-primary/15 px-2 py-0.5 uppercase tracking-wider text-primary">
                                    {current.contentTypeLabel}
                                </span>
                            {/if}
                            {#if score}
                                <span class="flex items-center gap-1 rounded bg-emerald-500/15 px-2 py-0.5 text-emerald-400">
                                    <Star class="size-3 fill-current" /> {score}
                                </span>
                            {/if}
                            {#if current.year}
                                <span class="text-foreground/80">{current.year}</span>
                            {/if}
                            {#if current.episodeCount}
                                <span class="text-muted-foreground">
                                    • {i18n.t('home.hero.eps', { count: current.episodeCount })}
                                </span>
                            {/if}
                        </div>

                        <h1 class="mb-2 line-clamp-2 text-3xl font-black leading-tight tracking-tight text-foreground">
                            {title}
                        </h1>

                        <p class="mb-4 line-clamp-3 text-sm font-medium leading-relaxed text-muted-foreground">
                            {current.synopsis || i18n.t('home.hero.no_synopsis')}
                        </p>
                        <div class="flex items-center gap-3">
                            <a
                                    href={current.href}
                                    data-sn
                                    data-autofocus
                                    onfocus={revealTop}
                                    class="flex h-11 items-center gap-2 rounded-md bg-primary px-7 text-sm font-bold text-primary-foreground outline-none
                                   transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/60"
                            >
                                <Play class="size-4 fill-current" />
                                {i18n.t('home.hero.watch')}
                            </a>

                            <ListEditorButton
                                    cid={current.cid}
                                    title={title}
                                    contentType="anime"
                                    coverImage={current.cover}
                                    class="h-11 px-5 text-sm"
                            />
                        </div>

                    </div>
                </div>
            </div>
        {/key}

        {#if items.length > 1}
            <div class="absolute bottom-6 right-12 z-10 flex items-center gap-2" aria-hidden="true">
                {#each items as _, i}
                    <span class="block h-1.5 rounded-full transition-all duration-300 {i === index % items.length ? 'w-6 bg-primary' : 'w-1.5 bg-foreground/30'}"></span>
                {/each}
            </div>
        {/if}
    </div>
{/if}

<style>
    .ambient-overlay {
        background:
                linear-gradient(
                        to right,
                        var(--color-background) 0%,
                        color-mix(in srgb, var(--color-background) 40%, transparent) 40%,
                        transparent 80%
                ),
                linear-gradient(to top, var(--color-background) 0%, transparent 35%);
    }
</style>