<script lang="ts">
    import { CheckCircle2 } from 'lucide-svelte';
    import SmartImage from '@/components/SmartImage.svelte';

    let {
        href,
        image = null,
        alt = '',
        blur = false,
        nsfw = false,
        series = null,
        episode,
        episodeTitle = null,
        badge = null,
        progress = 0,
        placeholder = '?',
        stacked = false,
        tag = null,
        completed = false,
        highlight = false,
        nativeImg = false
    }: {
        href: string;
        image?: string | null;
        alt?: string;
        blur?: boolean;
        nsfw?: boolean;
        /** Series name line above the episode. Leave out inside a single series' grid. */
        series?: string | null;
        episode: string;
        episodeTitle?: string | null;
        badge?: string | null;
        /** 0-100, hidden when 0 */
        progress?: number;
        placeholder?: string;
        /** Grid layout: fills its cell, small label over a (2-line) title. Default is the fixed-width row card. */
        stacked?: boolean;
        /** Accent text after the label, stacked layout only (e.g. "Resume") */
        tag?: string | null;
        /** Dims the image and shows a check */
        completed?: boolean;
        /** Ring around the thumbnail (e.g. the episode to resume) */
        highlight?: boolean;
        /** Plain lazy <img> instead of SmartImage (used by the episode grid) */
        nativeImg?: boolean;
    } = $props();
</script>

<!-- Landscape (16:9) card. Default: fixed-width card for rows ("continue watching", "recently aired").
     `stacked`: fills its grid cell (episode grids on the content page). -->
<a {href} data-sn class="group block scroll-my-16 outline-none {stacked ? 'w-full' : 'w-64 shrink-0'}">
    <div
            class="relative aspect-video overflow-hidden rounded-md border border-border/30 bg-muted/20
               transition-transform duration-150
               group-focus:scale-105 group-focus:ring-4 group-focus:ring-primary
               {highlight ? 'ring-2 ring-primary/50' : ''}"
    >
        {#if image}
            {#if nativeImg}
                <img
                        src={image}
                        {alt}
                        loading="lazy"
                        class="size-full object-cover {completed ? 'opacity-40' : ''}"
                />
            {:else}
                <div class="absolute inset-0">
                    <SmartImage
                            src={image}
                            {alt}
                            shouldBlur={blur}
                            class="size-full object-cover {completed ? 'opacity-40' : ''}"
                    />
                </div>
            {/if}
        {:else}
            <div class="flex size-full items-center justify-center">
                <span class="select-none text-5xl font-black text-muted-foreground/10">{placeholder}</span>
            </div>
        {/if}

        <div class="pointer-events-none absolute inset-x-0 bottom-0 h-1/3 bg-gradient-to-t from-black/60 to-transparent"></div>

        {#if completed}
            <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
                <CheckCircle2 class="size-9 text-primary/80" />
            </div>
        {/if}

        {#if badge}
            <div class="absolute left-1.5 top-1.5 rounded bg-black/60 px-1.5 py-0.5 font-mono text-[10px] font-bold uppercase tracking-wider text-white">
                {badge}
            </div>
        {/if}

        {#if nsfw}
            <div class="absolute right-1.5 top-1.5 rounded bg-destructive px-1.5 py-0.5 text-[10px] font-black uppercase tracking-wider text-white">
                18+
            </div>
        {/if}

        {#if progress > 0}
            <div class="absolute inset-x-0 bottom-0 h-1 bg-white/10">
                <div class="h-full bg-primary" style="width: {progress}%"></div>
            </div>
        {/if}
    </div>

    {#if stacked}
        <div class="mt-2 space-y-0.5 px-0.5">
            <p class="text-xs font-bold uppercase tracking-wide text-muted-foreground/70">
                {episode}
                {#if tag}
                    <span class="text-primary">· {tag}</span>
                {/if}
            </p>
            {#if episodeTitle}
                <p class="line-clamp-2 text-sm font-semibold leading-snug group-focus:text-primary">{episodeTitle}</p>
            {/if}
        </div>
    {:else}
        <div class="mt-2 space-y-0.5 px-0.5">
            {#if series}
                <p class="truncate text-xs font-bold uppercase tracking-widest text-muted-foreground/70">{series}</p>
            {/if}
            <div class="flex items-baseline gap-2 overflow-hidden">
                <span class="whitespace-nowrap text-sm font-black group-focus:text-primary">{episode}</span>
                {#if episodeTitle}
                    <span class="line-clamp-1 text-sm font-semibold text-muted-foreground">{episodeTitle}</span>
                {/if}
            </div>
        </div>
    {/if}
</a>