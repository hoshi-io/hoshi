<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import { Slider } from "$lib/components/ui/slider";
    import * as Kbd from "$lib/components/ui/kbd/index.js";
    import {
        Type,
        AlignLeft,
        AlignJustify,
        Palette,
        Expand,
        Monitor,
        Info,
        ALargeSmall,
        Baseline,
        Space
    } from "lucide-svelte";
    import type { NovelConfig, NovelTheme, FontFamily } from "@/api/config/types";
    import { i18n } from '@/stores/i18n.svelte.js';

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

    $effect(() => {
        if (config.fontSize) localSize = config.fontSize;
        if (config.lineHeight) localLine = config.lineHeight;
        if (config.paragraphSpacing) localSpacing = config.paragraphSpacing;
        if (config.maxWidth) localWidth = config.maxWidth;
    });

    const themes = {
        light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
        dark: { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
        sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
        oled: { bg: "#000000", text: "#d1d5db", border: "#171717" }
    };

    const novelShortcuts = [
        { label: i18n.t('reader.scroll_down'), keys: ["↓"] },
        { label: i18n.t('reader.scroll_up'), keys: ["↑"] },
        { label: i18n.t('reader.page_scroll'), keys: ["Space"]},
        { label: i18n.t('reader.increase_font'), keys: ["+", "="] },
        { label: i18n.t('reader.decrease_font'), keys: ["-"] },
    ];

    function handleCommitSize(val: number) { config.fontSize = val; onSave(); }
    function handleCommitLine(val: number) { config.lineHeight = val; onSave(); }
    function handleCommitSpacing(val: number) { config.paragraphSpacing = val; onSave(); }
    function handleCommitWidth(val: number) { config.maxWidth = val; onSave(); }
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
                style="background-color: {themes[config.theme as keyof typeof themes]?.bg || themes.dark.bg}; border-color: {themes[config.theme as keyof typeof themes]?.border || themes.dark.border}; color: {themes[config.theme as keyof typeof themes]?.text || themes.dark.text};"
        >
            <div class="h-8 w-full border-b flex items-center px-4 justify-between opacity-40" style="border-color: {themes[config.theme as keyof typeof themes]?.border || themes.dark.border};">
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
                        font-family: {config.fontFamily === 'sans' ? 'sans-serif' : config.fontFamily === 'serif' ? 'serif' : 'monospace'};
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
                {i18n.t('settings.novel_preview_desc.novel_preview_desc', {width: localWidth})}
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
                    {#each Object.entries(themes) as [t, colors]}
                        <Button
                                variant={config.theme === t ? 'default' : 'outline'}
                                class="h-12 rounded-sm flex flex-col gap-0.5 relative overflow-hidden transition-colors"
                                style="background-color: {config.theme === t ? '' : colors.bg}; color: {config.theme === t ? '' : colors.text};"
                                onclick={() => { config.theme = t as NovelTheme; onSave(); }}
                        >
                            <span class="text-xs font-bold capitalize">{t}</span>
                        </Button>
                    {/each}
                </div>
            </div>

            <div class="grid sm:grid-cols-2 gap-8">
                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2"><Type class="size-4 text-primary"/> {i18n.t('reader.font_family')}</Label>
                    <Tabs.Root value={config.fontFamily} onValueChange={(v) => { config.fontFamily = v as FontFamily; onSave(); }}>
                        <Tabs.List class="grid w-full grid-cols-3 rounded-sm h-11 p-1 bg-muted/50">
                            <Tabs.Trigger value="sans" class="rounded-lg font-sans font-bold">Sans</Tabs.Trigger>
                            <Tabs.Trigger value="serif" class="rounded-lg font-serif font-bold">Serif</Tabs.Trigger>
                            <Tabs.Trigger value="mono" class="rounded-lg font-mono font-bold">Mono</Tabs.Trigger>
                        </Tabs.List>
                    </Tabs.Root>
                </div>

                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2">{i18n.t('reader.alignment')}</Label>
                    <div class="flex bg-muted/50 p-1 rounded-sm h-11">
                        <Button
                                variant={config.textAlign === 'left' ? 'secondary' : 'ghost'}
                                class="flex-1 rounded-lg font-bold"
                                onclick={() => { config.textAlign = 'left'; onSave(); }}
                        >
                            <AlignLeft class="size-4 mr-2"/> {i18n.t('reader.align_left')}
                        </Button>
                        <Button
                                variant={config.textAlign === 'justify' ? 'secondary' : 'ghost'}
                                class="flex-1 rounded-lg font-bold"
                                onclick={() => { config.textAlign = 'justify'; onSave(); }}
                        >
                            <AlignJustify class="size-4 mr-2"/> {i18n.t('reader.align_justify')}
                        </Button>
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
                    <Slider type="single" bind:value={localSize} min={12} max={32} step={1} onValueCommit={handleCommitSize} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Baseline class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.line_height')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localLine}</span>
                    </div>
                    <Slider type="single" bind:value={localLine} min={1} max={3} step={0.1} onValueCommit={handleCommitLine} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Space class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('reader.paragraph_spacing')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localSpacing}em</span>
                    </div>
                    <Slider type="single" bind:value={localSpacing} min={0.5} max={4} step={0.1} onValueCommit={handleCommitSpacing} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Expand class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('content_width')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localWidth}px</span>
                    </div>
                    <Slider type="single" bind:value={localWidth} min={400} max={1200} step={50} onValueCommit={handleCommitWidth} />
                </div>
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