<script lang="ts">
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import ArrowLeftRight from 'phosphor-svelte/lib/ArrowsLeftRightIcon';
    import GalleryVertical from 'phosphor-svelte/lib/RowsIcon';
    import BookOpen from 'phosphor-svelte/lib/BookOpenIcon';
    import Maximize from 'phosphor-svelte/lib/CornersOutIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { MangaReaderState } from "@/app/manga.svelte";
    import type { MangaLayout } from "@/api/config/types";

    interface Props {
        readerState: MangaReaderState;
    }

    let { readerState }: Props = $props();
</script>

<div class="space-y-6 px-1">
    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <GalleryVertical class="size-4"/> {i18n.t("reader.reading_mode")}
        </Label>
        <Tabs.Root value={readerState.layout} onValueChange={(v) => readerState.updateMangaConfig({ layout: v as MangaLayout })} class="w-full">
            <Tabs.List class="grid w-full grid-cols-2 rounded-xl h-11 p-1 bg-muted/50">
                <Tabs.Trigger value="scroll" class="rounded-lg gap-2 font-bold h-9"><GalleryVertical class="size-3"/>{i18n.t("reader.scroll")}</Tabs.Trigger>
                <Tabs.Trigger value="paged" class="rounded-lg gap-2 font-bold h-9"><BookOpen class="size-3"/>{i18n.t("reader.paged")}</Tabs.Trigger>
            </Tabs.List>
        </Tabs.Root>
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <ArrowLeftRight class="size-4"/> {i18n.t("reader.direction_and_pages")}
        </Label>
        <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl mb-2">
            <Button variant={readerState.pagesPerView === 1 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => { readerState.updateMangaConfig({ pagesPerView: 1 }); readerState.currentGroupIndex = 0; }}>{i18n.t("reader.single_page")}</Button>
            <Button variant={readerState.pagesPerView === 2 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => { readerState.updateMangaConfig({ pagesPerView: 2 }); readerState.currentGroupIndex = 0; }}>{i18n.t("reader.double_page")}</Button>
        </div>
        <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
            <Button variant={readerState.direction === "ltr" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ direction: "ltr" })}>LTR</Button>
            <Button variant={readerState.direction === "rtl" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ direction: "rtl" })}>RTL</Button>
        </div>
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Maximize class="size-4"/> {i18n.t("reader.image_fit")}
        </Label>
        <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
            <Button variant={readerState.fitMode === "width" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ fitMode: "width" })}>{i18n.t("reader.fit_width")}</Button>
            <Button variant={readerState.fitMode === "height" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ fitMode: "height" })}>{i18n.t("reader.fit_height")}</Button>
        </div>
    </div>

    <div class="space-y-5 pt-4 border-t border-border/40">
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_x")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.gapX}px</span>
            </div>
            <Slider type="single" value={readerState.gapX} onValueChange={(v) => readerState.updateMangaConfig({ gapX: v })} max={100} step={2} class="w-full" />
        </div>
        {#if readerState.layout === "scroll"}
            <div>
                <div class="flex items-center justify-between mb-3">
                    <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_y")}</Label>
                    <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.gapY}px</span>
                </div>
                <Slider type="single" value={readerState.gapY} onValueChange={(v) => readerState.updateMangaConfig({ gapY: v })} max={100} step={2} class="w-full" />
            </div>
        {/if}
    </div>
</div>