<script lang="ts">
    import { fade } from "svelte/transition";
    import { Button } from "@/components/ui/button";
    import * as Drawer from "@/components/ui/drawer";
    import * as Sheet from "@/components/ui/sheet";
    import * as Popover from "@/components/ui/popover";
    import { Spinner } from "@/components/ui/spinner";
    import {
        AlertCircle, ChevronLeft, Settings2,
        ArrowLeft, ArrowRight, Check, ChevronsUpDown
    } from "lucide-svelte";
    import type { Snippet } from "svelte";
    import { i18n } from '@/stores/i18n.svelte.js';
    import { discordApi } from "@/api/discord/discord";
    import type { BaseReaderState } from "@/app/reader.svelte";
    import {layoutState} from "@/stores/layout.svelte";

    let {
        readerState,
        contentType = "manga",
        currentProgress = null,
        children,
        settings,
    }: {
        readerState: BaseReaderState;
        contentType?: "manga" | "novel";
        currentProgress?: string | null;
        children: Snippet;
        settings: Snippet;
    } = $props();

    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024);
    let chapterDropdownOpen = $state(false);

    let sortedChapters = $derived(
        [...readerState.allChapters].sort(
            (a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber)
        )
    );
    let currentIndex = $derived(
        sortedChapters.findIndex(c => Number(c.number ?? c.unitNumber) === readerState.chapterNumber)
    );
    let prevChapter = $derived(currentIndex > 0 ? sortedChapters[currentIndex - 1] : null);
    let nextChapter = $derived(
        currentIndex >= 0 && currentIndex < sortedChapters.length - 1
            ? sortedChapters[currentIndex + 1]
            : null
    );
    let baseRoute = $derived(contentType === "novel" ? "/read-novel" : "/read");

    function getCleanTitle(chap: any, num: number): string | null {
        const title = chap?.title?.trim();
        if (!title) return null;
        const numText = i18n.t('reader.chapter_number', { count: num }).trim();
        if (title.toLowerCase() === numText.toLowerCase()) return null;
        if (/^chapter\s+\d+(\.\d+)?$/i.test(title) && Number(title.replace(/[^\d.]/g, '')) === num) {
            return null;
        }
        return title;
    }

    let displayChapterText = $derived.by(() => {
        if (readerState.isLoading) return i18n.t('reader.loading');
        const chap = sortedChapters[currentIndex];
        const numText = i18n.t('reader.chapter_number', { count: readerState.chapterNumber });
        const cleanTitle = getCleanTitle(chap, readerState.chapterNumber);
        return cleanTitle ? `${cleanTitle} — ${numText}` : numText;
    });

    let immersive = $derived(contentType === "manga" && (readerState as any).layout === "paged");
    let uiVisible = $derived(!immersive || !!(readerState as any).showOverlay);

    let headerClasses = $derived.by(() => {
        const base = "z-40 p-2 shrink-0 min-h-[60px] flex items-center justify-between gap-2 transition-all duration-300 ease-out";
        const safeArea = !layoutState.isMobile ? "pt-13" : "pt-safe";
        if (immersive) {
            return `${base} ${safeArea} absolute top-0 left-0 right-0 reader-header-immersive bg-gradient-to-b from-black/70 via-black/35 to-transparent border-none shadow-none ${
                uiVisible ? "opacity-100 translate-y-0" : "opacity-0 -translate-y-3 pointer-events-none"
            }`;
        }
        return `${base} ${safeArea} relative bg-background/95 backdrop-blur-md border-b border-border/50 shadow-sm`;
    });

    $effect(() => {
        if (!readerState.isLoading && !readerState.error && readerState.title) {
            discordApi.setActivity({
                title: readerState.title,
                details: displayChapterText,
                imageUrl: readerState.coverImage,
                startTime: null,
                endTime: null,
                isVideo: false,
                isNsfw: readerState.isNsfw,
            }).catch(() => {});
        }
        return () => { discordApi.clearActivity().catch(() => {}); };
    });

    function getChapterUrl(chap: any) {
        if (!chap) return "#";
        const num = chap.number ?? chap.unitNumber;
        return `${baseRoute}/${readerState.cid}/${readerState.extension}/${num}`;
    }

    let isDesktopUiVisible = $state(true);
    let hideTimeout: ReturnType<typeof setTimeout>;

    function handleMouseMove() {
        if (isMobile) return;
        isDesktopUiVisible = true;
        clearTimeout(hideTimeout);

        hideTimeout = setTimeout(() => {
            if (!chapterDropdownOpen && !readerState.showSettings) {
                isDesktopUiVisible = false;
            }
        }, 2500);
    }

    $effect(() => {
        if (!isMobile && !chapterDropdownOpen && !readerState.showSettings) {
            handleMouseMove();
        }
        return () => clearTimeout(hideTimeout);
    });
</script>

