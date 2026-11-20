<script lang="ts">
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";
    import ArrowLeftRight from 'phosphor-svelte/lib/ArrowsLeftRightIcon';
    import GalleryVertical from 'phosphor-svelte/lib/RowsIcon';
    import BookOpen from 'phosphor-svelte/lib/BookOpenIcon';
    import Maximize from 'phosphor-svelte/lib/CornersOutIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { MangaReaderState } from "@/app/manga.svelte";
    import type { FitMode, MangaLayout, ReadingDirection } from "@/api/config/types";

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
<SegmentedControl
                fluid
                value={readerState.layout}
                options={[
                    { value: "scroll", label: i18n.t("reader.scroll"), icon: GalleryVertical },
                    { value: "paged", label: i18n.t("reader.paged"), icon: BookOpen },
                ]}
                onValueChange={(v) => readerState.updateMangaConfig({ layout: v as MangaLayout })}
        />
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <ArrowLeftRight class="size-4"/> {i18n.t("reader.direction_and_pages")}
        </Label>
        <SegmentedControl
                fluid
                value={String(readerState.pagesPerView)}
                options={[
                    { value: "1", label: i18n.t("reader.single_page") },
                    { value: "2", label: i18n.t("reader.double_page") },
                ]}
                onValueChange={(v) => { readerState.updateMangaConfig({ pagesPerView: Number(v) }); readerState.currentGroupIndex = 0; }}
        />
        <SegmentedControl
                fluid
                value={readerState.direction}
                options={[
                    { value: "ltr", label: "LTR" },
                    { value: "rtl", label: "RTL" },
                ]}
                onValueChange={(v) => readerState.updateMangaConfig({ direction: v as ReadingDirection })}
        />
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Maximize class="size-4"/> {i18n.t("reader.image_fit")}
        </Label>
        <SegmentedControl
                fluid
                value={readerState.fitMode}
                options={[
                    { value: "width", label: i18n.t("reader.fit_width") },
                    { value: "height", label: i18n.t("reader.fit_height") },
                ]}
                onValueChange={(v) => readerState.updateMangaConfig({ fitMode: v as FitMode })}
        />
    </div>

    <div class="space-y-5 pt-4 border-t border-border/40">
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_x")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-sm">{readerState.gapX}px</span>
            </div>
            <Slider type="single" value={readerState.gapX} onValueChange={(v) => readerState.updateMangaConfig({ gapX: v })} max={100} step={2} class="w-full" />
        </div>
        {#if readerState.layout === "scroll"}
            <div>
                <div class="flex items-center justify-between mb-3">
                    <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_y")}</Label>
                    <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-sm">{readerState.gapY}px</span>
                </div>
                <Slider type="single" value={readerState.gapY} onValueChange={(v) => readerState.updateMangaConfig({ gapY: v })} max={100} step={2} class="w-full" />
            </div>
        {/if}
    </div>
</div>