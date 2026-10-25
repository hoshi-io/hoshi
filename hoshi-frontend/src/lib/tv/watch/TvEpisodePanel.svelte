<script lang="ts">
    import { CheckCircle2 } from 'lucide-svelte';
    import type { WatchState } from '@/app/watch.svelte.js';
    import { progressApi } from '@/api/progress/progress';
    import type { AnimeProgress } from '@/api/progress/types';
    import { primaryMetadata } from '@/api/content/types';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import SmartImage from '@/components/SmartImage.svelte';

    let {
        pageState,
        onselect
    }: {
        pageState: WatchState;
        onselect: () => void;
    } = $props();

    const CHUNK = 25;

    const shouldBlur = $derived(
        !!pageState.animeData?.content?.nsfw && !!appConfig.data?.general?.blurAdultContent
    );

    const wideThumb = (url: string | null | undefined) => (url ? url.replace('_m.', '_w.') : null);

    type Item = { number: number; title: string | null; thumbnail: string | null };

    const items = $derived.by<Item[]>(() => {
        const data = pageState.animeData;
        if (!data) return [];

        const units = new Map<number, NonNullable<typeof data.contentUnits>[number]>();
        for (const u of data.contentUnits ?? []) {
            if (u.contentType?.toLowerCase() !== 'episode') continue;
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
                thumbnail: wideThumb(u?.thumbnailUrl)
            };
        });
    });

    const rangeCount = $derived(Math.ceil(items.length / CHUNK));
    const currentRange = $derived(Math.max(0, Math.floor((pageState.epNumber - 1) / CHUNK)));
    let manualRange = $state<number | null>(null);
    const activeRange = $derived(manualRange ?? currentRange);
    const visible = $derived(items.slice(activeRange * CHUNK, (activeRange + 1) * CHUNK));

    const rangeLabel = (r: number) => `${r * CHUNK + 1}–${Math.min((r + 1) * CHUNK, items.length)}`;

    let progress = $state(new Map<number, AnimeProgress>());

    $effect(() => {
        const cid = pageState.cid;
        if (!cid) return;
        let cancelled = false;
        progressApi
            .getContentProgress(cid)
            .then((res) => {
                if (!cancelled) progress = new Map(res.animeProgress.map((p) => [p.episode, p]));
            })
            .catch(() => {
                // progress is decoration only
            });
        return () => {
            cancelled = true;
        };
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

    function pick(n: number) {
        onselect();
        pageState.goToEpisode(n);
    }
</script>

{#if rangeCount > 1}
    <div class="-mx-1 mb-4 flex gap-2 overflow-x-auto px-1 py-2 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {#each Array.from({ length: rangeCount }, (_, r) => r) as r (r)}
            <button
                    type="button"
                    data-sn
                    aria-pressed={r === activeRange}
                    onclick={() => (manualRange = r)}
                    class="h-11 shrink-0 rounded-lg px-4 text-sm font-bold tabular-nums outline-none
                           focus:bg-white focus:text-black focus:ring-4 focus:ring-primary/60
                           {r === activeRange ? 'bg-primary text-primary-foreground' : 'bg-white/10 text-white'}"
            >
                {rangeLabel(r)}
            </button>
        {/each}
    </div>
{/if}

{#if pageState.isLoadingMeta && items.length === 0}
    <p class="py-8 text-center text-white/60">…</p>
{:else}
    <ul class="flex flex-col gap-2">
        {#each visible as item (item.number)}
            {@const current = item.number === pageState.epNumber}
            {@const frac = fractionFor(item.number)}
            {@const watched = progress.get(item.number)?.completed ?? false}
            <li>
                <button
                        type="button"
                        data-sn
                        data-current={current}
                        onclick={() => pick(item.number)}
                        class="flex w-full items-center gap-4 rounded-lg p-3 text-left outline-none
                               focus:bg-white/15 focus:ring-4 focus:ring-primary
                               {current ? 'bg-primary/20' : 'bg-white/5'}"
                >
                    <div class="relative aspect-video w-40 shrink-0 overflow-hidden rounded-md bg-white/10">
                        {#if item.thumbnail}
                            <SmartImage
                                    src={item.thumbnail}
                                    alt=""
                                    {shouldBlur}
                                    class="size-full object-cover {watched && !current ? 'opacity-40' : ''}"
                            />
                        {:else}
                            <div class="flex size-full items-center justify-center">
                                <span class="text-3xl font-black text-white/20">{item.number}</span>
                            </div>
                        {/if}
                        {#if watched}
                            <div class="absolute inset-0 flex items-center justify-center">
                                <CheckCircle2 class="size-8 text-primary/90" />
                            </div>
                        {/if}
                        {#if frac > 0 && !watched}
                            <div class="absolute inset-x-0 bottom-0 h-1.5 bg-black/50">
                                <div class="h-full bg-primary" style="width: {frac * 100}%"></div>
                            </div>
                        {/if}
                    </div>

                    <div class="min-w-0 flex-1 space-y-0.5">
                        <p class="text-xs font-bold uppercase tracking-wide text-white/60">
                            {i18n.t('content.episode_title', { num: item.number })}
                        </p>
                        {#if item.title}
                            <p class="line-clamp-2 text-base font-semibold leading-snug">{item.title}</p>
                        {/if}
                    </div>
                </button>
            </li>
        {/each}
    </ul>
{/if}