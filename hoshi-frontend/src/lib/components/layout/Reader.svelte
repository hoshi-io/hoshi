<script lang="ts" module>
    /** Desktop progress rail. `value` is the filled part (0..1); `seek` gets the clicked spot (0..1). */
    export type ReaderProgress = {
        value: number;
        label: string;
        pages?: number;
        /** Right-to-left manga: the mobile slider runs the other way. */
        rtl?: boolean;
        seek: (ratio: number) => void;
    };
</script>

<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { fade, slide } from "svelte/transition";
    import { goto } from "$app/navigation";
    import { Button } from "@/components/ui/button";
    import * as Drawer from "@/components/ui/drawer";
    import { Spinner } from "@/components/ui/spinner";
    import AlertCircle from 'phosphor-svelte/lib/WarningCircleIcon';
    import ChevronLeft from 'phosphor-svelte/lib/CaretLeftIcon';
    import Settings2 from 'phosphor-svelte/lib/SlidersIcon';
    import ArrowLeft from 'phosphor-svelte/lib/ArrowLeftIcon';
    import ChevronsUpDown from 'phosphor-svelte/lib/CaretUpDownIcon';
    import ChevronRight from 'phosphor-svelte/lib/CaretRightIcon';
    import ListBullets from 'phosphor-svelte/lib/ListBulletsIcon';
    import CornersOut from 'phosphor-svelte/lib/CornersOutIcon';
    import CornersIn from 'phosphor-svelte/lib/CornersInIcon';
    import Reload from 'phosphor-svelte/lib/ArrowsClockwiseIcon';
    import X from 'phosphor-svelte/lib/XIcon';
    import { setWindowFullscreen, isWindowFullscreen } from "@/utils/fullscreen";
    import { chapterLabel } from "@/utils/chapterTitle";
    import ChapterList from "@/components/readers/ChapterList.svelte";
    import type { Snippet } from "svelte";
    import { i18n } from '@/stores/i18n.svelte.js';
    import { discordApi } from "@/api/discord/discord";
    import type { BaseReaderState } from "@/app/reader.svelte";
    import {layoutState} from "@/stores/layout.svelte";
    import { scanlatorLabel } from "@/utils/chapterVariants.ts";

    let {
        readerState,
        contentType = "manga",
        progress = null,
        children,
        settings,
    }: {
        readerState: BaseReaderState;
        contentType?: "manga" | "novel";
        progress?: ReaderProgress | null;
        children: Snippet;
        settings: Snippet;
    } = $props();

    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024);
    let showChapters = $state(false);

    function toggleChapters() {
        if (!showChapters) readerState.showSettings = false;
        showChapters = !showChapters;
    }

    // One side panel at a time.
    $effect(() => {
        if (readerState.showSettings) showChapters = false;
    });

    let sortedChapters = $derived(readerState.allChapters);
    let prevChapter = $derived(readerState.prevChapter);
    let nextChapter = $derived(readerState.nextChapter);

    let displayChapterText = $derived.by(() => {
        if (readerState.isLoading) return i18n.t('reader.loading');
        const { title, subtitle } = chapterLabel(readerState.currentChapter, readerState.chapterNumber);
        return subtitle ? `${title} — ${subtitle}` : title;
    });

    let immersive = $derived(contentType === "manga" && (readerState as any).layout === "paged");
    let uiVisible = $derived(!immersive || !!(readerState as any).showOverlay);

    /** Floats over the page in every mode; only the immersive (paged manga) one toggles. */
    let footerClasses = $derived(
        `absolute bottom-0 left-0 right-0 z-40 flex items-center gap-2 px-3 pt-2 pb-[max(env(safe-area-inset-bottom),0.75rem)] transition-all duration-300 ease-out ${
            !immersive || uiVisible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-3 pointer-events-none"
        }`
    );
    const footerPiece = "rounded-sm border border-border/50 bg-background/95 shadow-lg backdrop-blur-xl";

    let headerClasses = $derived.by(() => {
        const base = "z-40 p-2 shrink-0 min-h-[60px] flex items-center justify-between gap-2 transition-all duration-300 ease-out";
        const safeArea = !layoutState.isMobile ? "pt-13" : "pt-safe";
        if (immersive) {
            return `${base} ${safeArea} absolute top-0 left-0 right-0 bg-background/95 backdrop-blur-xl border-b border-border/50 ${
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
        return chap ? readerState.chapterHref(chap) : "#";
    }

    const EDGE = 120;
    const HIDE_AFTER = 10000;

    let nearTop = $state(false);
    let hoveringHeader = $state(false);
    let idle = $state(false);
    let hoveringTools = $state(false);
    let railEl = $state<HTMLDivElement>();
    let railDragging = $state(false);
    let railHover = $state<number | null>(null);
    let idleTimeout: ReturnType<typeof setTimeout>;
    let headerTimeout: ReturnType<typeof setTimeout> | undefined;

    /** The header lingers a while after the mouse leaves the top edge. */
    function setNearTop(near: boolean) {
        if (near) {
            clearTimeout(headerTimeout);
            headerTimeout = undefined;
            nearTop = true;
        } else if (nearTop && !headerTimeout) {
            headerTimeout = setTimeout(() => {
                nearTop = false;
                headerTimeout = undefined;
            }, HIDE_AFTER);
        }
    }

    const ready = $derived(!readerState.isLoading && !readerState.error && !readerState.isEmpty);
    const headerShown = $derived(!ready || nearTop || hoveringHeader);
    const toolsShown = $derived(!idle || hoveringTools);

    let rootEl = $state<HTMLDivElement>();

    // On the window, not the reader: the titlebar sits above it and must keep the header up.
    function handleMouseMove(e: MouseEvent) {
        if (isMobile || !rootEl) return;
        const rect = rootEl.getBoundingClientRect();
        setNearTop(e.clientY - rect.top < EDGE);
        idle = false;
        clearTimeout(idleTimeout);
        idleTimeout = setTimeout(() => (idle = true), HIDE_AFTER);
    }

    function handleMouseLeave(e: MouseEvent) {
        if (e.relatedTarget) return; // still inside the window (e.g. the titlebar)
        setNearTop(false);
    }

    $effect(() => () => {
        clearTimeout(idleTimeout);
        clearTimeout(headerTimeout);
    });

    // ---- fullscreen (desktop window) ----
    let fullscreen = $state(false);
    let enteredFullscreen = false;

    onMount(() => {
        if (!layoutState.isMobile) isWindowFullscreen().then((v) => (fullscreen = v));
    });

    onDestroy(() => {
        if (enteredFullscreen) setWindowFullscreen(false);
    });

    async function toggleFullscreen() {
        fullscreen = await setWindowFullscreen(!fullscreen);
        enteredFullscreen = fullscreen;
    }

    // ---- shortcuts ----
    function onKeyDown(e: KeyboardEvent) {
        const t = e.target as HTMLElement | null;
        if (t?.closest("input, textarea, select, [contenteditable=true]")) return;
        if (e.ctrlKey || e.metaKey || e.altKey) return;

        switch (e.key) {
            case "[":
                if (prevChapter) goto(getChapterUrl(prevChapter));
                break;
            case "]":
                if (nextChapter) goto(getChapterUrl(nextChapter));
                break;
            case "f":
            case "F":
                if (!isMobile) toggleFullscreen();
                break;
            case "s":
            case "S":
                if (!readerState.isLoading && !readerState.error) readerState.showSettings = !readerState.showSettings;
                break;
            case "c":
            case "C":
                if (!readerState.isLoading && readerState.rawChapters.length) toggleChapters();
                break;
            case "Escape":
                if (isMobile) return;
                readerState.showSettings = false;
                showChapters = false;
                break;
            default:
                return;
        }
        e.preventDefault();
    }

    // ---- mobile page slider ----
    let sliderEl = $state<HTMLDivElement>();
    let sliderDrag = $state<number | null>(null);

    /** Thumb position (0..1): pages sit on evenly spaced stops, first page at 0, last at 1. */
    const sliderPos = $derived.by(() => {
        if (!progress) return 0;
        const { value, pages } = progress;
        return pages && pages > 1 ? (Math.round(value * pages) - 1) / (pages - 1) : value;
    });

    function sliderRatio(e: PointerEvent) {
        const rect = sliderEl!.getBoundingClientRect();
        const r = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
        return progress?.rtl ? 1 - r : r;
    }

    function sliderPage(r: number) {
        return progress?.pages ? Math.round(r * (progress.pages - 1)) : 0;
    }

    function sliderSeek(r: number) {
        if (!progress) return;
        // Snap to the nearest page stop; `seek` expects the page's slice of the rail.
        progress.seek(progress.pages ? (sliderPage(r) + 0.5) / progress.pages : Math.min(r, 0.9999));
    }

    function onSliderDown(e: PointerEvent) {
        e.stopPropagation();
        sliderEl!.setPointerCapture(e.pointerId);
        sliderDrag = sliderRatio(e);
        sliderSeek(sliderDrag);
    }

    function onSliderMove(e: PointerEvent) {
        if (sliderDrag === null) return;
        sliderDrag = sliderRatio(e);
        sliderSeek(sliderDrag);
    }

    function onSliderUp(e: PointerEvent) {
        sliderDrag = null;
        sliderEl?.releasePointerCapture(e.pointerId);
    }

    // ---- progress rail ----

    function railRatio(e: PointerEvent) {
        const rect = railEl!.getBoundingClientRect();
        return Math.min(0.9999, Math.max(0, (e.clientY - rect.top) / rect.height));
    }

    function railLabel(r: number) {
        if (!progress) return "";
        return progress.pages ? `${Math.floor(r * progress.pages) + 1} / ${progress.pages}` : `${Math.round(r * 100)}%`;
    }

    function onRailDown(e: PointerEvent) {
        e.preventDefault();
        railDragging = true;
        railEl!.setPointerCapture(e.pointerId);
        progress?.seek(railRatio(e));
    }

    function onRailMove(e: PointerEvent) {
        railHover = railRatio(e);
        if (railDragging) progress?.seek(railHover);
    }

    function onRailUp(e: PointerEvent) {
        railDragging = false;
        railEl?.releasePointerCapture(e.pointerId);
    }

    const chapterGroup = $derived(readerState.currentChapter ? scanlatorLabel(readerState.currentChapter) : null);
</script>

<svelte:window bind:innerWidth onkeydown={onKeyDown} onmousemove={handleMouseMove} />

<div
        bind:this={rootEl}
        class="bg-background text-foreground flex flex-col h-full w-full relative overflow-hidden"
        onmouseleave={handleMouseLeave}
        role="presentation"
>
    {#if isMobile}
        <header class={headerClasses}>
            <div class="flex items-center gap-1.5 sm:gap-3 overflow-hidden flex-1">
                <Button variant="ghost" size="icon" href={`/c/${readerState.cid}`} class="rounded-full size-9 shrink-0">
                    <ChevronLeft class="size-5" />
                </Button>

                <button
                        type="button"
                        onclick={toggleChapters}
                        class="flex min-w-0 flex-1 items-center gap-2 rounded-sm px-2.5 py-1 text-left outline-none transition-colors hover:bg-muted/50"
                        disabled={readerState.isLoading || !!readerState.error}
                        aria-label={i18n.t('reader.select_chapter')}
                >
                    <span class="min-w-0 flex-1">
                        <span class="block truncate text-[13px] font-bold leading-tight sm:text-sm">
                            {readerState.title || i18n.t('reader.loading')}
                        </span>
                        <span class="mt-0.5 block truncate text-[11px] text-muted-foreground sm:text-xs">
                            {displayChapterText}
                        </span>
                    </span>
                    <ChevronsUpDown class="size-4 shrink-0 text-muted-foreground" />
                </button>
            </div>

            <div class="flex items-center gap-1 sm:gap-2 shrink-0">
                <Button variant={readerState.showSettings ? 'secondary' : 'ghost'} size="icon" disabled={readerState.isLoading || !!readerState.error} class="rounded-full size-9" onclick={() => (readerState.showSettings = !readerState.showSettings)}>
                    <Settings2 class="size-4" />
                </Button>
            </div>
        </header>
    {/if}

    <div class="flex flex-1 min-h-0 relative">
        <div class="relative flex min-w-0 flex-1 flex-col {!isMobile && !toolsShown && !headerShown ? 'cursor-none' : ''}">
            {#if readerState.isLoading}
                <div transition:fade class="absolute inset-0 z-30 flex flex-col items-center justify-center gap-4 bg-background">
                    <Spinner class="w-10 h-10 text-primary" />
                    <span class="text-muted-foreground font-medium tracking-wide">{i18n.t('reader.loading')}</span>
                </div>
            {:else if readerState.error}
                <div transition:fade class="absolute inset-0 z-30 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                    <p class="text-foreground text-lg font-medium">{i18n.t(readerState.error.key)}</p>
                    <Button variant="secondary" onclick={() => readerState.retry()}>{i18n.t('content.retry')}</Button>
                </div>
            {:else if readerState.isEmpty}
                <div transition:fade class="absolute inset-0 z-30 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                    <AlertCircle class="w-12 h-12 text-muted-foreground" />
                    <p class="text-foreground text-lg font-medium">{i18n.t('reader.no_content')}</p>
                    <Button variant="secondary" onclick={() => readerState.retry()}>{i18n.t('content.retry')}</Button>
                </div>
            {:else}
                <div class="flex-1 relative flex flex-col min-h-0 overflow-hidden">
                    {@render children()}
                </div>
            {/if}

            {#if !isMobile}
                <!-- Top bar -->
                <div
                        role="toolbar"
                        tabindex="-1"
                        class="absolute inset-x-0 top-0 z-40 border-b border-border/40 bg-background/90 pt-8 backdrop-blur-xl transition-all duration-300 ease-out
                               {headerShown ? 'translate-y-0 opacity-100' : 'pointer-events-none -translate-y-full opacity-0'}"
                        onmouseenter={() => (hoveringHeader = true)}
                        onmouseleave={() => (hoveringHeader = false)}
                >
                    <div class="grid h-14 grid-cols-[1fr_minmax(0,auto)_1fr] items-center gap-4 px-4">
                        <div class="flex items-center gap-2">
                            <Button variant="secondary" size="icon" href={`/c/${readerState.cid}`} class="size-9 rounded-sm" aria-label={i18n.t('reader.back')}>
                                <ArrowLeft class="size-4" />
                            </Button>
                            {#if prevChapter}
                                <Button variant="ghost" href={getChapterUrl(prevChapter)} class="h-9 gap-1 rounded-sm px-3 text-sm font-semibold text-muted-foreground hover:text-foreground">
                                    <ChevronLeft class="size-4" />
                                    {i18n.t('reader.prev_chapter')}
                                </Button>
                            {/if}
                        </div>

                        <div class="min-w-0 text-center">
                            <p class="truncate text-sm font-bold leading-tight">{readerState.title || i18n.t('reader.loading')}</p>
                            <p class="mt-0.5 truncate text-xs text-muted-foreground">
                                {displayChapterText}
                                {#if chapterGroup}<span class="opacity-50"> · </span>{chapterGroup}{/if}
                                {#if ready && progress}<span class="opacity-50"> · </span><span class="font-mono tabular-nums">{progress.label}</span>{/if}
                            </p>
                        </div>

                        <div class="flex items-center justify-end gap-2">
                            <Button
                                    variant={showChapters ? "secondary" : "ghost"}
                                    size="icon"
                                    class="size-9 rounded-sm"
                                    disabled={readerState.isLoading || sortedChapters.length === 0}
                                    title="{i18n.t('reader.select_chapter')} (C)"
                                    aria-label={i18n.t('reader.select_chapter')}
                                    onclick={toggleChapters}
                            >
                                <ListBullets class="size-4" />
                            </Button>
                            {#if nextChapter}
                                <Button variant="secondary" href={getChapterUrl(nextChapter)} class="h-9 gap-1 rounded-sm px-3.5 text-sm font-semibold">
                                    {i18n.t('reader.next_chapter')}
                                    <ChevronRight class="size-4" />
                                </Button>
                            {/if}
                        </div>
                    </div>
                </div>

                {#if ready}
                    <!-- Tool rail -->
                    <div
                            class="absolute right-4 top-1/2 z-40 flex -translate-y-1/2 flex-col gap-1 rounded-sm border border-border/40 bg-background/80 p-1.5 shadow-lg backdrop-blur-xl transition-all duration-300 ease-out
                                   {toolsShown ? 'translate-x-0 opacity-100' : 'pointer-events-none translate-x-4 opacity-0'}"
                            role="toolbar"
                            tabindex="-1"
                            onmouseenter={() => (hoveringTools = true)}
                            onmouseleave={() => (hoveringTools = false)}
                    >
                        <Button
                                variant={readerState.showSettings ? "secondary" : "ghost"}
                                size="icon"
                                class="size-10 rounded-sm"
                                title="{i18n.t('reader.settings')} (S)"
                                aria-label={i18n.t('reader.settings')}
                                onclick={() => (readerState.showSettings = !readerState.showSettings)}
                        >
                            <Settings2 class="size-[18px]" />
                        </Button>
                        <Button
                                variant="ghost"
                                size="icon"
                                class="size-10 rounded-sm"
                                title="{i18n.t(fullscreen ? 'watch.player.fullscreen_exit' : 'watch.player.fullscreen_enter')} (F)"
                                aria-label={i18n.t(fullscreen ? 'watch.player.fullscreen_exit' : 'watch.player.fullscreen_enter')}
                                onclick={toggleFullscreen}
                        >
                            {#if fullscreen}<CornersIn class="size-[18px]" />{:else}<CornersOut class="size-[18px]" />{/if}
                        </Button>
                        <Button
                                variant="ghost"
                                size="icon"
                                class="size-10 rounded-sm"
                                title={i18n.t('reader.reload')}
                                aria-label={i18n.t('reader.reload')}
                                onclick={() => readerState.retry()}
                        >
                            <Reload class="size-[18px]" />
                        </Button>
                    </div>

                    {#if progress}
                        <!-- Progress rail -->
                        <div
                                bind:this={railEl}
                                class="group/rail absolute bottom-6 left-0 top-28 z-30 flex w-6 cursor-pointer justify-center"
                                role="slider"
                                tabindex="-1"
                                aria-valuenow={Math.round(progress.value * 100)}
                                aria-valuemin={0}
                                aria-valuemax={100}
                                onpointerdown={onRailDown}
                                onpointermove={onRailMove}
                                onpointerup={onRailUp}
                                onpointercancel={onRailUp}
                                onpointerleave={() => (railHover = null)}
                        >
                            <div
                                    class="relative h-full overflow-hidden rounded-sm bg-foreground/10 transition-all duration-200
                                           {railDragging ? 'w-2 opacity-100' : 'w-[3px] opacity-50 group-hover/rail:w-2 group-hover/rail:opacity-100'}"
                                    style={progress.pages && progress.pages <= 200
                                        ? `background-image: repeating-linear-gradient(to bottom, transparent 0, transparent calc(${100 / progress.pages}% - 1px), color-mix(in oklab, var(--background) 70%, transparent) calc(${100 / progress.pages}% - 1px), color-mix(in oklab, var(--background) 70%, transparent) ${100 / progress.pages}%);`
                                        : ''}
                            >
                                <div class="absolute inset-x-0 top-0 rounded-sm bg-primary transition-[height] duration-150" style="height: {progress.value * 100}%"></div>
                            </div>
                            {#if railHover !== null}
                                <span
                                        class="pointer-events-none absolute left-6 -translate-y-1/2 whitespace-nowrap rounded-sm border border-border/50 bg-popover px-2 py-1 font-mono text-[11px] font-semibold tabular-nums text-popover-foreground shadow-md"
                                        style="top: {railHover * 100}%"
                                >
                                    {railLabel(railHover)}
                                </span>
                            {/if}
                        </div>
                    {/if}
                {/if}
            {/if}
        </div>

        {#if !isMobile && (readerState.showSettings || (showChapters && !readerState.isLoading))}
            <aside
                    class="z-40 flex w-[380px] shrink-0 flex-col overflow-hidden border-l border-border/50 bg-card pt-8"
                    transition:slide={{ axis: 'x', duration: 220 }}
            >
                <div class="flex h-14 shrink-0 items-center justify-between border-b border-border/40 px-5">
                    <h2 class="text-base font-semibold">{i18n.t(showChapters ? 'reader.chapters' : 'reader.settings')}</h2>
                    <Button
                            variant="ghost"
                            size="icon"
                            class="size-8 rounded-sm"
                            aria-label={i18n.t('reader.close_settings')}
                            onclick={() => { readerState.showSettings = false; showChapters = false; }}
                    >
                        <X class="size-4" />
                    </Button>
                </div>
                {#if showChapters}
                    <div class="flex min-h-0 flex-1 flex-col px-4 pt-4">
                        <ChapterList {readerState} />
                    </div>
                {:else}
                    <div class="min-h-0 flex-1 overflow-y-auto px-5 py-6">{@render settings()}</div>
                {/if}
            </aside>
        {/if}
    </div>

    {#if isMobile && ready && progress}
        {@const pos = sliderDrag ?? sliderPos}
        {@const shown = progress.rtl ? 1 - pos : pos}
        <div class={footerClasses} role="toolbar" tabindex="-1">
            <Button variant="ghost" size="icon" class="size-11 shrink-0 {footerPiece}" href={prevChapter ? getChapterUrl(prevChapter) : undefined} disabled={!prevChapter} aria-label={i18n.t('reader.prev_chapter')}>
                <ChevronLeft class="size-5" />
            </Button>

            <div class="flex h-11 min-w-0 flex-1 items-center gap-1 px-1 {footerPiece} {progress.rtl ? 'flex-row-reverse' : ''}">
            <span class="w-9 shrink-0 text-center font-mono text-xs font-bold tabular-nums">
                {progress.pages ? sliderPage(pos) + 1 : progress.label}
            </span>

            <div
                    bind:this={sliderEl}
                    class="relative flex h-11 min-w-0 flex-1 touch-none items-center"
                    role="slider"
                    tabindex="0"
                    aria-label={i18n.t('reader.page')}
                    aria-valuenow={progress.pages ? sliderPage(pos) + 1 : Math.round(pos * 100)}
                    aria-valuemin={progress.pages ? 1 : 0}
                    aria-valuemax={progress.pages ?? 100}
                    onpointerdown={onSliderDown}
                    onpointermove={onSliderMove}
                    onpointerup={onSliderUp}
                    onpointercancel={onSliderUp}
            >
                <div class="relative h-1 w-full overflow-hidden rounded-sm bg-current/20">
                    <div
                            class="absolute inset-y-0 bg-primary {progress.rtl ? 'right-0' : 'left-0'}"
                            style="width: {pos * 100}%"
                    ></div>
                </div>
                <div
                        class="pointer-events-none absolute top-1/2 size-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-background bg-primary shadow-md transition-transform duration-150 {sliderDrag !== null ? 'scale-125' : ''}"
                        style="left: {shown * 100}%"
                ></div>
            </div>

            <span class="w-9 shrink-0 text-center font-mono text-xs font-bold tabular-nums opacity-60">
                {progress.pages ?? ''}
            </span>
            </div>

            <Button variant="ghost" size="icon" class="size-11 shrink-0 {footerPiece}" href={nextChapter ? getChapterUrl(nextChapter) : undefined} disabled={!nextChapter} aria-label={i18n.t('reader.next_chapter')}>
                <ChevronRight class="size-5" />
            </Button>
        </div>
    {/if}

    {#if isMobile && !readerState.isLoading}
        <Drawer.Root bind:open={showChapters}>
            <Drawer.Content class="h-[80dvh] border-border/50 bg-background">
                <Drawer.Header class="text-left">
                    <Drawer.Title>{i18n.t('reader.chapters')}</Drawer.Title>
                </Drawer.Header>
                <div class="flex min-h-0 flex-1 flex-col px-4 pb-[max(env(safe-area-inset-bottom),1rem)]">
                    <ChapterList {readerState} onSelect={() => (showChapters = false)} />
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}

    {#if isMobile && !readerState.isLoading && !readerState.error}
        <Drawer.Root bind:open={readerState.showSettings}>
            <Drawer.Content class="bg-background/95 backdrop-blur-xl border-border/50">
                <Drawer.Header>
                    <Drawer.Title>{i18n.t('reader.settings')}</Drawer.Title>
                </Drawer.Header>
                <div class="p-4 pb-8 max-h-[75vh] overflow-y-auto">{@render settings()}</div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150,150,150,0.3); border-radius: 10px; }

    .pt-safe {
        padding-top: max(calc(env(safe-area-inset-top) + 0.5rem), 0.875rem);
    }
</style>