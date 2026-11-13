<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import { extensions } from "@/stores/extensions.svelte.js";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import BookOpen from 'phosphor-svelte/lib/BookOpenIcon';
    import SearchX from 'phosphor-svelte/lib/MagnifyingGlassIcon';
    import AlertCircle from 'phosphor-svelte/lib/WarningCircleIcon';
    import BookOpenCheck from 'phosphor-svelte/lib/CheckCircleIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { CoreError } from "@/api/client";
    import type { ChapterProgress } from "@/api/progress/types";
    import ResponsiveSelect from "$lib/components/ResponsiveSelect.svelte";
    import {
        ALL, chapterFacets, chapterHref, chapterNum, filterChapters, scanlatorLabel, type ChapterLike,
    } from "@/app/chapterVariants";

    let {
        cid,
        contentType,
        progress = [],
        isDerived,
        source
    }: {
        cid: string,
        contentType: string,
        progress?: ChapterProgress[],
        isDerived: boolean,
        source: string
    } = $props();

    const ARC_SIZE = 24;
    const basePath = $derived(contentType === "novel" ? "/read-novel" : "/read");

    let selectedExtensionName = $derived(
        isDerived ? source : ""
    );
    /** Every version the extension returned (several per number on multi-language/scanlator sources). */
    let rawChapters = $state<ChapterLike[]>([]);
    let language = $state(ALL);
    let scanlator = $state(ALL);

    const languages = $derived(chapterFacets(rawChapters).languages);
    const scanlators = $derived(
        chapterFacets(filterChapters(rawChapters, { language })).scanlators
    );
    const languageItems = $derived(languages.length > 1
        ? [{ value: ALL, label: i18n.t('content.all_languages') },
           ...languages.map(l => ({ value: l, label: l.toUpperCase() }))]
        : []);
    const scanlatorItems = $derived(scanlators.length > 1
        ? [{ value: ALL, label: i18n.t('content.all_scanlators') },
           ...scanlators.map(s => ({ value: s, label: s }))]
        : []);

    // A scanlator can vanish when the language changes.
    $effect(() => {
        if (scanlator !== ALL && !scanlators.includes(scanlator)) scanlator = ALL;
    });

    // Metadata columns only exist when they'd differ between rows.
    const showLangCol = $derived(languageItems.length > 0 && language === ALL);
    const showGroupCol = $derived(rawChapters.some(c => c.scanlators?.length));

    const chapters = $derived(filterChapters(rawChapters, { language, scanlator }) as any[]);
    /** Distinct chapter numbers of the filtered list (versions of one number count once). */
    const numbers = $derived([...new Set(chapters.map(chapterNum))]);
    let loading = $state(false);
    let error = $state<CoreError | null>(null);
    let selectedArc = $state("0");

    let loadToken = 0;

    const progressMap = $derived(
        new Map(progress.map(p => [p.chapter, p]))
    );

    const firstUnread = $derived.by(() => {
        const completedNums = new Set(progress.filter(p => p.completed).map(p => p.chapter));
        return chapters.find(ch => !completedNums.has(ch.number ?? ch.unitNumber));
    });

    const resumeChapter = $derived(
        chapters.length === 0 ? 1 : (firstUnread?.number ?? firstUnread?.unitNumber) ?? 1
    );

    const arcs = $derived.by(() => {
        if (numbers.length <= ARC_SIZE) return null;
        const chunks: { start: number; end: number; label: string }[] = [];
        for (let i = 0; i < numbers.length; i += ARC_SIZE) {
            const slice = numbers.slice(i, i + ARC_SIZE);
            const start = slice[0];
            const end = slice[slice.length - 1];
            chunks.push({ start, end, label: `${start}–${end}` });
        }
        return chunks;
    });

    const arcItems = $derived(
        arcs?.map((arc, i) => ({
            value: String(i),
            label: arc.label,
        })) ?? []
    );

    const resumeArcIndex = $derived.by(() => {
        if (!arcs) return 0;
        const idx = arcs.findIndex(a => resumeChapter >= a.start && resumeChapter <= a.end);
        return idx >= 0 ? idx : 0;
    });

    $effect(() => {
        if (chapters.length > 0) {
            selectedArc = String(resumeArcIndex);
        }
    });

    const visibleChapters = $derived.by(() => {
        if (!arcs) return chapters;
        const arc = arcs[Number(selectedArc)];
        return chapters.filter(ch => {
            const num = ch.number ?? ch.unitNumber;
            return num >= arc.start && num <= arc.end;
        });
    });

    let currentExtensions = $derived(
        contentType === "manga" ? extensions.manga :
            contentType === "novel" ? extensions.novel : []
    );

    let extensionItems = $derived(
        currentExtensions
            .map(ext => {
                const hasDuplicateName = currentExtensions.some(e => e.id !== ext.id && e.name === ext.name);
                const label = hasDuplicateName && ext.source
                    ? `${ext.name} (${ext.source})`
                    : ext.name;

                return { value: ext.id, label };
            })
            .sort((a, b) => a.label.localeCompare(b.label))
    );

    $effect(() => {
        if (!selectedExtensionName && extensionItems.length > 0) {
            selectedExtensionName = extensionItems[0].value;
        }
    });

    $effect(() => {
        if (selectedExtensionName) loadChapters(selectedExtensionName);
    });

    async function loadChapters(extId: string) {
        const token = ++loadToken;
        loading = true;
        error = null;
        language = ALL;
        scanlator = ALL;
        try {
            const res = await contentApi.getItems(cid, extId);
            if (token !== loadToken) return; // a newer extension switch happened, drop this

            rawChapters = (Array.isArray(res) ? res : []).sort((a, b) => chapterNum(a) - chapterNum(b));
        } catch (e: any) {
            if (token !== loadToken) return;
            error = e.key ? e : { key: 'content.failed_load' };
        } finally {
            if (token === loadToken) loading = false;
        }
    }

    function scrollIfResume(node: HTMLElement, isResume: boolean) {
        if (isResume) {
            requestAnimationFrame(() => {
                node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            });
        }
    }
