<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import SettingSlider from "@/components/settings/parts/SettingSlider.svelte";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";
    import * as Kbd from "$lib/components/ui/kbd/index.js";
    import Type from 'phosphor-svelte/lib/TextTIcon';
    import AlignLeft from 'phosphor-svelte/lib/TextAlignLeftIcon';
    import AlignJustify from 'phosphor-svelte/lib/TextAlignJustifyIcon';
    import Palette from 'phosphor-svelte/lib/PaletteIcon';
    import Expand from 'phosphor-svelte/lib/ArrowsOutIcon';
    import Monitor from 'phosphor-svelte/lib/MonitorIcon';
    import Info from 'phosphor-svelte/lib/InfoIcon';
    import ALargeSmall from 'phosphor-svelte/lib/TextAaIcon';
    import Baseline from 'phosphor-svelte/lib/TextUnderlineIcon';
    import Space from 'phosphor-svelte/lib/ArrowsOutLineHorizontalIcon';
    import type { NovelConfig, NovelTheme, FontFamily } from "@/api/config/types";
    import { i18n } from '@/stores/i18n.svelte.js';
    import { NOVEL_THEMES, novelThemeColors } from "@/app/novel.svelte";
    import { FONTS, READER_FONTS, fontStack } from "@/fonts";

    let {
        config = $bindable(),
        onSave
    }: {
        config: NovelConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let localSize = $state(config.fontSize ?? 16);
    let localLine = $state(config.lineHeight ?? 1.6);
    let localSpacing = $state(config.paragraphSpacing ?? 1.5);
    let localWidth = $state(config.maxWidth ?? 700);
    let localMargin = $state(config.margin ?? 24);
    let localLetter = $state(config.letterSpacing ?? 0);
    let localWord = $state(config.wordSpacing ?? 0);

    $effect(() => {
        if (config.fontSize) localSize = config.fontSize;
        if (config.lineHeight) localLine = config.lineHeight;
        if (config.paragraphSpacing) localSpacing = config.paragraphSpacing;
        if (config.maxWidth) localWidth = config.maxWidth;
        localMargin = config.margin ?? 24;
        localLetter = config.letterSpacing ?? 0;
        localWord = config.wordSpacing ?? 0;
    });

    const colors = $derived(novelThemeColors(config));


    const novelShortcuts = [
        { label: i18n.t('reader.scroll_down'), keys: ["↓"] },
        { label: i18n.t('reader.scroll_up'), keys: ["↑"] },
        { label: i18n.t('reader.page_scroll'), keys: ["Space"]},
        { label: i18n.t('reader.turn_page'), keys: ["←", "→"] },
        { label: i18n.t('reader.increase_font'), keys: ["+", "="] },
        { label: i18n.t('reader.decrease_font'), keys: ["-"] },
    ];


    const alignOptions = [
        { value: "left", label: i18n.t('reader.align_left'), icon: AlignLeft },
        { value: "justify", label: i18n.t('reader.align_justify'), icon: AlignJustify }
    ];

    function changeFontFamily(v: string) { config.fontFamily = v as FontFamily; onSave(); }
    function changeAlign(v: string) { config.textAlign = v as NovelConfig['textAlign']; onSave(); }

    function handleCommitSize(val: number) { config.fontSize = val; onSave(); }
    function handleCommitLine(val: number) { config.lineHeight = val; onSave(); }
    function handleCommitSpacing(val: number) { config.paragraphSpacing = val; onSave(); }
    function handleCommitWidth(val: number) { config.maxWidth = val; onSave(); }
    function handleCommitMargin(val: number) { config.margin = val; onSave(); }
    function handleCommitLetter(val: number) { config.letterSpacing = val; onSave(); }
    function handleCommitWord(val: number) { config.wordSpacing = val; onSave(); }

    const layoutOptions = [
        { value: "scroll", label: i18n.t('reader.scroll') },
        { value: "paged", label: i18n.t('reader.paged') }
    ];
    const spreadOptions = [
        { value: "1", label: i18n.t('reader.single_page') },
        { value: "2", label: i18n.t('reader.double_page') }
    ];
    function changeLayout(v: string) { config.layout = v as NovelConfig['layout']; onSave(); }
    function changeSpread(v: string) { config.pagesPerView = Number(v); onSave(); }
    function setCustom(key: 'customBg' | 'customText', value: string) { config[key] = value; config.theme = 'custom'; onSave(); }
</script>

<div class="flex flex-col xl:flex-row gap-8 items-start">

    <aside class="w-full xl:w-[450px] xl:sticky xl:top-24 space-y-4">
        <div class="flex items-center justify-between px-1">
            <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                <Monitor class="size-3"/> {i18n.t('settings.readers_section.novel_reader_preview')}
            </Label>
        </div>

        <div
                class="relative aspect-[3/4] w-full rounded-2xl border-4 shadow-2xl overflow-hidden flex flex-col transition-colors duration-500"
                style="background-color: {colors.bg}; border-color: {colors.border}; color: {colors.text};"
        >
            <div class="h-8 w-full border-b flex items-center px-4 justify-between opacity-40" style="border-color: {colors.border};">
                <div class="h-1.5 w-16 bg-current rounded-full"></div>
                <div class="h-1.5 w-4 bg-current rounded-full"></div>
            </div>

            <div class="flex-1 overflow-hidden p-6 text-sm">
                <article
                        class="mx-auto h-full transition-all duration-300 preview-content"
                        style="
                        max-width: 100%;
                        font-size: {localSize * 0.6}px;
                        line-height: {localLine};
                        text-align: {config.textAlign};
                        font-family: {fontStack(config.fontFamily)};
                        letter-spacing: {localLetter}em;
                        word-spacing: {localWord}em;
                        padding-inline: {localMargin * 0.4}px;
                        --preview-spacing: {localSpacing}em;
                    "
                >
                    <h3 class="font-bold opacity-90" style="font-size: 1.4em; margin-bottom: var(--preview-spacing)">Preview</h3>
                    <p>
                        Había una vez en un lugar muy lejano, un lector que buscaba la configuración perfecta.
                        Ajustó la fuente, cambió los colores y encontró la paz en cada línea.
                    </p>
                    <p class="opacity-80">
                        La tipografía no es solo legibilidad, es la voz del autor susurrando directamente
                        a tu imaginación a través de una pantalla.
                    </p>
                </article>
            </div>

            <div class="h-6 w-full flex items-center justify-center text-[10px] opacity-40 font-mono">
                — 42 —
            </div>
        </div>

        <div class="bg-muted/30 border border-border/50 rounded-sm p-3 flex gap-3">
            <Info class="size-4 text-muted-foreground shrink-0 mt-0.5"/>
            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                {i18n.t('settings.readers_section.novel_preview_desc', {width: localWidth})}
            </p>
        </div>
    </aside>

    <div class="flex-1 w-full space-y-8">
        <div class="grid gap-8">
            <div class="space-y-4">
                <div class="flex items-center gap-2">
                    <Palette class="size-5 text-primary"/>
                    <Label class="text-base font-bold">{i18n.t('reader.theme')}</Label>
                </div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                    {#each Object.entries(NOVEL_THEMES) as [t, swatch]}
                        <Button
                                variant={config.theme === t ? 'default' : 'outline'}
                                class="h-12 rounded-sm flex flex-col gap-0.5 relative overflow-hidden transition-colors"
                                style="background-color: {config.theme === t ? '' : swatch.bg}; color: {config.theme === t ? '' : swatch.text};"
                                onclick={() => { config.theme = t as NovelTheme; onSave(); }}
                        >
                            <span class="text-xs font-bold capitalize">{t}</span>
                        </Button>
                    {/each}
                </div>
                <div class="flex flex-wrap items-center gap-4 rounded-sm border-2 px-3 py-2 {config.theme === 'custom' ? 'border-primary' : 'border-border/50'}">
                    <span class="text-sm font-bold flex-1">{i18n.t('reader.theme_custom')}</span>
                    <label class="flex items-center gap-2 text-xs text-muted-foreground">
                        {i18n.t('reader.background')}
                        <input type="color" value={config.customBg ?? '#1e1e24'} onchange={(e) => setCustom('customBg', e.currentTarget.value)} class="size-8 cursor-pointer rounded border-0 bg-transparent p-0" />
                    </label>
                    <label class="flex items-center gap-2 text-xs text-muted-foreground">
                        {i18n.t('reader.text_color')}
                        <input type="color" value={config.customText ?? '#e4e4e7'} onchange={(e) => setCustom('customText', e.currentTarget.value)} class="size-8 cursor-pointer rounded border-0 bg-transparent p-0" />
                    </label>
                </div>
            </div>

            <div class="grid sm:grid-cols-2 gap-8">
                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2">{i18n.t('reader.layout')}</Label>
                    <SegmentedControl fluid label={i18n.t('reader.layout')} value={config.layout ?? 'scroll'} options={layoutOptions} onValueChange={changeLayout} />
                </div>
                {#if config.layout === 'paged'}
                    <div class="space-y-4">
                        <Label class="font-bold flex items-center gap-2">{i18n.t('reader.pages_per_view')}</Label>
                        <SegmentedControl fluid label={i18n.t('reader.pages_per_view')} value={String(config.pagesPerView ?? 2)} options={spreadOptions} onValueChange={changeSpread} />
                    </div>
                {/if}
            </div>

            <div class="grid sm:grid-cols-2 gap-8">
                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2"><Type class="size-4 text-primary"/> {i18n.t('reader.font_family')}</Label>
                    <div class="grid grid-cols-2 gap-2">
                        {#each READER_FONTS as id (id)}
                            <button
                                    type="button"
                                    aria-pressed={config.fontFamily === id}
                                    onclick={() => changeFontFamily(id)}
                                    class="h-10 truncate rounded-sm border-2 px-3 text-left text-sm transition-colors
                                           {config.fontFamily === id ? 'border-primary bg-primary/10 font-semibold' : 'border-border/50 hover:bg-muted/50'}"
                                    style="font-family: {FONTS[id].stack}"
                            >{FONTS[id].label}</button>
                        {/each}
                    </div>
                </div>

                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2">{i18n.t('reader.alignment')}</Label>
                    <div>
                        <SegmentedControl
                                fluid
                                label={i18n.t('reader.alignment')}
                                value={config.textAlign}
                                options={alignOptions}
                                onValueChange={changeAlign}
                        />
                    </div>
                </div>
            </div>

            <div class="bg-muted/30 rounded-2xl p-6 border border-border/50 space-y-10">

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <ALargeSmall class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.font_size')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localSize}px</span>
                    </div>
                    <SettingSlider fluid showValue={false} bind:value={localSize} min={12} max={32} step={1} oncommit={handleCommitSize} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Baseline class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.line_height')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localLine}</span>
                    </div>
                    <SettingSlider fluid showValue={false} bind:value={localLine} min={1} max={3} step={0.1} oncommit={handleCommitLine} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Space class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.paragraph_spacing')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localSpacing}em</span>
                    </div>
                    <SettingSlider fluid showValue={false} bind:value={localSpacing} min={0.5} max={4} step={0.1} oncommit={handleCommitSpacing} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Expand class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.content_width')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localWidth}px</span>
                    </div>
                    <SettingSlider fluid showValue={false} bind:value={localWidth} min={400} max={1200} step={50} oncommit={handleCommitWidth} />
                </div>

                {#each [
                    { label: i18n.t('reader.margins'), unit: 'px', get: () => localMargin, set: (v: number) => (localMargin = v), min: 0, max: 96, step: 4, commit: handleCommitMargin, fmt: (v: number) => `${v}` },
                    { label: i18n.t('reader.letter_spacing'), unit: 'em', get: () => localLetter, set: (v: number) => (localLetter = v), min: -0.05, max: 0.2, step: 0.01, commit: handleCommitLetter, fmt: (v: number) => v.toFixed(2) },
                    { label: i18n.t('reader.word_spacing'), unit: 'em', get: () => localWord, set: (v: number) => (localWord = v), min: 0, max: 0.5, step: 0.02, commit: handleCommitWord, fmt: (v: number) => v.toFixed(2) },
                ] as row (row.label)}
                    <div class="space-y-4">
                        <div class="flex justify-between items-center">
                            <div class="flex items-center gap-2">
                                <Space class="size-4 text-primary"/>
                                <Label class="font-bold text-sm">{row.label}</Label>
                            </div>
                            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{row.fmt(row.get())}{row.unit}</span>
                        </div>
                        <SettingSlider fluid showValue={false} bind:value={row.get, row.set} min={row.min} max={row.max} step={row.step} oncommit={row.commit} />
                    </div>
                {/each}
            </div>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 border-t border-border/40">
            {#each novelShortcuts as shortcut}
                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium">{shortcut.label}</span>
                    </div>

                    <Kbd.Group>
                        {#each shortcut.keys as key, i}
                            <Kbd.Root>{key}</Kbd.Root>
                            {#if i < shortcut.keys.length - 1}
                                <span class="text-xs text-muted-foreground/50 mx-1">/</span>
                            {/if}
                        {/each}
                    </Kbd.Group>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .preview-content p {
        margin-bottom: var(--preview-spacing, 1.5em);
    }
</style>