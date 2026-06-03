<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { UiConfig } from "@/api/config/types";
    import {i18n} from "@/stores/i18n.svelte.js";
    import {themeManager} from "@/stores/theme.svelte";
    import {Check, Palette} from "lucide-svelte";
    import {Input} from "@/components/ui/input";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: UiConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950 border-zinc-200' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50 border-zinc-800' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white border-zinc-900' }
    ];

    const colorPresets = [
        { name: 'Purple', value: '#a855f7' },
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Cyan', value: '#06b6d4' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Amber', value: '#f59e0b' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Pink', value: '#ec4899' },
    ];

    function changeTheme(themeId: string) {
        themeManager.setTheme(themeId);
    }

    function setPresetColor(color: string) {
        themeManager.setAccentColor(color);
    }

    function handleCustomColor(event: Event) {
        const input = event.target as HTMLInputElement;
        themeManager.setAccentColor(input.value);
    }

    const titleLanguageItems = [
        { value: "romaji", label: i18n.t('settings.ui_section.title_language_romaji') },
        { value: "english", label: i18n.t('settings.ui_section.title_language_english') },
        { value: "native", label: i18n.t('settings.ui_section.title_language_native') },
        { value: "chinese", label: i18n.t('settings.ui_section.title_language_chinese') },
    ];

    const homeSectionItems = [
        { value: "anime", label: i18n.t('settings.ui_section.default_home_anime') },
        { value: "manga", label: i18n.t('settings.ui_section.default_home_manga') },
        { value: "novel", label: i18n.t('settings.ui_section.default_home_novel') }
    ];
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.interface')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.ui_section.interface_desc')}</p>
    </div>

    <div class="flex flex-col gap-4 py-6 border-b border-border/40">
        <div class="space-y-1">
            <Label class="text-base font-bold flex items-center gap-2">
                <Palette class="size-4" /> {i18n.t('settings.general_section.theme')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.general_section.theme_desc')}</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
            {#each themes as theme}
                <button
                        type="button"
                        onclick={() => changeTheme(theme.id)}
                        class="relative flex items-center justify-center h-14 rounded-sm border-2 font-bold transition-all overflow-hidden {theme.classes}
        {themeManager.theme === theme.id
            ? 'ring-2 ring-primary ring-offset-2 ring-offset-background border-transparent'
            : 'opacity-80 hover:opacity-100 border-transparent'}"
                >
                    {theme.label}
                    {#if themeManager.theme === theme.id}
                        <div class="absolute top-1.5 right-1.5 bg-primary rounded-full p-0.5">
                            <Check class="size-3 text-primary-foreground" />
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    </div>

    <div class="flex flex-col gap-4 py-6 border-b border-border/40">
        <div class="space-y-1">
            <Label class="text-base font-bold">{i18n.t('settings.general_section.accent_color')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.general_section.accent_color_desc')}</p>
        </div>

        <div class="flex flex-wrap items-center gap-3">
            <div class="relative flex items-center gap-3 bg-muted/20 p-2 rounded-2xl border border-border/50">
                <Input
                        type="color"
                        value={themeManager.accentColor || '#ffffff'}
                        onchange={handleCustomColor}
                        class="w-10 h-10 p-0 rounded-lg border-none cursor-pointer bg-transparent shrink-0"
                />
                <span class="text-xs font-mono font-bold pr-2 uppercase opacity-70">{themeManager.accentColor}</span>
            </div>

            <div class="h-8 w-px bg-border/40 mx-2 hidden sm:block"></div>

            <div class="flex flex-wrap gap-2">
                {#each colorPresets as preset}
                    <button
                            type="button"
                            onclick={() => setPresetColor(preset.value)}
                            class="size-10 rounded-full border-2 border-background shadow-sm transition-transform active:scale-90 flex items-center justify-center"
                            style="background-color: {preset.value}"
                            title={preset.name}
                    >
                        {#if themeManager.accentColor?.toLowerCase() === preset.value.toLowerCase()}
                            <Check class="size-5 text-white drop-shadow-md" />
                        {/if}
                    </button>
                {/each}
            </div>
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.ui_section.title_language')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.ui_section.title_language_desc')}</p>
        </div>
        <ResponsiveSelect
                bind:value={config.titleLanguage}
                items={titleLanguageItems}
                class="rounded-sm h-11 w-full sm:max-w-md capitalize"
                onValueChange={onSave}
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.ui_section.default_home')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.ui_section.default_home_desc')}</p>
        </div>
        <ResponsiveSelect
                bind:value={config.defaultHomeSection}
                items={homeSectionItems}
                class="rounded-sm h-11 w-full sm:max-w-md capitalize"
                onValueChange={onSave}
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="disableCardTrailers">{i18n.t('settings.ui_section.disable_card_trailers')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.ui_section.disable_card_trailers_desc')}</p>
        </div>
        <Switch id="disableCardTrailers" bind:checked={config.disableCardTrailers} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>