</script>

<div class="space-y-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
        <h2 class="text-base md:text-lg font-bold tracking-tight">
            {i18n.t('content.chapters_title')}
            {#if numbers.length > 0}
                <span class="text-muted-foreground/50 font-medium">· {numbers.length}</span>
            {/if}
        </h2>

        {#if extensionItems.length > 1 || languageItems.length > 0 || scanlatorItems.length > 0 || (arcs && !loading && !error)}
            <div class="flex flex-wrap items-center gap-3">
                {#if extensionItems.length > 1}
                    <div class="w-[180px] shrink-0">
                        <ResponsiveSelect
                                bind:value={selectedExtensionName}
                                items={extensionItems}
                                placeholder={i18n.t('content.select_extension')}
                                class="bg-muted/30 border-border/20 hover:bg-muted/50 transition-colors rounded-sm font-medium capitalize text-xs h-8"
                        />
                    </div>
                {/if}

                {#if languageItems.length > 0 && !loading && !error}
                    <ResponsiveSelect
                            bind:value={language}
                            items={languageItems}
                            placeholder={i18n.t('content.all_languages')}
                            class="w-auto min-w-[7rem] bg-muted/30 border-border/20 hover:bg-muted/50 transition-colors rounded-sm font-medium text-xs h-8"
                    />
                {/if}

                {#if scanlatorItems.length > 0 && !loading && !error}
                    <ResponsiveSelect
                            bind:value={scanlator}
                            items={scanlatorItems}
                            placeholder={i18n.t('content.all_scanlators')}
                            class="w-auto min-w-[7rem] max-w-[200px] bg-muted/30 border-border/20 hover:bg-muted/50 transition-colors rounded-sm font-medium text-xs h-8"
                    />
                {/if}

                {#if arcs && !loading && !error}
                    <ResponsiveSelect
                            bind:value={selectedArc}
                            items={arcItems}
                            placeholder="..."
                            class="w-auto min-w-[7rem] bg-muted/30 border-border/20 hover:bg-muted/50 transition-colors rounded-sm font-medium text-xs h-8"
                    />
                {/if}
            </div>
        {/if}
    </div>

    <!-- Content Area -->
    {#if extensionItems.length === 0}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-sm border border-border/20 bg-muted/5">
            <BookOpen class="w-8 h-8 text-muted-foreground/20" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t('content.no_sources')}</p>
                <p class="text-[11px] text-muted-foreground/40">{i18n.t('content.install_extension')}</p>
            </div>
        </div>
    {:else if loading}
        <div class="flex flex-col gap-1.5">
            {#each Array(8) as _}
                <div class="flex items-center gap-3 px-3 py-2.5 rounded-sm border border-border/20 bg-muted/10">
                    <Skeleton class="shrink-0 w-9 h-9 rounded-sm" />
                    <div class="flex-1 space-y-1.5">
                        <Skeleton class="h-3.5 w-3/5 rounded-sm" />
                        <Skeleton class="h-2.5 w-1/4 rounded-sm opacity-40" />
                    </div>
                </div>
            {/each}
        </div>
    {:else if error}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-sm border border-destructive/10 bg-destructive/5">
            <AlertCircle class="w-8 h-8 text-destructive/40" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t(error.key)}</p>
            </div>
            <button onclick={() => loadChapters(selectedExtensionName)} class="text-[11px] font-bold text-muted-foreground/40 hover:text-muted-foreground/70 uppercase tracking-widest transition-colors">
                {i18n.t('content.retry')}
            </button>
        </div>
    {:else if chapters.length === 0}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-sm border border-border/20 bg-muted/5">
            <SearchX class="w-8 h-8 text-muted-foreground/20" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t('content.no_chapters')}</p>
                <p class="text-[11px] text-muted-foreground/40">{i18n.t('content.no_chapters_desc')}</p>
            </div>
        </div>
    {:else}
        <div class="flex flex-col gap-1.5 xl:max-h-[calc(100vh-32rem)] xl:overflow-y-auto xl:pr-1 hide-scrollbar">
            {#each visibleChapters as chapter (chapter.id || (chapter.number ?? chapter.unitNumber))}
                {@const num = chapter.number ?? chapter.unitNumber}
                {@const prog = progressMap.get(num)}
                {@const isRead = prog?.completed ?? false}
                {@const isResume = firstUnread ? chapter === firstUnread : num === resumeChapter}
                {@const url = chapterHref(basePath, cid, selectedExtensionName, chapter, rawChapters)}
                {@const group = scanlatorLabel(chapter)}
                {@const lang = showLangCol ? chapter.language : null}
                {@const title = chapter.title?.trim() ? chapter.title : null}

                <a
                        use:scrollIfResume={isResume}
                        href={url}
                        class="group flex items-center gap-3 px-3 py-2.5 rounded-sm border transition-all duration-200
                        {isRead
                            ? 'border-border/10 bg-muted/5 opacity-45 hover:opacity-70 hover:bg-muted/15 hover:border-border/30'
                            : isResume
                            ? 'border-primary/30 bg-primary/5 hover:bg-primary/10 hover:border-primary/50'
                            : 'border-border/20 bg-muted/10 hover:bg-muted/25 hover:border-border/50'}"
                >
                    <div class="shrink-0 w-9 h-9 rounded-lg bg-muted/40 flex items-center justify-center border border-border/20 group-hover:border-border/40 transition-colors">
                        <span class="text-xs font-black text-muted-foreground/50 group-hover:text-muted-foreground/80 transition-colors">{num}</span>
                    </div>

                    <div class="flex-1 min-w-0">
                        {#if title}
                            <p class="font-semibold text-sm leading-snug line-clamp-1 group-hover:text-primary transition-colors duration-150 {isRead ? 'text-muted-foreground/60' : ''}">
                                {title}
                            </p>
                        {:else}
                            <p class="font-semibold text-sm text-muted-foreground/60 group-hover:text-muted-foreground transition-colors duration-150">
                                {i18n.t('content.chapter')} {num}
                            </p>
                        {/if}

                        {#if isResume && !isRead}
                            <p class="text-[10px] font-bold text-primary uppercase tracking-tighter mt-0.5">{i18n.t('content.resume')}</p>
                        {/if}
                        <!-- Mobile: metadata goes under the title -->
                        {#if group || lang}
                            <p class="sm:hidden text-[10px] text-muted-foreground/40 truncate mt-0.5">
                                {#if lang}<span class="font-bold uppercase">{lang}</span>{#if group} · {/if}{/if}{group ?? ''}
                            </p>
                        {/if}
                    </div>

                    <!-- Desktop: fixed-width columns so rows line up like a table -->
                    {#if showGroupCol}
                        <span class="hidden sm:block shrink-0 w-44 truncate text-xs text-muted-foreground/50" title={group ?? undefined}>
                            {group ?? ''}
                        </span>
                    {/if}
                    {#if showLangCol}
                        <span class="hidden sm:flex shrink-0 w-9 justify-center">
                            {#if lang}
                                <span class="px-1.5 py-0.5 rounded-sm border border-border/30 bg-muted/30 text-[10px] font-bold uppercase text-muted-foreground/70">{lang}</span>
                            {/if}
                        </span>
                    {/if}

                    <span class="shrink-0 w-3.5">
                        {#if isRead}
                            <BookOpenCheck class="w-3.5 h-3.5 text-primary/50" />
                        {/if}
                    </span>
                </a>
            {/each}
        </div>
    {/if}
</div>

<style>
    .hide-scrollbar::-webkit-scrollbar { display: none; }
    .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>