{#snippet chapterDropdownContent()}
    <Popover.Content class="w-[280px] sm:w-[320px] p-0 flex flex-col max-h-[60vh] shadow-xl border-border/50" collisionPadding={16}>
        <div class="px-4 py-3 border-b border-border/40 bg-muted/30">
            <h4 class="font-bold text-sm">{i18n.t('reader.select_chapter')}</h4>
            <p class="text-xs text-muted-foreground">
                {i18n.t('reader.chapters_available', { count: sortedChapters.length })}
            </p>
        </div>
        <div class="flex-1 overflow-y-auto custom-scrollbar p-1 flex flex-col gap-0.5">
            {#each sortedChapters as chap}
                {@const num = chap.number ?? chap.unitNumber}
                {@const isCurrent = num === readerState.chapterNumber}
                {@const cleanTitle = getCleanTitle(chap, num)}
                <a
                        href={getChapterUrl(chap)}
                        onclick={() => (chapterDropdownOpen = false)}
                        class="flex flex-col items-start px-3 py-2 text-sm rounded-md transition-colors {isCurrent ? 'bg-primary/10 text-primary font-bold' : 'hover:bg-muted'}"
                >
                    <div class="flex items-center justify-between w-full">
                        <span class="truncate pr-2">
                            {#if cleanTitle}
                                {cleanTitle}
                                <span class="opacity-70 text-xs ml-1">
                                    • {i18n.t('reader.chapter_number', { count: num })}
                                </span>
                            {:else}
                                {i18n.t('reader.chapter_number', { count: num })}
                            {/if}
                        </span>
                        {#if isCurrent}<Check class="size-4 shrink-0" />{/if}
                    </div>
                </a>
            {/each}
        </div>
    </Popover.Content>
{/snippet}

<svelte:window bind:innerWidth />

<div
        class="bg-background text-foreground flex flex-col h-full w-full relative overflow-hidden"
        onmousemove={handleMouseMove}
        role="presentation"
>
    {#if isMobile}
        <header class={headerClasses}>
            <div class="flex items-center gap-1.5 sm:gap-3 overflow-hidden flex-1">
                <Button variant="ghost" size="icon" href={`/c/${readerState.cid}`} class="rounded-full size-9 shrink-0">
                    <ChevronLeft class="size-5" />
                </Button>

                <Popover.Root bind:open={chapterDropdownOpen}>
                    <Popover.Trigger
                            class="flex flex-col items-start justify-center h-auto py-1 px-2.5 rounded-lg hover:bg-muted/50 transition-colors w-full max-w-[200px] sm:max-w-[300px] outline-none"
                            disabled={readerState.isLoading || !!readerState.error}
                    >
                        <span class="font-bold text-[13px] sm:text-sm leading-tight truncate w-full text-left">
                            {readerState.title || i18n.t('reader.loading')}
                        </span>
                        <div class="flex items-center gap-1 mt-0.5 w-full">
                            <span class="text-[11px] sm:text-xs text-muted-foreground truncate text-left">
                                {displayChapterText}
                            </span>
                            <ChevronsUpDown class="size-3 text-muted-foreground shrink-0 opacity-50" />
                        </div>
                    </Popover.Trigger>
                    {@render chapterDropdownContent()}
                </Popover.Root>
            </div>

            <div class="flex items-center gap-1 sm:gap-2 shrink-0">
                {#if currentProgress && !readerState.isLoading && !readerState.error}
                    <div class="text-[10px] sm:text-xs font-mono font-bold text-muted-foreground bg-muted px-2 py-1 rounded-md border border-border/50 hidden md:block">
                        {currentProgress}
                    </div>
                {/if}

                <div class="flex bg-muted/50 rounded-lg p-0.5 border border-border/50">
                    <Button variant="ghost" size="icon" class="size-8 rounded-md" href={prevChapter ? getChapterUrl(prevChapter) : undefined} disabled={!prevChapter || readerState.isLoading}>
                        <ArrowLeft class="size-4" />
                    </Button>
                    <Button variant="ghost" size="icon" class="size-8 rounded-md" href={nextChapter ? getChapterUrl(nextChapter) : undefined} disabled={!nextChapter || readerState.isLoading}>
                        <ArrowRight class="size-4" />
                    </Button>
                </div>

                <Button variant={readerState.showSettings ? 'secondary' : 'ghost'} size="icon" disabled={readerState.isLoading || !!readerState.error} class="rounded-full size-9" onclick={() => (readerState.showSettings = !readerState.showSettings)}>
                    <Settings2 class="size-4" />
                </Button>
            </div>
        </header>
    {/if}

    <div class="flex flex-1 min-h-0 relative">
        {#if readerState.isLoading}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background">
                <Spinner class="w-10 h-10 text-primary" />
                <span class="text-muted-foreground font-medium tracking-wide">{i18n.t('reader.loading')}</span>
            </div>
        {:else if readerState.error}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                <AlertCircle class="w-12 h-12 text-destructive" />
                <p class="text-foreground text-lg font-medium">{i18n.t(readerState.error.key)}</p>
                <Button variant="secondary" onclick={() => readerState.retry()}>{i18n.t('content.retry')}</Button>
            </div>
        {:else if readerState.isEmpty}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                <AlertCircle class="w-12 h-12 text-muted-foreground" />
                <p class="text-foreground text-lg font-medium">{i18n.t('reader.no_content')}</p>
                <Button variant="secondary" onclick={() => readerState.retry()}>{i18n.t('content.retry')}</Button>
            </div>
        {:else}
            <div class="flex-1 relative flex flex-col min-h-0 overflow-hidden">
                {#if currentProgress}
                    <div class="absolute top-0 left-0 right-0 z-30 h-[3px] bg-foreground/10 pointer-events-none">
                        <div
                                class="h-full bg-primary transition-[width] duration-150 ease-out"
                                style="width: {currentProgress}"
                        ></div>
                    </div>
                {/if}
                {@render children()}
            </div>
        {/if}
    </div>

    {#if !isMobile && !readerState.isLoading && !readerState.error && !readerState.isEmpty}
        <div
                class="absolute bottom-8 left-1/2 -translate-x-1/2 z-[60] transition-all duration-500 ease-out"
                class:opacity-0={!isDesktopUiVisible}
                class:translate-y-6={!isDesktopUiVisible}
                class:pointer-events-none={!isDesktopUiVisible}
                onmouseenter={() => { isDesktopUiVisible = true; clearTimeout(hideTimeout); }}
                onmouseleave={handleMouseMove}
                role="toolbar"
                tabindex="-1"
        >
            <div class="flex items-center gap-2 bg-card/80 backdrop-blur-2xl border border-border/50 shadow-xl rounded-full px-2 py-2">

                <div class="flex items-center gap-1">
                    <Button variant="ghost" size="icon" class="size-9 rounded-full" href={prevChapter ? getChapterUrl(prevChapter) : undefined} disabled={!prevChapter}>
                        <ArrowLeft class="size-4" />
                    </Button>

                    <Popover.Root bind:open={chapterDropdownOpen}>
                        <Popover.Trigger class="px-4 py-1 hover:bg-foreground/5 rounded-full min-w-[200px] outline-none">
                            <span class="font-bold text-sm block truncate">{readerState.title}</span>
                            <span class="text-[11px] text-muted-foreground">{displayChapterText}</span>
                        </Popover.Trigger>
                        {@render chapterDropdownContent()}
                    </Popover.Root>

                    <Button variant="ghost" size="icon" class="size-9 rounded-full" href={nextChapter ? getChapterUrl(nextChapter) : undefined} disabled={!nextChapter}>
                        <ArrowRight class="size-4" />
                    </Button>
                </div>

                {#if currentProgress}
                    <div class="text-[11px] font-mono font-semibold text-muted-foreground bg-foreground/5 px-3 py-1 rounded-full border border-border/30 text-center min-w-[4rem]">
                        {currentProgress}
                    </div>
                {/if}

                <Button variant="ghost" size="icon" href={`/c/${readerState.cid}`} class="rounded-full size-9">
                    <ChevronLeft class="size-5" />
                </Button>
                <Button variant={readerState.showSettings ? 'secondary' : 'ghost'} size="icon" class="rounded-full size-9" onclick={() => (readerState.showSettings = !readerState.showSettings)}>
                    <Settings2 class="size-4" />
                </Button>
            </div>
        </div>
    {/if}

    {#if !readerState.isLoading && !readerState.error}
        {#if isMobile}
            <Drawer.Root bind:open={readerState.showSettings}>
                <Drawer.Content class="bg-background/95 backdrop-blur-xl border-border/50">
                    <Drawer.Header>
                        <Drawer.Title>{i18n.t('reader.settings')}</Drawer.Title>
                    </Drawer.Header>
                    <div class="p-4 pb-8 max-h-[75vh] overflow-y-auto">{@render settings()}</div>
                </Drawer.Content>
            </Drawer.Root>
        {:else}
            <Sheet.Root bind:open={readerState.showSettings}>
                <Sheet.Content side="right" class="w-[340px] sm:w-[400px] overflow-y-auto bg-card/95 backdrop-blur-xl border-l border-border/50 shadow-2xl p-0">
                    <Sheet.Header class="p-6 pb-0">
                        <Sheet.Title class="text-left font-semibold text-lg border-b border-border/40 pb-4 mb-6">
                            {i18n.t('reader.settings')}
                        </Sheet.Title>
                    </Sheet.Header>
                    <div class="px-6 pb-8">{@render settings()}</div>
                </Sheet.Content>
            </Sheet.Root>
        {/if}
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150,150,150,0.3); border-radius: 10px; }

    .pt-safe {
        padding-top: max(calc(env(safe-area-inset-top) + 0.5rem), 0.875rem);
    }

    :global(.reader-header-immersive) {
        color: #fff;
    }
    :global(.reader-header-immersive button) {
        color: #fff;
    }
    :global(.reader-header-immersive) :global(.text-muted-foreground) {
        color: rgba(255, 255, 255, 0.75);
    }
    :global(.reader-header-immersive) :global(.border-border\/50),
    :global(.reader-header-immersive) :global(.border-border\/40) {
        border-color: rgba(255, 255, 255, 0.18);
    }
    :global(.reader-header-immersive) :global(.bg-muted\/50) {
        background-color: rgba(255, 255, 255, 0.15);
    }
</style>