<script lang="ts">
    import { tick } from "svelte";
    import { NovelReaderState } from "@/app/novel.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import NovelReaderSettings from "@/components/readers/NovelReaderSettings.svelte";
    import { fontStack } from "@/fonts";
    import { chapterLabel } from "@/utils/chapterTitle";

    const readerState = new NovelReaderState();

    const progress = $derived(
        readerState.paged
            ? {
                value: (readerState.page + 1) / readerState.pageCount,
                label: `${readerState.page + 1} / ${readerState.pageCount}`,
                pages: readerState.pageCount,
                seek: (r: number) => readerState.goToPage(Math.floor(r * readerState.pageCount)),
            }
            : {
                value: (readerState.scrollProgress ?? 0) / 100,
                label: `${Math.round(readerState.scrollProgress ?? 0)}%`,
                seek: (r: number) => readerState.scrollToPercentage(r * 100),
            }
    );

    const MIN_COLUMN = 320;
    const PAD_Y = 40;
    /** Mobile: room for the floating page bar. */
    const PAD_BOTTOM_MOBILE = 88;

    let innerWidth = $state(0);
    /** Desktop: the auto-hiding header (titlebar 32 + bar 56) gets its own strip, so it never covers text. */
    const PAD_TOP_DESKTOP = 88 + 24;
    const padBottom = $derived(innerWidth < 1024 ? PAD_BOTTOM_MOBILE : PAD_Y);
    const padTop = $derived(innerWidth < 1024 ? PAD_Y : PAD_TOP_DESKTOP);
    const desktop = $derived(innerWidth >= 1024);
    const runningTitle = $derived(chapterLabel(readerState.currentChapter, readerState.chapterNumber).title);


    // ---- paged geometry ----
    let mainW = $state(0);
    let mainH = $state(0);
    let fontsVersion = $state(0);

    const gap = $derived(Math.max(32, readerState.margin * 2));
    const available = $derived(Math.max(0, mainW - readerState.margin * 2));
    const columns = $derived(readerState.pagesPerView >= 2 && available >= MIN_COLUMN * 2 + gap ? 2 : 1);
    const spreadWidth = $derived(Math.min(available, columns * readerState.maxWidth + (columns - 1) * gap));
    /** An explicit width keeps one-column mode paginated: WebKit drops multicol for `column-count: 1`. */
    const columnWidth = $derived(Math.max(1, (spreadWidth - (columns - 1) * gap) / columns));
    const pageHeight = $derived(Math.max(120, mainH - padTop - padBottom));

    const typography = $derived(`
        font-family: ${fontStack(readerState.fontFamily)};
        font-size: ${readerState.fontSize}px;
        line-height: ${readerState.lineHeight};
        text-align: ${readerState.textAlign};
        letter-spacing: ${readerState.letterSpacing}em;
        word-spacing: ${readerState.wordSpacing}em;
        --paragraph-spacing: ${readerState.paragraphSpacing}em;
    `);

    // Late-loading fonts change text width, so pages are re-measured once they arrive.
    $effect(() => {
        const bump = () => fontsVersion++;
        document.fonts.addEventListener("loadingdone", bump);
        return () => document.fonts.removeEventListener("loadingdone", bump);
    });

    $effect(() => {
        if (!readerState.paged) return;
        // Everything that reflows the text:
        void [readerState.contentHtml, typography, spreadWidth, pageHeight, columns, gap, fontsVersion];
        tick().then(() => readerState.relayout(spreadWidth, gap, columns));
    });

    // ---- paged input: tap/click zones, swipe, wheel ----
    let swipeStart: { x: number; y: number } | null = null;
    let swiped = false;
    let wheelLock = 0;

    function onPagedPointerDown(e: PointerEvent) {
        swiped = false;
        swipeStart = e.pointerType === "mouse" ? null : { x: e.clientX, y: e.clientY };
    }

    function onPagedPointerUp(e: PointerEvent) {
        if (!swipeStart) return;
        const dx = e.clientX - swipeStart.x;
        const dy = e.clientY - swipeStart.y;
        swipeStart = null;
        if (Math.abs(dx) > 50 && Math.abs(dx) > Math.abs(dy)) {
            swiped = true;
            dx < 0 ? readerState.nextPage() : readerState.prevPage();
        }
    }

    function onPagedClick(e: MouseEvent) {
        if (swiped) return;
        if ((e.target as HTMLElement).closest("a, button")) return;
        if (window.getSelection()?.toString()) return;
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const x = (e.clientX - rect.left) / rect.width;
        if (x < 0.3) readerState.prevPage();
        else if (x > 0.7) readerState.nextPage();
    }

    function onPagedWheel(e: WheelEvent) {
        const delta = Math.abs(e.deltaY) > Math.abs(e.deltaX) ? e.deltaY : e.deltaX;
        if (Math.abs(delta) < 4 || Date.now() < wheelLock) return;
        wheelLock = Date.now() + 350;
        delta > 0 ? readerState.nextPage() : readerState.prevPage();
    }

