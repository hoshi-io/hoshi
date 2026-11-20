<script lang="ts">
    import { commonReaderShortcuts } from "@/utils/readerShortcuts";
    import { Label } from "@/components/ui/label";
    import SettingSlider from "@/components/settings/parts/SettingSlider.svelte";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";
    import Stepper from "@/components/settings/parts/Stepper.svelte";
    import Ruler from 'phosphor-svelte/lib/RulerIcon';
    import Sliders from 'phosphor-svelte/lib/SlidersHorizontalIcon';
    import * as Kbd from "@/components/ui/kbd/index";
    import Type from 'phosphor-svelte/lib/TextTIcon';
    import AlignLeft from 'phosphor-svelte/lib/TextAlignLeftIcon';
    import AlignJustify from 'phosphor-svelte/lib/TextAlignJustifyIcon';
    import Palette from 'phosphor-svelte/lib/PaletteIcon';
    import Monitor from 'phosphor-svelte/lib/MonitorIcon';
    import Info from 'phosphor-svelte/lib/InfoIcon';
    import type { NovelConfig, NovelTheme, FontFamily } from "@/api/config/types";
    import { i18n } from '@/stores/i18n.svelte.js';
    import { NOVEL_THEMES, novelThemeColors } from "@/app/novel.svelte";
    import { FONTS, READER_FONTS, fontStack } from "@/utils/fonts";

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
    /** Spacing in em reads better as a signed percentage ("+4%", "0%"). */
    const percent = (v: number) => `${v > 0 ? "+" : ""}${Math.round(v * 100)}%`;


    const novelShortcuts: { label: string; keys: string[]; hint?: string }[] = [
        { label: i18n.t('reader.scroll_down'), keys: ["↓"] },
        { label: i18n.t('reader.scroll_up'), keys: ["↑"] },
        { label: i18n.t('reader.page_scroll'), keys: ["Space"]},
        { label: i18n.t('reader.turn_page'), keys: ["←", "→"] },
        { label: i18n.t('reader.increase_font'), keys: ["+", "="] },
        { label: i18n.t('reader.decrease_font'), keys: ["-"] },
        ...commonReaderShortcuts(),
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
                        <button
                                type="button"
                                aria-pressed={config.theme === t}
                                class="h-12 rounded-sm border text-xs font-bold capitalize transition-shadow
                                       {config.theme === t ? 'border-primary ring-2 ring-primary' : 'border-border/50'}"
                                style="background-color: {swatch.bg}; color: {swatch.text};"
                                onclick={() => { config.theme = t as NovelTheme; onSave(); }}
                        >
                            {t}
                        </button>
                    {/each}
                </div>
                <div class="flex flex-wrap items-center gap-4 rounded-sm border px-3 py-2 {config.theme === 'custom' ? 'border-primary ring-2 ring-primary' : 'border-border/50'}">
                    <span class="text-sm font-bold flex-1">{i18n.t('reader.theme_custom')}</span>
                    <label class="flex items-center gap-2 text-xs text-muted-foreground">
                        {i18n.t('reader.background')}
                        <input type="color" value={config.customBg ?? '#1e1e24'} onchange={(e) => setCustom('customBg', e.currentTarget.value)} class="size-8 cursor-pointer rounded-sm border-0 bg-transparent p-0" />
                    </label>
                    <label class="flex items-center gap-2 text-xs text-muted-foreground">
                        {i18n.t('reader.text_color')}
                        <input type="color" value={config.customText ?? '#e4e4e7'} onchange={(e) => setCustom('customText', e.currentTarget.value)} class="size-8 cursor-pointer rounded-sm border-0 bg-transparent p-0" />
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

                <div class="space-y-8">
                    <div class="space-y-4">
                        <Label class="font-bold flex items-center gap-2">{i18n.t('reader.alignment')}</Label>
                        <SegmentedControl fluid label={i18n.t('reader.alignment')} value={config.textAlign} options={alignOptions} onValueChange={changeAlign} />
                    </div>
                    <div class="space-y-4">
                        <Label class="font-bold flex items-center gap-2"><Type class="size-4 text-primary"/> {i18n.t('reader.text')}</Label>
                        <div class="space-y-3 rounded-sm border border-border/50 bg-muted/20 p-4">
                            <Stepper label={i18n.t('reader.font_size')} value={localSize} min={12} max={32} step={1} onChange={(v) => { localSize = v; handleCommitSize(v); }} />
                            <Stepper label={i18n.t('reader.line_height')} value={localLine} min={1} max={3} step={0.1} format={(v) => v.toFixed(1)} onChange={(v) => { localLine = v; handleCommitLine(v); }} />
                            <Stepper label={i18n.t('reader.paragraph_spacing')} value={localSpacing} min={0.5} max={4} step={0.1} format={(v) => v.toFixed(1)} onChange={(v) => { localSpacing = v; handleCommitSpacing(v); }} />
                        </div>
                    </div>
                </div>
            </div>

            <div class="grid gap-8 sm:grid-cols-2">
                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2"><Ruler class="size-4 text-primary"/> {i18n.t('reader.page_size')}</Label>
                    <div class="space-y-5 rounded-sm border border-border/50 bg-muted/20 p-4">
                        <div class="space-y-3">
                            <div class="flex items-center justify-between gap-3">
                                <span class="text-sm font-medium">{i18n.t('reader.content_width')}</span>
                                <span class="font-mono text-sm font-semibold tabular-nums text-muted-foreground">{localWidth}px</span>
                            </div>
                            <SettingSlider fluid showValue={false} bind:value={localWidth} min={400} max={1200} step={50} oncommit={handleCommitWidth} />
                        </div>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between gap-3">
                                <span class="text-sm font-medium">{i18n.t('reader.margins')}</span>
                                <span class="font-mono text-sm font-semibold tabular-nums text-muted-foreground">{localMargin}px</span>
                            </div>
                            <SettingSlider fluid showValue={false} bind:value={localMargin} min={0} max={96} step={4} oncommit={handleCommitMargin} />
                        </div>
                    </div>
                </div>

                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2"><Sliders class="size-4 text-primary"/> {i18n.t('reader.advanced')}</Label>
                    <div class="space-y-3 rounded-sm border border-border/50 bg-muted/20 p-4">
                        <Stepper label={i18n.t('reader.letter_spacing')} value={localLetter} min={-0.05} max={0.2} step={0.01} format={percent} onChange={(v) => { localLetter = v; handleCommitLetter(v); }} />
                        <Stepper label={i18n.t('reader.word_spacing')} value={localWord} min={0} max={0.5} step={0.02} format={percent} onChange={(v) => { localWord = v; handleCommitWord(v); }} />
                    </div>
                </div>
            </div>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 border-t border-border/40">
            {#each novelShortcuts as shortcut}
                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium">{shortcut.label}</span>
                        {#if shortcut.hint}
                            <span class="text-[10px] text-muted-foreground uppercase tracking-wider">{shortcut.hint}</span>
                        {/if}
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