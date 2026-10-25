<script lang="ts">
    import SmartImage from '@/components/SmartImage.svelte';

    let {
        href,
        image = null,
        alt = '',
        blur = false,
        nsfw = false,
        series,
        episode,
        episodeTitle = null,
        badge = null,
        progress = 0,
        placeholder = '?'
    }: {
        href: string;
        image?: string | null;
        alt?: string;
        blur?: boolean;
        nsfw?: boolean;
        series: string;
        episode: string;
        episodeTitle?: string | null;
        badge?: string | null;
        progress?: number;
        placeholder?: string;
    } = $props();
</script>

<!-- Landscape (16:9) card shared by "continue watching" and "recently aired". -->
<a {href} data-sn class="group block w-64 shrink-0 scroll-my-16 outline-none">
    <div
            class="relative aspect-video overflow-hidden rounded-md border border-border/30 bg-muted/20
               transition-transform duration-150
               group-focus:scale-105 group-focus:ring-4 group-focus:ring-primary"
    >
        {#if image}
            <div class="absolute inset-0">
                <SmartImage src={image} {alt} shouldBlur={blur} class="size-full object-cover" />
            </div>
        {:else}
            <div class="flex size-full items-center justify-center">
                <span class="select-none text-5xl font-black text-muted-foreground/10">{placeholder}</span>
            </div>
        {/if}

        <div class="pointer-events-none absolute inset-x-0 bottom-0 h-1/3 bg-gradient-to-t from-black/60 to-transparent"></div>

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

    <div class="mt-2 space-y-0.5 px-0.5">
        <p class="truncate text-xs font-bold uppercase tracking-widest text-muted-foreground/70">{series}</p>
        <div class="flex items-baseline gap-2 overflow-hidden">
            <span class="whitespace-nowrap text-sm font-black group-focus:text-primary">{episode}</span>
            {#if episodeTitle}
                <span class="line-clamp-1 text-sm font-semibold text-muted-foreground">{episodeTitle}</span>
            {/if}
        </div>
    </div>
</a>