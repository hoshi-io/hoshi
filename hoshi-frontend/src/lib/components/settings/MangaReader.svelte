<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import { Slider } from "$lib/components/ui/slider";
    import { fade } from "svelte/transition";
    import {
        BookOpen, Maximize,
        GalleryVertical, ChevronLeft, ChevronRight,
        Monitor, Info
    } from "lucide-svelte";
    import * as Kbd from "$lib/components/ui/kbd";
    import type { MangaConfig, MangaLayout } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";

    let {
        config = $bindable(),
        onSave
    }: {
        config: MangaConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let localGapX = $state(config.gapX ?? 0);
    let localGapY = $state(config.gapY ?? 8);

    function handleCommitX(val: number) {
        localGapX = val;
        config.gapX = val;
        onSave();
    }
    function handleCommitY(val: number) {
        localGapY = val;
        config.gapY = val;
        onSave();
    }

    function changeLayout(v: string) {
        config.layout = v as MangaLayout;
        onSave();
    }

    const pagesPerViewItems = [
        { value: 1, label: i18n.t('reader.single_page') },
        { value: 2, label: i18n.t('reader.double_page') },
    ];

    const readerShortcuts = $derived([
        {
            label: i18n.t('reader.next_page'),
            key: config.direction === 'rtl' ? "ArrowLeft" : "ArrowRight"
        },
        {
            label: i18n.t('reader.prev_page'),
            key: config.direction === 'rtl' ? "ArrowRight" : "ArrowLeft"
        }
    ]);
</script>

<div class="flex flex-col xl:flex-row gap-8 items-start">

    <aside class="w-full xl:w-[450px] xl:sticky xl:top-24 space-y-4">
        <div class="flex items-center justify-between px-1">
            <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                <Monitor class="size-3"/> {i18n.t('settings.readers_section.reader_preview')}
            </Label>
        </div>

        <div class="relative aspect-[3/4] w-full bg-muted/30 rounded-2xl border-2 border-border shadow-inner overflow-hidden flex flex-col">

            <div class="h-7 w-full bg-card border-b border-border flex items-center px-4 justify-between z-10 shadow-sm">
                <div class="h-1.5 w-12 bg-muted-foreground/30 rounded-full"></div>
                <div class="size-2.5 bg-primary rounded-full"></div>
            </div>

            <div class="flex-1 overflow-hidden relative">
                {#if config.layout === 'scroll'}
                    <div class="h-full w-full overflow-y-auto flex flex-col items-center p-4"
                         style="row-gap: {localGapY/3}px">

                        {#each [1, 2, 3] as row}
                            <div class="w-full flex justify-center transition-all duration-300"
                                 style="column-gap: {localGapX/3}px; flex-direction: {config.direction === 'rtl' ? 'row-reverse' : 'row'}">

                                <div class="flex-1 min-w-0 aspect-[2/3] bg-background border-2 border-border rounded-md flex items-center justify-center text-[10px] font-black text-muted-foreground shadow-sm shrink-0">
                                    {i18n.t('settings.page')}
                                </div>

                                {#if config.pagesPerView === 2}
                                    <div class="flex-1 min-w-0 aspect-[2/3] bg-background border-2 border-border rounded-md flex items-center justify-center text-[10px] font-black text-muted-foreground shadow-sm shrink-0" in:fade>
                                        {i18n.t('settings.page')}
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {:else}
                    <div class="w-full h-full flex items-center justify-center p-6 transition-all duration-500"
                         style="column-gap: {localGapX/3}px; flex-direction: {config.direction === 'rtl' ? 'row-reverse' : 'row'}">

                        <div class="flex-1 min-w-0 h-full max-h-[85%] aspect-[2/3] bg-background border-2 border-border rounded-lg flex flex-col items-center justify-center shadow-md relative overflow-hidden" in:fade>
                            <div class="absolute top-3 {config.direction === 'rtl' ? 'right-3' : 'left-3'} size-2 bg-primary/40 rounded-full"></div>
                            <span class="text-xs font-black text-muted-foreground">P. {config.direction === 'rtl' ? (config.pagesPerView === 2 ? '2' : '1') : '1'}</span>
                        </div>

                        {#if config.pagesPerView === 2}
                            <div class="flex-1 min-w-0 h-full max-h-[85%] aspect-[2/3] bg-background border-2 border-border rounded-lg flex flex-col items-center justify-center shadow-md relative overflow-hidden" in:fade>
                                <span class="text-xs font-black text-muted-foreground">P. {config.direction === 'rtl' ? '1' : '2'}</span>
                            </div>
                        {/if}
                    </div>

                    <div class="absolute inset-y-0 left-0 w-8 flex items-center justify-center text-foreground/50"><ChevronLeft class="size-6"/></div>
                    <div class="absolute inset-y-0 right-0 w-8 flex items-center justify-center text-foreground/50"><ChevronRight class="size-6"/></div>
                {/if}
            </div>

            <div class="h-7 w-full bg-card border-t border-border flex items-center px-8 shadow-[0_-2px_10px_rgba(0,0,0,0.05)]">
                <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                    <div class="h-full bg-primary w-1/3"></div>
                </div>
            </div>
        </div>

        <div class="bg-muted/30 border border-border/50 rounded-sm p-3 flex gap-3">
            <Info class="size-4 text-muted-foreground shrink-0 mt-0.5"/>
            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                Preview: <b>{config.layout.toUpperCase()}</b> + <b>{config.pagesPerView === 1 ? 'SINGLE' : 'DOUBLE'}</b>.
                {i18n.t('settings.reader_preview.preview_desc')}
            </p>
        </div>
    </aside>

    <div class="flex-1 w-full space-y-8">
        <div class="grid gap-8">

            <div class="space-y-4">
                <div class="space-y-1">
                    <Label class="text-base font-bold">{i18n.t('reader.reading_mode')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.readers_section.reading_mode_desc')}</p>
                </div>
                <Tabs.Root value={config.layout} onValueChange={changeLayout} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-2 rounded-sm h-12 p-1 bg-muted/50">
                        <Tabs.Trigger value="scroll" class="rounded-lg gap-2 font-bold">
                            <GalleryVertical class="size-4"/> {i18n.t('reader.scroll')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="paged" class="rounded-lg gap-2 font-bold">
                            <BookOpen class="size-4"/> {i18n.t('reader.paged')}
                        </Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="grid sm:grid-cols-2 gap-6">
                <div class="space-y-3">
                    <Label class="font-bold">{i18n.t('settings.readers_section.direction')}</Label>
                    <div class="flex bg-muted/50 p-1 rounded-sm h-11">
                        <Button variant={config.direction === 'ltr' ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { config.direction = 'ltr'; onSave(); }}>LTR</Button>
                        <Button variant={config.direction === 'rtl' ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { config.direction = 'rtl'; onSave(); }}>RTL</Button>
                    </div>
                </div>

                <div class="space-y-3">
                    <Label class="font-bold">{i18n.t('settings.readers_section.pages_per_view')}</Label>
                    <div class="flex bg-muted/50 p-1 rounded-sm h-11">
                        <Button variant={config.pagesPerView === 1 ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { config.pagesPerView = 1; onSave(); }}>
                            {i18n.t('reader.single_page')}
                        </Button>
                        <Button variant={config.pagesPerView === 2 ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { config.pagesPerView = 2; onSave(); }}>
                            {i18n.t('reader.double_page')}
                        </Button>
                    </div>
                </div>
            </div>

            <div class="space-y-3">
                <Label class="font-bold">{i18n.t('reader.image_fit')}</Label>
                <div class="grid grid-cols-2 gap-2">
                    <Button variant={config.fitMode === 'width' ? 'secondary' : 'outline'} class="h-12 rounded-sm gap-2 font-bold" onclick={() => { config.fitMode = 'width'; onSave(); }}>
                        <Maximize class="size-4 rotate-90"/> {i18n.t('reader.fit_width')}
                    </Button>
                    <Button variant={config.fitMode === 'height' ? 'secondary' : 'outline'} class="h-12 rounded-sm gap-2 font-bold" onclick={() => { config.fitMode = 'height'; onSave(); }}>
                        <Maximize class="size-4"/> {i18n.t('reader.fit_height')}
                    </Button>
                </div>
            </div>

            <div class="bg-muted/30 rounded-2xl p-6 border border-border/50 space-y-8">
                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <Label class="font-bold text-sm">{i18n.t('reader.gap_x')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localGapX}px</span>
                    </div>
                    <Slider type="single" bind:value={localGapX} max={100} step={2} onValueCommit={handleCommitX} />
                </div>

                {#if config.layout === 'scroll'}
                    <div class="space-y-4" in:fade>
                        <div class="flex justify-between items-center">
                            <Label class="font-bold text-sm">{i18n.t('reader.gap_y')}</Label>
                            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localGapY}px</span>
                        </div>
                        <Slider type="single" bind:value={localGapY} max={100} step={2} onValueCommit={handleCommitY} />
                    </div>
                {/if}
            </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 border-t border-border/40">
            {#each readerShortcuts as shortcut}
                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium">{shortcut.label}</span>
                        <span class="text-[10px] text-muted-foreground uppercase tracking-wider">
                            {config.direction.toUpperCase()} Mode
                        </span>
                    </div>
                    <Kbd.Group>
                        <Kbd.Root>{shortcut.key}</Kbd.Root>
                    </Kbd.Group>
                </div>
            {/each}
        </div>
    </div>
</div>