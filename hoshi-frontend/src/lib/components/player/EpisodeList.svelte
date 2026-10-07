<script lang="ts">
    import { tick, untrack } from "svelte";
    import { X, Play } from "lucide-svelte";
    import type { WatchState } from "@/app/watch.svelte.js";
    import { progressApi } from "@/api/progress/progress";
    import type { AnimeProgress } from "@/api/progress/types";
    import { primaryMetadata } from "@/api/content/types";
    import { i18n } from "@/stores/i18n.svelte";
    import { appConfig } from "@/stores/config.svelte.js";
    import { layoutState } from "@/stores/layout.svelte.js";
    import * as Carousel from "@/components/ui/carousel";
    import SmartImage from "@/components/SmartImage.svelte";

    let {
        pageState,
        onClose,
    }: {
        pageState: WatchState;
        onClose?: () => void;
    } = $props();

    const isMobile = $derived(layoutState.isMobile);

    const shouldBlur = $derived(
        !!pageState.animeData?.content?.nsfw && !!appConfig.data?.general?.blurAdultContent,
    );

    /** simkl thumbnails: `_m.` is the small variant, `_w.` the wide one. */
    const wideThumb = (url: string | null | undefined) =>
        url ? url.replace("_m.", "_w.") : null;

    interface EpisodeItem {
        number: number;
        title: string | null;
        thumbnail: string | null;
        duration: number | null; // seconds
        releasedAt: string | null;
        absoluteNumber: number | null;
    }

    const items = $derived.by<EpisodeItem[]>(() => {
        const data = pageState.animeData;
        if (!data) return [];

        const units = new Map<number, NonNullable<typeof data.contentUnits>[number]>();
        for (const u of data.contentUnits ?? []) {
            if (u.contentType?.toLowerCase() !== "episode") continue;
            if (u.unitNumber >= 1) units.set(u.unitNumber, u);
        }

        const maxUnit = units.size ? Math.max(...units.keys()) : 0;
        const meta = primaryMetadata(data);
        const total = Math.max(meta?.epsOrChapters ?? 0, maxUnit, pageState.epNumber || 0);

        return Array.from({ length: total }, (_, i) => {
            const n = i + 1;
            const u = units.get(n);
            const generic = u?.title?.trim().toLowerCase() === `episode ${n}`;
            return {
                number: n,
                title: u?.title && !generic ? u.title : null,
                thumbnail: wideThumb(u?.thumbnailUrl),
                duration: u?.duration ?? null,
                releasedAt: u?.releasedAt ?? null,
                absoluteNumber:
                    u?.absoluteNumber != null && u.absoluteNumber !== n ? u.absoluteNumber : null,
            };
        });
    });

    /** Rich cards if any episode has real metadata, otherwise number chips. */
    const mode = $derived<"rich" | "grid">(
        items.some((i) => i.title || i.thumbnail) ? "rich" : "grid",
    );

    const chunkSize = $derived(mode === "grid" ? 100 : 50);
    const rangeCount = $derived(Math.ceil(items.length / chunkSize));
    const currentRange = $derived(Math.floor((pageState.epNumber - 1) / chunkSize));

    let manual = $state<{ range: number; ep: number } | null>(null);
    const activeRange = $derived(
        manual && manual.ep === pageState.epNumber ? manual.range : currentRange,
    );

    const visible = $derived(items.slice(activeRange * chunkSize, (activeRange + 1) * chunkSize));
    const currentIndex = $derived(pageState.epNumber - 1 - activeRange * chunkSize);

    function rangeLabel(r: number) {
        const start = r * chunkSize + 1;
        const end = Math.min((r + 1) * chunkSize, items.length);
        return `${start}–${end}`;
    }

    let progress = $state<Map<number, AnimeProgress>>(new Map());
    let progressToken = 0;

    async function loadProgress(cid: string) {
        const token = ++progressToken;
        try {
            const res = await progressApi.getContentProgress(cid);
            if (token !== progressToken) return;
            progress = new Map(res.animeProgress.map((p) => [p.episode, p]));
        } catch {
            /* progress is decoration, never block the list on it */
        }
    }

    $effect(() => {
        const cid = pageState.cid;
        void pageState.epNumber;
        if (cid) untrack(() => loadProgress(cid));
    });

    function fractionFor(n: number): number {
        if (n === pageState.epNumber) {
            return pageState.currentDuration > 0
                ? Math.min(pageState.currentTime / pageState.currentDuration, 1)
                : 0;
        }
        const p = progress.get(n);
        if (!p?.episodeDurationSeconds) return 0;
        return Math.min(p.timestampSeconds / p.episodeDurationSeconds, 1);
    }

    const isWatched = (n: number) => progress.get(n)?.completed ?? false;

    let scroller: HTMLElement | undefined = $state();$effect(() => {
        void pageState.epNumber;
        void activeRange;
        void items.length;
        tick().then(() => {
            scroller?.querySelector('[data-current="true"]')?.scrollIntoView({ block: "center" });
        });
    });

    // Mobile: Embla carousel.
    let api = $state<{ scrollTo: (index: number, jump?: boolean) => void } | undefined>();$effect(() => {
        const a = api;
        const idx = currentIndex;
        if (a && idx >= 0 && idx < visible.length) a.scrollTo(idx);
    });

    function select(n: number) {
        pageState.goToEpisode(n);
    }

    function formatDuration(seconds: number | null): string {
        if (!seconds || seconds <= 0) return "";
        return `${Math.round(seconds / 60)}m`;
    }

    function formatDate(iso: string | null): string {
        if (!iso) return "";
        const d = new Date(iso);
        return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString();
    }

    function tr(key: string, fallback: string): string {
        const v = i18n.t(key);
        return !v || v === key ? fallback : v;
    }

    const label = (item: EpisodeItem) =>
        item.title ?? i18n.t("watch.episode_number", { num: item.number });
