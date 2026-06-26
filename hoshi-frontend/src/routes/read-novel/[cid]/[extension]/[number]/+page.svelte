<script lang="ts">
    import { NovelReaderState, NOVEL_THEMES } from "@/app/novel.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Type, AlignLeft, AlignJustify, Palette, Expand, Baseline, Space } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { NovelTheme, FontFamily } from "@/api/config/types";
    import NovelReaderSettings from "@/components/readers/NovelReaderSettings.svelte";

    const readerState = new NovelReaderState();

    let trackEl: HTMLDivElement;
    let isDragging = $state(false);

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

    <main
            id="novel-main-container"
            class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
            style="background-color: {readerState.themeColors.bg}; color: {readerState.themeColors.text};"
            onscroll={(e) => readerState.onScroll(e)}
    >
        <article
                class="mx-auto px-4 py-8 md:py-12 transition-all duration-300 {readerState.fontFamily === 'sans' ? 'font-sans' : readerState.fontFamily === 'serif' ? 'font-serif' : 'font-mono'}"
                style="
                max-width: {readerState.maxWidth}px;
                font-size: {readerState.fontSize}px;
                line-height: {readerState.lineHeight};
                text-align: {readerState.textAlign};
                --paragraph-spacing: {readerState.paragraphSpacing}em;
            "
        >
            <div class="prose max-w-none novel-content" style="color: inherit; text-align: inherit; line-height: inherit;">
                {@html readerState.contentHtml}
            </div>
            <div class="h-24 w-full"></div>
        </article>
    </main>

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
                {Math.round(readerState.scrollProgress ?? 0)}%
            </span>
        </div>
    {/if}
</Reader>

<style>
    :global(.novel-content *) {
        text-align: inherit;
        line-height: inherit;
        color: inherit;
    }
    :global(.novel-content p) {
        margin-bottom: var(--paragraph-spacing, 1.5em);
    }
</style>