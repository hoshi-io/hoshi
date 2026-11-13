<script lang="ts">
    import { NOVEL_THEMES } from "@/app/novel.svelte";
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import Type from 'phosphor-svelte/lib/TextTIcon';
    import AlignLeft from 'phosphor-svelte/lib/TextAlignLeftIcon';
    import AlignJustify from 'phosphor-svelte/lib/TextAlignJustifyIcon';
    import Palette from 'phosphor-svelte/lib/PaletteIcon';
    import Expand from 'phosphor-svelte/lib/ArrowsOutIcon';
    import Baseline from 'phosphor-svelte/lib/TextUnderlineIcon';
    import Space from 'phosphor-svelte/lib/ArrowsOutLineHorizontalIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { NovelReaderState } from "@/app/novel.svelte";
    import type { NovelTheme, FontFamily } from "@/api/config/types";

    interface Props {
        readerState: NovelReaderState;
    }

    let { readerState }: Props = $props();
</script>

<div class="space-y-6 px-1">
    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Palette class="size-4"/> {i18n.t("reader.theme")}
        </Label>
        <div class="grid grid-cols-2 gap-2">
            {#each Object.entries(NOVEL_THEMES) as [t, colors]}
                <Button
                        variant={readerState.theme === t ? "default" : "outline"}
                        class="text-sm h-10 font-bold border-border/50 relative overflow-hidden"
                        style="background-color: {readerState.theme === t ? '' : colors.bg}; color: {readerState.theme === t ? '' : colors.text};"
                        onclick={() => readerState.updateNovelConfig({ theme: t as NovelTheme })}
                >
                    <span class="capitalize">{t}</span>
                </Button>
            {/each}
        </div>
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Type class="size-4"/> {i18n.t("reader.font_family")}
        </Label>
        <Tabs.Root value={readerState.fontFamily} onValueChange={(v) => readerState.updateNovelConfig({ fontFamily: v as FontFamily })} class="w-full">
            <Tabs.List class="grid w-full grid-cols-3 rounded-xl h-11 p-1 bg-muted/50">
                <Tabs.Trigger value="sans" class="rounded-lg font-sans font-bold h-9">Sans</Tabs.Trigger>
                <Tabs.Trigger value="serif" class="rounded-lg font-serif font-bold h-9">Serif</Tabs.Trigger>
                <Tabs.Trigger value="mono" class="rounded-lg font-mono font-bold h-9">Mono</Tabs.Trigger>
            </Tabs.List>
        </Tabs.Root>
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            {i18n.t("reader.alignment")}
        </Label>
        <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
            <Button variant={readerState.textAlign === "left" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ textAlign: "left" })}><AlignLeft class="size-4 mr-2"/> {i18n.t("reader.align_left")}</Button>
            <Button variant={readerState.textAlign === "justify" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ textAlign: "justify" })}><AlignJustify class="size-4 mr-2"/> {i18n.t("reader.align_justify")}</Button>
        </div>
    </div>

    <div class="space-y-5 pt-4 border-t border-border/40">
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Type class="size-3.5"/> {i18n.t("reader.font_size")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.fontSize}px</span>
            </div>
            <Slider type="single" bind:value={readerState.fontSize} min={12} max={32} step={1} class="w-full" />
        </div>
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Baseline class="size-3.5"/> {i18n.t("reader.line_height")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.lineHeight}</span>
            </div>
            <Slider type="single" bind:value={readerState.lineHeight} min={1} max={3} step={0.1} class="w-full" />
        </div>
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Space class="size-3.5"/> {i18n.t("reader.paragraph_spacing")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.paragraphSpacing}em</span>
            </div>
            <Slider type="single" bind:value={readerState.paragraphSpacing} min={0.5} max={4} step={0.1} class="w-full" />
        </div>
        <div>
            <div class="flex items-center justify-between mb-3">
                <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Expand class="size-3.5"/> {i18n.t("reader.content_width")}</Label>
                <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.maxWidth}px</span>
            </div>
            <Slider type="single" bind:value={readerState.maxWidth} min={400} max={1200} step={50} class="w-full" />
        </div>
    </div>
</div>