</script>

{#snippet tabs()}
    <div class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto">
        {#each Array.from({ length: rangeCount }, (_, r) => r) as r (r)}
            <button
                    class="shrink-0 rounded-sm border px-2.5 py-0.5 text-xs transition-colors
                    {r === activeRange
                        ? 'border-primary/70 bg-foreground/15 text-foreground'
                        : 'border-border/30 text-muted-foreground hover:bg-foreground/10'}"
                    onclick={() => (manual = { range: r, ep: pageState.epNumber })}
            >
                {rangeLabel(r)}
            </button>
        {/each}
    </div>
{/snippet}

{#snippet card(item: EpisodeItem)}
    {@const current = item.number === pageState.epNumber}
    {@const frac = fractionFor(item.number)}
    {@const watched = isWatched(item.number)}
    <button
            data-current={current}
            class="group relative block aspect-video w-full overflow-hidden rounded-sm border bg-foreground/10 text-left transition-colors
            {current ? 'border-primary ring-1 ring-primary' : 'border-border/30 hover:border-white/40'}"
            onclick={() => select(item.number)}
    >
        {#if item.thumbnail}
            <div
                    class="h-full w-full transition-transform duration-500 ease-out group-hover:scale-105
                    {watched && !current ? 'opacity-50' : ''}"
            >
                <SmartImage src={item.thumbnail} {shouldBlur} class="h-full w-full" />
            </div>
        {:else}
            <div class="flex h-full w-full items-center justify-center text-3xl font-semibold text-white/40">
                {item.number}
            </div>
        {/if}

        {#if current}
            <div class="absolute inset-0 z-10 flex items-center justify-center bg-black/40">
                <div class="flex size-10 items-center justify-center rounded-full bg-primary/90 text-primary-foreground shadow-sm backdrop-blur-md">
                    <Play class="ml-1 size-5 fill-current" />
                </div>
            </div>
        {/if}

        <span class="absolute left-1.5 top-1.5 z-20 rounded-sm bg-black/65 px-1.5 py-0.5 text-[11px] font-semibold text-white">
            {item.number}{item.absoluteNumber ? ` · #${item.absoluteNumber}` : ""}
        </span>

        {#if watched}
            <span class="absolute right-1.5 top-1.5 z-20 rounded-sm bg-black/65 px-1.5 py-0.5 text-[11px] text-white">✓</span>
        {:else if formatDuration(item.duration)}
            <span class="absolute right-1.5 top-1.5 z-20 rounded-sm bg-black/65 px-1.5 py-0.5 text-[11px] text-white">
                {formatDuration(item.duration)}
            </span>
        {/if}

        <div class="absolute inset-x-0 bottom-0 z-20 bg-gradient-to-t from-black/90 via-black/60 to-transparent px-2.5 pb-2 pt-6">
            <p class="line-clamp-2 font-medium leading-snug text-white {isMobile ? 'text-xs sm:text-sm' : 'text-sm'}">
                {label(item)}
            </p>
            {#if !isMobile && formatDate(item.releasedAt)}
                <p class="mt-0.5 text-xs text-white/70">{formatDate(item.releasedAt)}</p>
            {/if}
        </div>

        {#if frac > 0 && !watched}
            <div class="absolute inset-x-0 bottom-0 z-20 h-1 bg-black/50">
                <div class="h-full bg-primary" style="width: {frac * 100}%"></div>
            </div>
        {/if}
    </button>
{/snippet}

{#snippet chip(item: EpisodeItem)}
    {@const current = item.number === pageState.epNumber}
    {@const frac = fractionFor(item.number)}
    {@const watched = isWatched(item.number)}
    <button
            data-current={current}
            class="relative flex aspect-square w-full items-center justify-center overflow-hidden rounded-sm border text-base font-semibold transition-colors
            {current
                ? 'border-primary bg-primary/10 text-primary ring-1 ring-primary'
                : watched
                    ? 'border-transparent bg-foreground/5 text-muted-foreground hover:bg-foreground/15'
                    : 'border-transparent bg-foreground/10 hover:bg-foreground/20'}"
            onclick={() => select(item.number)}
    >
        {#if current}
            <span class="absolute left-1.5 top-1.5 flex h-1.5 w-1.5">
                <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary opacity-75"></span>
                <span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-primary"></span>
            </span>
        {/if}

        <span class="relative z-10">{item.number}</span>

        {#if frac > 0 && !watched}
            <span class="absolute inset-x-0 bottom-0 z-10 h-0.5 bg-primary/70" style="width: {frac * 100}%"></span>
        {/if}
    </button>
{/snippet}

<div class="flex min-h-0 flex-col {isMobile ? 'gap-2 px-3 pb-2 pt-2' : 'h-full'}">

    {#if isMobile}
        <div class="flex items-center gap-2">
            {@render tabs()}
            {#if onClose}
                <button
                        class="shrink-0 rounded-full p-1 text-muted-foreground transition-colors hover:bg-foreground/10 hover:text-foreground"
                        onclick={onClose}
                        aria-label="Close"
                >
                    <X class="size-4" />
                </button>
            {/if}
        </div>
    {:else if rangeCount > 1}
        <div class="flex px-4 pb-2">{@render tabs()}</div>
    {/if}

    <!-- body -->
    {#if pageState.isLoadingMeta && items.length === 0}
        <p class="px-2 py-6 text-center text-sm text-muted-foreground">…</p>

    {:else if isMobile}
        {#key activeRange}
            <Carousel.Root
                    opts={{ align: "start", dragFree: true, startIndex: Math.max(0, currentIndex) }}
                    setApi={(a) => (api = a)}
                    class="w-full touch-pan-y select-none"
            >
                <Carousel.Content class="-ml-2">
                    {#each visible as item (item.number)}
                        <Carousel.Item class="pl-2 {mode === 'rich' ? 'basis-[160px]' : 'basis-12'}">
                            {#if mode === "rich"}
                                {@render card(item)}
                            {:else}
                                {@render chip(item)}
                            {/if}
                        </Carousel.Item>
                    {/each}
                </Carousel.Content>
            </Carousel.Root>
        {/key}

    {:else}
        <div
                bind:this={scroller}
                class="min-h-0 flex-1 overflow-y-auto px-2 py-3 [mask-image:linear-gradient(to_bottom,transparent,black_0.75rem,black_calc(100%-0.75rem),transparent)]"
        >
            {#if mode === "grid"}
                <div class="grid grid-cols-5 gap-2 px-2">
                    {#each visible as item (item.number)}
                        {@render chip(item)}
                    {/each}
                </div>
            {:else}
                <ul class="flex flex-col gap-3 px-2">
                    {#each visible as item (item.number)}
                        <li>{@render card(item)}</li>
                    {/each}
                </ul>
            {/if}
        </div>
    {/if}
</div>