<script lang="ts">
    import { BookOpen, FileText } from "lucide-svelte";
    import type { ContinueItem } from '@/api/progress/types';
    import type { ContentType } from '@/api/content/types';
    import { i18n } from "@/stores/i18n.svelte.js";
    import * as Carousel from "@/components/ui/carousel";
    import SmartImage from "@/components/SmartImage.svelte";
    import EpisodeCard from "@/components/card/EpisodeCard.svelte";
    import {
        continueProgress,
        continueUrl,
        displayTitle,
        episodeLabel,
        shouldBlurNsfw,
        useContinueItems,
        wideImage
    } from "@/utils/episode-rows.svelte.js";

    let {
        items,
        mode
    }: {
        items: ContinueItem[];
        mode: ContentType;
    } = $props();

    const rows = useContinueItems(() => items);
</script>

{#if rows.items.length > 0}
    <div class="space-y-2 md:space-y-4">
        <h2 class="text-lg md:text-2xl font-bold tracking-tight text-foreground px-1">
            {mode === 'anime' ? i18n.t("home.continue.continue_watching") : i18n.t("home.continue.continue_reading")}
        </h2>

        <Carousel.Root
                opts={{ align: "start", dragFree: true }}
                class="w-full px-1"
        >
            <Carousel.Content class="-ml-5 pt-2 pb-2 md:pb-6">
                {#each rows.items as item}
                    {@const title = displayTitle(item.titleI18n, item.title)}

                    <!-- Equalized column basis bounds for structural parity across formats -->
                    <Carousel.Item class="pl-5 {mode === 'anime' ? 'basis-[275px] sm:basis-[360px]' : 'basis-[155px] sm:basis-[185px]'}">
                        {#if mode === 'anime'}
                            {@const ep = episodeLabel(item.episode, item.unit?.title)}
                            <EpisodeCard
                                    href={continueUrl(item)}
                                    image={wideImage(item.unit?.thumbnailUrl || item.coverImage)}
                                    alt={title}
                                    blur={shouldBlurNsfw(item.nsfw)}
                                    nsfw={!!item.nsfw}
                                    series={title}
                                    episode={ep.number}
                                    episodeTitle={ep.title}
                                    progress={continueProgress(item)}
                                    placeholder={String(item.episode ?? '?')}
                            />
                        {:else}
                            {@const coverImg = wideImage(item.coverImage)}

                            <!-- Portrait cards -->
                            <a href={continueUrl(item)}
                               class="reader-card group flex flex-col gap-3 w-full focus-visible:outline-none"
                            >
                                <div class="relative w-full aspect-[2/3] overflow-hidden bg-muted/15 border border-border/30 rounded-sm shadow-sm">
                                    {#if coverImg}
                                        <SmartImage
                                                src={coverImg}
                                                alt={title}
                                                shouldBlur={shouldBlurNsfw(item.nsfw)}
                                                class="w-full h-full object-cover transition-all duration-500 group-hover:scale-[1.04] group-hover:brightness-110"
                                        />
                                    {:else}
                                        <div class="w-full h-full flex items-center justify-center bg-muted/20">
                                            {#if mode === 'novel'}
                                                <FileText class="size-6 text-muted-foreground/30" />
                                            {:else}
                                                <BookOpen class="size-6 text-muted-foreground/30" />
                                            {/if}
                                        </div>
                                    {/if}

                                    <div class="absolute top-2 left-2 bg-background/80 backdrop-blur-md text-muted-foreground font-mono text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-sm border border-border/20 flex items-center gap-1">
                                        {#if mode === 'novel'}
                                            <FileText class="size-2.5 text-primary" />
                                            <span>LN</span>
                                        {:else}
                                            <BookOpen class="size-2.5 text-primary" />
                                            <span>MANGA</span>
                                        {/if}
                                    </div>

                                    {#if item.nsfw}
                                        <div class="absolute top-2 right-2 bg-destructive text-white text-[9px] font-black uppercase px-1.5 py-0.5 rounded-sm tracking-wider">
                                            18+
                                        </div>
                                    {/if}

                                    <div class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/80 via-black/40 to-transparent pointer-events-none flex items-end p-2">
                                        <p class="text-[10px] text-white/90 font-black tracking-wide uppercase line-clamp-1">
                                            {i18n.t('home.continue.chapters', { num: item.chapter ?? "?" })}
                                        </p>
                                    </div>
                                </div>

                                <div class="flex flex-col gap-0.5 px-0.5">
                                    <p class="font-black text-sm text-foreground line-clamp-1 group-hover:text-primary transition-colors duration-200">
                                        {title}
                                    </p>
                                    <span class="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-wider">
                                        {i18n.t('home.continue.continue_reading')} →
                                    </span>
                                </div>
                            </a>
                        {/if}
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    </div>
{/if}