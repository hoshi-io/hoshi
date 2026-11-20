<script lang="ts">
    import { tick } from "svelte";
    import { Input } from "@/components/ui/input";
    import { Button } from "@/components/ui/button";
    import Search from 'phosphor-svelte/lib/MagnifyingGlassIcon';
    import SortAsc from 'phosphor-svelte/lib/SortAscendingIcon';
    import SortDesc from 'phosphor-svelte/lib/SortDescendingIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { BaseReaderState } from "@/app/reader.svelte";
    import { chapterNum, scanlatorLabel, type ChapterLike } from "@/utils/chapterVariants";
    import { chapterLabel } from "@/utils/chapterTitle";

    let { readerState, onSelect }: { readerState: BaseReaderState; onSelect?: () => void } = $props();

    /** Fixed row height: the list only renders the rows in view. */
    const ROW = 60;
    const OVERSCAN = 8;

    let query = $state("");
    let descending = $state(false);
    let scrollEl = $state<HTMLDivElement>();
    let scrollTop = $state(0);
    let viewH = $state(0);

    type Row = { chap: ChapterLike; num: number; title: string; subtitle: string | null; meta: string | null; search: string };

    const rows = $derived.by((): Row[] => {
        const out = readerState.rawChapters.map((chap) => {
            const num = chapterNum(chap);
            const { title, subtitle } = chapterLabel(chap, num);
            const group = scanlatorLabel(chap);
            const lang = chap.language ? chap.language.toUpperCase() : null;
            const meta = [lang, group].filter(Boolean).join(" · ") || null;
            return { chap, num, title, subtitle, meta, search: `${num} ${title} ${subtitle ?? ""} ${meta ?? ""}`.toLowerCase() };
        });
        return descending ? out.reverse() : out;
    });

    const filtered = $derived.by(() => {
        const q = query.trim().toLowerCase();
        return q ? rows.filter((r) => r.search.includes(q)) : rows;
    });

    const start = $derived(Math.max(0, Math.floor(scrollTop / ROW) - OVERSCAN));
    const end = $derived(Math.min(filtered.length, Math.ceil((scrollTop + viewH) / ROW) + OVERSCAN));
    const visible = $derived(filtered.slice(start, end));

    const currentIndex = $derived(filtered.findIndex((r) => r.chap === readerState.currentChapter));

    // Open centred on the chapter being read.
    let centred = false;
    $effect(() => {
        if (centred || !scrollEl || viewH === 0 || currentIndex < 0) return;
        centred = true;
        tick().then(() => {
            if (scrollEl) scrollEl.scrollTop = Math.max(0, currentIndex * ROW - viewH / 2 + ROW / 2);
        });
    });

    $effect(() => {
        query; descending;
        if (scrollEl) scrollEl.scrollTop = 0;
    });
</script>

<div class="flex h-full min-h-0 flex-col">
    <div class="flex shrink-0 items-center gap-2 pb-3">
        <div class="relative min-w-0 flex-1">
            <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
                    bind:value={query}
                    placeholder={i18n.t('reader.search_chapters')}
                    class="h-9 rounded-sm bg-background pl-9 text-sm"
            />
        </div>
        <Button
                variant="outline"
                size="icon"
                class="size-9 shrink-0 rounded-sm"
                aria-label={i18n.t(descending ? 'reader.sort_oldest' : 'reader.sort_newest')}
                title={i18n.t(descending ? 'reader.sort_oldest' : 'reader.sort_newest')}
                onclick={() => (descending = !descending)}
        >
            {#if descending}<SortDesc class="size-4" />{:else}<SortAsc class="size-4" />{/if}
        </Button>
    </div>

    <p class="shrink-0 pb-2 text-xs text-muted-foreground">
        {i18n.t('reader.chapters_available', { count: readerState.allChapters.length })}
    </p>

    <div
            bind:this={scrollEl}
            bind:clientHeight={viewH}
            class="min-h-0 flex-1 overflow-y-auto overscroll-contain"
            onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
    >
        {#if filtered.length === 0}
            <p class="py-10 text-center text-sm text-muted-foreground">{i18n.t('error.content.no_match_found')}</p>
        {:else}
            <div class="relative" style="height: {filtered.length * ROW}px;">
                {#each visible as row, i (row.chap.id ?? `${row.num}-${start + i}`)}
                    {@const current = row.chap === readerState.currentChapter}
                    <a
                            href={readerState.chapterHref(row.chap)}
                            onclick={() => onSelect?.()}
                            aria-current={current ? "page" : undefined}
                            class="absolute inset-x-0 flex items-center gap-3 rounded-sm px-2 transition-colors
                                   {current ? 'bg-primary/10' : 'hover:bg-muted/60'}"
                            style="top: {(start + i) * ROW}px; height: {ROW - 4}px;"
                    >
                        <span
                                class="flex h-9 min-w-9 shrink-0 items-center justify-center rounded-sm border px-1.5 font-mono text-xs font-bold tabular-nums
                                       {current ? 'border-primary bg-primary text-primary-foreground' : 'border-border/50 bg-muted/40 text-muted-foreground'}"
                        >
                            {row.num}
                        </span>
                        <span class="min-w-0 flex-1">
                            <span class="block truncate text-sm font-semibold {current ? 'text-primary' : ''}">{row.title}</span>
                            {#if row.subtitle || row.meta}
                                <span class="block truncate text-xs text-muted-foreground">
                                    {row.subtitle ?? ''}{#if row.subtitle && row.meta} · {/if}{row.meta ?? ''}
                                </span>
                            {/if}
                        </span>
                    </a>
                {/each}
            </div>
        {/if}
    </div>
</div>
