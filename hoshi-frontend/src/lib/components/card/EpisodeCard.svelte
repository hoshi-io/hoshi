<script lang="ts">
    import { CheckCircle2, Play } from "lucide-svelte";
    import SmartImage from "@/components/SmartImage.svelte";

    let {
        href,
        image,
        alt,
        blur = false,
        nsfw = false,
        series = null,
        episode,
        episodeTitle = null,
        progress = 0,
        badge = null,
        placeholder = '?',
        stacked = false,
        tag = null,
        subtext = null,
        completed = false,
        highlight = false,
        playOnHover = false,
        nativeImg = false
    }: {
        href: string;
        image: string | null;
        alt: string;
        blur?: boolean;
        nsfw?: boolean;
        /** Series name line above the episode. Leave out inside a single series' grid. */
        series?: string | null;
        episode: string;
        episodeTitle?: string | null;
        /** 0-100, hidden when 0 */
        progress?: number;
        /** Small label in the top-left corner (e.g. "3h ago", "24m") */
        badge?: string | null;
        /** Big faded text shown when there is no image */
        placeholder?: string;
        /** Grid layout: small episode label over a (2-line) title, instead of the inline row layout */
        stacked?: boolean;
        /** Accent text after the label, stacked layout only (e.g. "Resume") */
        tag?: string | null;
        /** Small mono line under the title, stacked layout only (e.g. "12:34 · 45%") */
        subtext?: string | null;
        /** Dims the image and shows a check */
        completed?: boolean;
        /** Ring around the thumbnail (e.g. the episode to resume) */
        highlight?: boolean;
        /** Shows a play button on hover */
        playOnHover?: boolean;
        /** Plain <img> instead of SmartImage (used by the episode grid) */
        nativeImg?: boolean;
    } = $props();
</script>

<a {href} class="group flex flex-col focus-visible:outline-none {stacked ? 'gap-2.5' : 'anime-card gap-4'}">
    <div class="relative w-full aspect-video overflow-hidden bg-muted/20 border border-border/30 rounded-sm {highlight ? 'ring-1 ring-primary/60' : ''}">
        {#if image}
            {#if nativeImg}
                <img
                        src={image}
                        {alt}
                        class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-[1.03] {completed ? 'opacity-40' : ''} {blur ? 'blur-xl scale-110' : ''}"
                />
            {:else}
                <SmartImage
                        src={image}
                        {alt}
                        shouldBlur={blur}
                        class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-[1.03] {completed ? 'opacity-40' : ''}"
                />
            {/if}
        {:else}
            <div class="w-full h-full flex items-center justify-center">
                <span class="text-6xl font-black text-muted-foreground/10 select-none">
                    {placeholder}
                </span>
            </div>
        {/if}

        <div class="absolute inset-x-0 bottom-0 h-1/3 bg-gradient-to-t from-black/60 to-transparent pointer-events-none"></div>

        {#if playOnHover}
            <div class="absolute inset-0 flex items-center justify-center bg-black/0 group-hover:bg-black/30 transition-colors duration-200">
                <div class="w-9 h-9 rounded-full bg-white/90 flex items-center justify-center scale-75 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all duration-200">
                    <Play class="w-4 h-4 text-black fill-black ml-0.5" />
                </div>
            </div>
        {/if}

        {#if completed}
            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                <CheckCircle2 class="w-7 h-7 text-primary/80" />
            </div>
        {/if}

        {#if nsfw}
            <div class="absolute top-2 right-2 bg-destructive text-white text-[10px] font-black uppercase px-2 py-0.5 rounded-sm tracking-wider">
                18+
            </div>
        {/if}

        {#if badge}
            <div class="absolute top-2 left-2 bg-background/80 backdrop-blur-md text-muted-foreground font-mono text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-sm border border-border/20">
                {badge}
            </div>
        {/if}

        {#if progress > 0}
            <div class="absolute bottom-0 left-0 right-0 h-[4px] bg-white/10">
                <div
                        class="h-full bg-primary transition-all shadow-[0_0_8px_color-mix(in_srgb,var(--primary)_60%,transparent)]"
                        style="width: {progress}%"
                ></div>
            </div>
        {/if}
    </div>

    {#if stacked}
        <div class="space-y-0.5 px-0.5">
            <div class="flex items-center gap-2">
                <p class="text-[11px] font-bold uppercase tracking-wide text-muted-foreground/60">
                    {episode}
                </p>
                {#if tag}
                    <span class="text-[10px] font-bold uppercase tracking-wide text-primary/80">· {tag}</span>
                {/if}
            </div>
            {#if episodeTitle}
                <p class="text-sm font-semibold leading-snug line-clamp-2 group-hover:text-primary transition-colors">
                    {episodeTitle}
                </p>
            {/if}
            {#if subtext}
                <p class="text-[10px] text-muted-foreground/40 font-mono tabular-nums pt-0.5">
                    {subtext}
                </p>
            {/if}
        </div>
    {:else}
        <div class="flex flex-col gap-1 px-1">
            {#if series}
                <p class="text-[11px] font-bold text-muted-foreground/60 uppercase tracking-widest truncate">
                    {series}
                </p>
            {/if}
            <div class="flex items-baseline gap-2 overflow-hidden">
                <span class="font-black text-base md:text-sm whitespace-nowrap group-hover:text-primary transition-colors">
                    {episode}
                </span>
                {#if episodeTitle}
                    <span class="text-muted-foreground/40 text-xs">.</span>
                    <span class="text-sm md:text-base font-semibold text-muted-foreground line-clamp-1">
                        {episodeTitle}
                    </span>
                {/if}
            </div>
        </div>
    {/if}
</a>