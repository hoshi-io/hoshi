<script lang="ts">
    import { tick } from "svelte";
    import { NovelReaderState } from "@/app/novel.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import NovelReaderSettings from "@/components/readers/NovelReaderSettings.svelte";
    import { fontStack } from "@/fonts";

    const readerState = new NovelReaderState();

    const MIN_COLUMN = 320;
    const PAD_Y = 40;

    let trackEl: HTMLDivElement;
    let isDragging = $state(false);

    // ---- paged geometry ----
    let mainW = $state(0);
    let mainH = $state(0);
    let fontsVersion = $state(0);

    const gap = $derived(Math.max(32, readerState.margin * 2));
    const available = $derived(Math.max(0, mainW - readerState.margin * 2));
    const columns = $derived(readerState.pagesPerView >= 2 && available >= MIN_COLUMN * 2 + gap ? 2 : 1);
    const spreadWidth = $derived(Math.min(available, columns * readerState.maxWidth + (columns - 1) * gap));
    const pageHeight = $derived(Math.max(120, mainH - PAD_Y * 2));

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

    // ---- side slider ----
    function getProgressFromPointer(e: PointerEvent): number {
        const rect = trackEl.getBoundingClientRect();
        const ratio = (e.clientY - rect.top) / rect.height;
        return Math.min(100, Math.max(0, ratio * 100));
    }

    function onPointerDown(e: PointerEvent) {
        e.preventDefault();
        isDragging = true;
        trackEl.setPointerCapture(e.pointerId);
        readerState.scrollToPercentage(getProgressFromPointer(e));
    }

    function onPointerMove(e: PointerEvent) {
        if (!isDragging) return;
        readerState.scrollToPercentage(getProgressFromPointer(e));
    }

    function onPointerUp(e: PointerEvent) {
        isDragging = false;
        trackEl.releasePointerCapture(e.pointerId);
    }
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<Reader {readerState} contentType="novel">
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
                    style="top: {PAD_Y}px; height: {pageHeight}px; width: {spreadWidth}px;"
            >
                <article
                        class="novel-content novel-paged prose max-w-none h-full"
                        style="{typography} column-count: {columns}; column-gap: {gap}px; column-fill: auto; --page-height: {pageHeight}px;"
                >
                    {@html readerState.contentHtml}
                </article>
            </div>

            {#if columns === 2}
                <div
                        class="pointer-events-none absolute left-1/2 w-px -translate-x-1/2 opacity-10"
                        style="top: {PAD_Y}px; height: {pageHeight}px; background-color: {readerState.themeColors.text};"
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
                    class="mx-auto py-8 md:py-12 transition-all duration-300"
                    style="{typography} max-width: {readerState.maxWidth + readerState.margin * 2}px; padding-left: {readerState.margin}px; padding-right: {readerState.margin}px;"
            >
                <div class="prose max-w-none novel-content">
                    {@html readerState.contentHtml}
                </div>
                <div class="h-24 w-full"></div>
            </article>
        </main>
    {/if}

    {#if !readerState.isLoading && !readerState.error}
        <div class="fixed right-3 top-1/2 -translate-y-1/2 z-40 flex flex-col items-center gap-3 select-none">
            <div
                    bind:this={trackEl}
                    class="relative flex items-center justify-center cursor-pointer"
                    style="width: 28px; height: 220px;"
                    onpointerdown={onPointerDown}
                    onpointermove={onPointerMove}
                    onpointerup={onPointerUp}
                    onpointercancel={onPointerUp}
                    role="slider"
                    aria-valuenow={Math.round(readerState.scrollProgress ?? 0)}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    tabindex="0"
                    onkeydown={(e) => {
                    const step = e.shiftKey ? 10 : 2;
                    if (e.key === 'ArrowDown') readerState.scrollToPercentage(Math.min(100, (readerState.scrollProgress ?? 0) + step));
                    if (e.key === 'ArrowUp') readerState.scrollToPercentage(Math.max(0, (readerState.scrollProgress ?? 0) - step));
                }}
            >
                <div class="absolute inset-x-0 mx-auto rounded-full bg-foreground/15" style="width: 6px; height: 100%;"></div>
                <div
                        class="absolute top-0 inset-x-0 mx-auto rounded-full bg-primary/60 transition-[height] duration-100 pointer-events-none"
                        style="width: 6px; height: {readerState.scrollProgress ?? 0}%;"
                ></div>
                <div
                        class="absolute inset-x-0 mx-auto rounded-full bg-primary shadow-lg border-2 border-background pointer-events-none transition-[top] duration-100"
                        class:scale-125={isDragging}
                        style="width: 18px; height: 18px; top: calc({readerState.scrollProgress ?? 0}% - 9px);"
                ></div>
            </div>
            <span class="text-xs font-mono font-semibold tabular-nums" style="color: {readerState.themeColors.text}; opacity: 0.5;">
                {#if readerState.paged}
                    {readerState.page + 1}/{readerState.pageCount}
                {:else}
                    {Math.round(readerState.scrollProgress ?? 0)}%
                {/if}
            </span>
        </div>
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