</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<svelte:window bind:innerWidth />

<Reader {readerState} contentType="novel" {progress}>
    {#snippet settings()}
        <NovelReaderSettings {readerState} />
    {/snippet}

    {#if readerState.paged}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
        <main
                id="novel-main-container"
                class="flex-1 min-h-0 overflow-hidden relative select-text transition-colors duration-300"
                style="background-color: {readerState.themeColors.bg}; color: {readerState.themeColors.text};"
                bind:clientWidth={mainW}
                bind:clientHeight={mainH}
                onclick={onPagedClick}
                onpointerdown={onPagedPointerDown}
                onpointerup={onPagedPointerUp}
                onwheel={onPagedWheel}
        >
            <div
                    id="novel-pager"
                    class="absolute left-1/2 -translate-x-1/2 overflow-hidden"
                    style="top: {padTop}px; height: {pageHeight}px; width: {spreadWidth}px;"
            >
                <article
                        id="novel-article"
                        class="novel-content novel-paged prose max-w-none h-full"
                        style="{typography} column-count: {columns}; column-width: {columnWidth}px; column-gap: {gap}px; column-fill: auto; --page-height: {pageHeight}px; transform: translateX(-{readerState.pageOffset}px);"
                >
                    {@html readerState.contentHtml}
                </article>
            </div>

            {#if desktop}
                <!-- Book-style running header and folio: the strip above the page belongs to the reader's top bar. -->
                <div
                        class="pointer-events-none absolute left-1/2 flex -translate-x-1/2 items-center justify-center gap-2 truncate text-xs tracking-wide opacity-45"
                        style="top: 32px; height: {padTop - 32}px; width: {spreadWidth}px;"
                >
                    <span class="min-w-0 truncate">{readerState.title}</span>
                    {#if runningTitle}<span class="opacity-60">·</span><span class="shrink-0">{runningTitle}</span>{/if}
                </div>
                <div
                        class="pointer-events-none absolute bottom-0 left-1/2 flex -translate-x-1/2 items-center justify-center font-mono text-xs tabular-nums opacity-45"
                        style="height: {padBottom}px; width: {spreadWidth}px;"
                >
                    {readerState.page + 1} / {readerState.pageCount}
                </div>
            {/if}

            {#if columns === 2}
                <div
                        class="pointer-events-none absolute left-1/2 w-px -translate-x-1/2 opacity-10"
                        style="top: {padTop}px; height: {pageHeight}px; background-color: {readerState.themeColors.text};"
                ></div>
            {/if}
        </main>
    {:else}
        <main
                id="novel-main-container"
                class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
                style="background-color: {readerState.themeColors.bg}; color: {readerState.themeColors.text};"
                onscroll={(e) => readerState.onScroll(e)}
        >
            <article
                    class="mx-auto py-8 md:py-12 lg:pt-28 transition-all duration-300"
                    style="{typography} max-width: {readerState.maxWidth + readerState.margin * 2}px; padding-left: {readerState.margin}px; padding-right: {readerState.margin}px;"
            >
                <div class="prose max-w-none novel-content">
                    {@html readerState.contentHtml}
                </div>
                <div class="h-24 w-full"></div>
            </article>
        </main>
    {/if}

</Reader>

<style>
    :global(.novel-content *) {
        text-align: inherit;
        line-height: inherit;
        color: inherit;
        font-family: inherit;
        letter-spacing: inherit;
        word-spacing: inherit;
    }
    :global(.novel-content p) {
        margin-top: 0;
        margin-bottom: var(--paragraph-spacing, 1.5em);
    }
    :global(.novel-paged img) {
        max-width: 100%;
        max-height: var(--page-height);
        object-fit: contain;
        break-inside: avoid;
    }
    :global(.novel-paged :is(h1, h2, h3, h4)) {
        break-after: avoid;
    }
    :global(.novel-paged > :first-child) {
        margin-top: 0;
    }
</style>
