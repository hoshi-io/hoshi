<script lang="ts">
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";
    import { NOVEL_THEMES } from "@/app/novel.svelte";
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
    import type { NovelLayout, NovelTheme, TextAlign } from "@/api/config/types";

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
            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-sm">{value}</span>
        </div>
        {@render children()}
    </div>
{/snippet}

<div class="space-y-6 px-1">
    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <BookOpen class="size-4"/> {i18n.t("reader.layout")}
        </Label>
        <SegmentedControl
                fluid
                value={readerState.layout}
                options={[
                    { value: "scroll", label: i18n.t("reader.scroll") },
                    { value: "paged", label: i18n.t("reader.paged") },
                ]}
                onValueChange={(v) => readerState.updateNovelConfig({ layout: v as NovelLayout })}
        />
        {#if readerState.paged}
            <SegmentedControl
                fluid
                    value={readerState.pagesPerView >= 2 ? "2" : "1"}
                    options={[
                        { value: "1", label: i18n.t("reader.single_page") },
                        { value: "2", label: i18n.t("reader.double_page") },
                    ]}
                    onValueChange={(v) => readerState.updateNovelConfig({ pagesPerView: Number(v) })}
            />
        {/if}
    </div>

    <div class="space-y-3">
        <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
            <Palette class="size-4"/> {i18n.t("reader.theme")}
        </Label>
        <div class="grid grid-cols-2 gap-2">
            {#each Object.entries(NOVEL_THEMES) as [t, colors]}
                <button
                        type="button"
                        aria-pressed={readerState.theme === t}
                        class="h-10 rounded-sm border text-sm font-bold transition-shadow
                               {readerState.theme === t ? 'border-primary ring-2 ring-primary' : 'border-border/50'}"
                        style="background-color: {colors.bg}; color: {colors.text};"
                        onclick={() => readerState.updateNovelConfig({ theme: t as NovelTheme })}
                >
                    <span class="capitalize">{t}</span>
                </button>
            {/each}
        </div>
        <div class="flex items-center gap-3 rounded-sm border p-2 {readerState.theme === 'custom' ? 'border-primary ring-2 ring-primary' : 'border-border/50'}">
            <span class="text-sm font-bold flex-1 px-1">{i18n.t("reader.theme_custom")}</span>
            <label class="flex items-center gap-1.5 text-xs text-muted-foreground">
                {i18n.t("reader.background")}
                <input type="color" value={customBg} onchange={(e) => setCustom("customBg", e.currentTarget.value)} class="size-8 cursor-pointer rounded-sm border-0 bg-transparent p-0" />
            </label>
            <label class="flex items-center gap-1.5 text-xs text-muted-foreground">
                {i18n.t("reader.text_color")}
                <input type="color" value={customText} onchange={(e) => setCustom("customText", e.currentTarget.value)} class="size-8 cursor-pointer rounded-sm border-0 bg-transparent p-0" />
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
                        class="h-10 truncate rounded-sm border px-3 text-left text-sm transition-colors
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
        <SegmentedControl
                fluid
                value={readerState.textAlign}
                options={[
                    { value: "left", label: i18n.t("reader.align_left"), icon: AlignLeft },
                    { value: "justify", label: i18n.t("reader.align_justify"), icon: AlignJustify },
                ]}
                onValueChange={(v) => readerState.updateNovelConfig({ textAlign: v as TextAlign })}
        />
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
