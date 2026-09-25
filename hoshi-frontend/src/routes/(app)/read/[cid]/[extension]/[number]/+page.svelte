<script lang="ts">
    import { MangaReaderState } from "@/app/manga.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import { Slider } from "@/components/ui/slider";
    import { i18n } from "@/stores/i18n.svelte";
    import { fly } from "svelte/transition";
    import ReaderImage from "@/components/readers/ReaderImage.svelte";
    import MangaReaderSettings from "@/components/readers/MangaReaderSettings.svelte";
    import {cubicOut} from "svelte/easing";
    import {layoutState} from "@/stores/layout.svelte";

    const readerState = new MangaReaderState();
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<Reader
        {readerState}
        contentType="manga"
        currentProgress={readerState.layout === "paged" ? readerState.currentProgress : null}
>
    {#snippet settings()}
        <MangaReaderSettings {readerState} />
    {/snippet}

    <main
            id="reader-main-container"
            role="presentation"
            class="safe-reader flex-1 relative bg-muted/10
            {readerState.layout === 'scroll' ? 'overflow-y-auto overflow-x-hidden' : 'overflow-hidden'}"
            onclick={(e) => readerState.handleZoneClick(e)}
            onkeydown={(e) => {
                if (e.key === 'ArrowRight') readerState.turnPage('next');
                if (e.key === 'ArrowLeft') readerState.turnPage('prev');
            }}
            ontouchstart={(e) => readerState.handleTouchStart(e)}
            ontouchend={(e) => readerState.handleTouchEnd(e)}
    >
        {#if readerState.layout === "scroll"}
            <div
                    class="flex flex-col items-center w-full py-4 pb-24"
                    style="row-gap: {readerState.gapY}px;"
            >
                {#each readerState.groupedImages as group}
                    <div
                            class="scroll-row flex items-start w-full"
                            class:row-fit-height={readerState.fitMode === 'height'}
                            style="column-gap: {readerState.gapX}px;"
                    >
                        {#if group[0]}
                            <ReaderImage imgEntry={group[0]} {readerState} />
                        {/if}
                        {#if group[1]}
                            <ReaderImage imgEntry={group[1]} {readerState} />
                        {/if}
                    </div>
                {/each}
            </div>

        {:else}
            <!--
                Paged layout: position:relative wrapper + absolute inset-0 pages.
                This gives ReaderImage a concrete h-full to fill, and overflow-hidden
                clips transitioning pages so they can't expand the container.
            -->
            <div class="relative w-full h-full overflow-hidden">
                {#key readerState.currentGroupIndex}
                    {@const group = readerState.groupedImages[readerState.currentGroupIndex]}
                    <div
                            class="absolute inset-0 flex items-center justify-center py-3 px-2 md:px-6"
                            style="column-gap: {readerState.gapX}px;"
                            in:fly={{ x: `${readerState.animDir * 100}%`, duration: readerState.skipAnimation ? 0 : 280, opacity: 1 }}
                            out:fly={{ x: `${-readerState.animDir * 100}%`, duration: readerState.skipAnimation ? 0 : 280, opacity: 1 }}
                    >
                        {#if group}
                            {#if group[0]}
                                <ReaderImage imgEntry={group[0]} {readerState} />
                            {/if}
                            {#if group[1]}
                                <ReaderImage imgEntry={group[1]} {readerState} />
                            {/if}
                        {/if}
                    </div>
                {/key}
            </div>
        {/if}
    </main>

    {#if readerState.layout === "paged" && readerState.showOverlay && readerState.groupedImages.length > 0 && layoutState.isMobile && !readerState.showSettings}
        <div
                class="fixed bottom-6 left-6 right-6 md:left-1/2 md:right-auto md:w-[420px] md:-translate-x-1/2 z-[100] pb-[env(safe-area-inset-bottom)]"
                transition:fly={{ y: 24, duration: 280, easing: cubicOut }}
                role="presentation"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
        >
            <div class="bg-background/40 backdrop-blur-xl shadow-none border-none rounded-3xl p-4 flex flex-col gap-4">
                <div class="flex justify-center">
                    <span class="text-foreground/80 bg-foreground/5 backdrop-blur-md px-3 py-1 rounded-full font-mono text-xs font-bold">
                        {readerState.currentGroupIndex + 1}
                        <span class="opacity-30 mx-0.5">/</span>
                        {readerState.groupedImages.length}
                    </span>
                </div>
                    <div class="px-2">
                        <Slider
                                value={[readerState.currentGroupIndex]}
                                min={0}
                                max={readerState.groupedImages.length - 1}
                                step={1}
                                dir={readerState.direction}
                                onValueChange={(v) => {
                                    readerState.skipAnimation = false;
                                    readerState.updatePageWithDir(v[0]);
                                }}
                                class="w-full"
                        />
                    </div>
            </div>
        </div>
    {/if}
</Reader>

<style>
    .safe-reader {
        padding-bottom: calc(env(safe-area-inset-bottom) + 2rem);
        padding-left: env(safe-area-inset-left);
        padding-right: env(safe-area-inset-right);
        touch-action: pan-y pinch-zoom;
    }

    .scroll-row {
        justify-content: center;
    }

    .scroll-row.row-fit-height {
        height: 90vh;
        align-items: center;
    }

    .scroll-row:not(.row-fit-height) :global(.relative) {
        min-height: 300px;
    }
</style>