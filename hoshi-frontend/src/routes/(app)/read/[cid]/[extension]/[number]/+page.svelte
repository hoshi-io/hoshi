<script lang="ts">
    import { MangaReaderState } from "@/app/manga.svelte";
    import Reader from "@/components/readers/Reader.svelte";
    import { fly } from "svelte/transition";
    import ReaderImage from "@/components/readers/ReaderImage.svelte";
    import MangaReaderSettings from "@/components/readers/MangaReaderSettings.svelte";

    const readerState = new MangaReaderState();

    const progress = $derived.by(() => {
        const pages = readerState.groupedImages.length;
        if (pages === 0) return null;
        const index = readerState.layout === "paged" ? readerState.currentGroupIndex : readerState.scrollRow;
        return {
            value: (index + 1) / pages,
            label: `${index + 1} / ${pages}`,
            pages,
            rtl: readerState.layout === "paged" && readerState.direction === "rtl",
            seek: (r: number) => readerState.seekPage(Math.floor(r * pages)),
        };
    });
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<Reader
        {readerState}
        contentType="manga"
        {progress}
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
            onscroll={(e) => readerState.onScroll(e)}
            onkeydown={(e) => {
                if (e.key === 'ArrowRight') readerState.turnPage('next');
                if (e.key === 'ArrowLeft') readerState.turnPage('prev');
            }}
            ontouchstart={(e) => readerState.handleTouchStart(e)}
            ontouchend={(e) => readerState.handleTouchEnd(e)}
    >
        {#if readerState.layout === "scroll"}
            <div
                    class="flex flex-col items-center w-full py-4 pb-24 lg:pt-24"
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