<script lang="ts">
    import { NOVEL_THEMES } from "@/app/novel.svelte";
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import Type from 'phosphor-svelte/lib/TextTIcon';
    import AlignLeft from 'phosphor-svelte/lib/TextAlignLeftIcon';
    import AlignJustify from 'phosphor-svelte/lib/TextAlignJustifyIcon';
    import Palette from 'phosphor-svelte/lib/PaletteIcon';
    import Expand from 'phosphor-svelte/lib/ArrowsOutIcon';
    import Baseline from 'phosphor-svelte/lib/TextUnderlineIcon';
    import Space from 'phosphor-svelte/lib/ArrowsOutLineHorizontalIcon';
    import Rows from 'phosphor-svelte/lib/RowsIcon';
    import BookOpen from 'phosphor-svelte/lib/BookOpenIcon';
    import { i18n } from "@/stores/i18n.svelte.js";
    import { FONTS, READER_FONTS } from "@/fonts";
    import type { NovelReaderState } from "@/app/novel.svelte";
    import type { NovelTheme } from "@/api/config/types";

    interface Props {
        readerState: NovelReaderState;
    }

    let { readerState }: Props = $props();

    const customBg = $derived(readerState.themeColors.bg);
    const customText = $derived(readerState.themeColors.text);

    function setCustom(key: "customBg" | "customText", value: string) {
        readerState.updateNovelConfig({ theme: "custom", [key]: value });
    }
</script>

{#snippet slider(label: string, icon: any, value: string, children: any)}
    {@const Icon = icon}
    <div>
        <div class="flex items-center justify-between mb-3">
            <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Icon class="size-3.5"/> {label}</Label>
            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{value}</span>
        </div>
        {@render children()}
    </div>
{/snippet}

<div class="space-y-6 px-1">
    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <BookOpen class="size-4"/> {i18n.t("reader.layout")}
        </Label>
        <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
            <Button variant={readerState.layout === "scroll" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ layout: "scroll" })}>{i18n.t("reader.scroll")}</Button>
            <Button variant={readerState.layout === "paged" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ layout: "paged" })}>{i18n.t("reader.paged")}</Button>
        </div>
        {#if readerState.paged}
            <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                <Button variant={readerState.pagesPerView === 1 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ pagesPerView: 1 })}>{i18n.t("reader.single_page")}</Button>
                <Button variant={readerState.pagesPerView >= 2 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateNovelConfig({ pagesPerView: 2 })}>{i18n.t("reader.double_page")}</Button>
            </div>
        {/if}
    </div>

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
        <div class="flex items-center gap-3 rounded-xl border p-2 {readerState.theme === 'custom' ? 'border-primary' : 'border-border/50'}">
            <span class="text-sm font-bold flex-1 px-1">{i18n.t("reader.theme_custom")}</span>
            <label class="flex items-center gap-1.5 text-xs text-muted-foreground">
                {i18n.t("reader.background")}
                <input type="color" value={customBg} onchange={(e) => setCustom("customBg", e.currentTarget.value)} class="size-8 cursor-pointer rounded border-0 bg-transparent p-0" />
            </label>
            <label class="flex items-center gap-1.5 text-xs text-muted-foreground">
                {i18n.t("reader.text_color")}
                <input type="color" value={customText} onchange={(e) => setCustom("customText", e.currentTarget.value)} class="size-8 cursor-pointer rounded border-0 bg-transparent p-0" />
            </label>
        </div>
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Type class="size-4"/> {i18n.t("reader.font_family")}
        </Label>
        <div class="grid grid-cols-2 gap-2">
            {#each READER_FONTS as id (id)}
                <button
                        type="button"
                        aria-pressed={readerState.fontFamily === id}
                        onclick={() => readerState.updateNovelConfig({ fontFamily: id })}
                        class="h-10 truncate rounded-lg border px-3 text-left text-sm transition-colors
                               {readerState.fontFamily === id ? 'border-primary bg-primary/10 font-semibold' : 'border-border/50 hover:bg-muted/50'}"
                        style="font-family: {FONTS[id].stack}"
                >
                    {FONTS[id].label}
                </button>
            {/each}
        </div>
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
        {#snippet fontSizeSlider()}<Slider type="single" bind:value={readerState.fontSize} min={12} max={32} step={1} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.font_size"), Type, `${readerState.fontSize}px`, fontSizeSlider)}

        {#snippet lineSlider()}<Slider type="single" bind:value={readerState.lineHeight} min={1} max={3} step={0.1} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.line_height"), Baseline, `${readerState.lineHeight}`, lineSlider)}

        {#snippet paragraphSlider()}<Slider type="single" bind:value={readerState.paragraphSpacing} min={0.5} max={4} step={0.1} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.paragraph_spacing"), Rows, `${readerState.paragraphSpacing}em`, paragraphSlider)}

        {#snippet letterSlider()}<Slider type="single" bind:value={readerState.letterSpacing} min={-0.05} max={0.2} step={0.01} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.letter_spacing"), Space, `${readerState.letterSpacing.toFixed(2)}em`, letterSlider)}

        {#snippet wordSlider()}<Slider type="single" bind:value={readerState.wordSpacing} min={0} max={0.5} step={0.02} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.word_spacing"), Space, `${readerState.wordSpacing.toFixed(2)}em`, wordSlider)}

        {#snippet widthSlider()}<Slider type="single" bind:value={readerState.maxWidth} min={400} max={1200} step={50} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.content_width"), Expand, `${readerState.maxWidth}px`, widthSlider)}

        {#snippet marginSlider()}<Slider type="single" bind:value={readerState.margin} min={0} max={96} step={4} class="w-full" />{/snippet}
        {@render slider(i18n.t("reader.margins"), Expand, `${readerState.margin}px`, marginSlider)}
    </div>
</div>
