<script lang="ts">
    import { scale } from "svelte/transition";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { UiConfig } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { themeManager } from "@/stores/theme.svelte";
    import { Check } from "lucide-svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import SettingRow from "@/components/settings/parts/SettingRow.svelte";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";

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

    const accent = $derived(themeManager.accentColor || '');
    const isPreset = $derived(colorPresets.some((p) => p.value.toLowerCase() === accent.toLowerCase()));

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

    // Labels are translated and can be long, so title language stays a dropdown.
    const titleLanguageItems = [
        { value: "romaji", label: i18n.t('settings.ui_section.title_language_romaji') },
        { value: "english", label: i18n.t('settings.ui_section.title_language_english') },
        { value: "native", label: i18n.t('settings.ui_section.title_language_native') },
        { value: "chinese", label: i18n.t('settings.ui_section.title_language_chinese') },
    ];

    // Three short options: a segmented control is quicker than opening a menu.
    const homeSectionItems = [
        { value: "anime", label: i18n.t('settings.ui_section.default_home_anime') },
        { value: "manga", label: i18n.t('settings.ui_section.default_home_manga') },
        { value: "novel", label: i18n.t('settings.ui_section.default_home_novel') }
    ];
</script>

<SettingsSection
        title={i18n.t('settings.interface')}
        description={i18n.t('settings.ui_section.interface_desc')}
>
    <SettingRow
            stacked
            label={i18n.t('settings.general_section.theme')}
            description={i18n.t('settings.general_section.theme_desc')}
    >
        <div role="radiogroup" aria-label={i18n.t('settings.general_section.theme')} class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
            {#each themes as theme (theme.id)}
                {@const selected = themeManager.theme === theme.id}
                <button
                        type="button"
                        role="radio"
                        aria-checked={selected}
                        onclick={() => changeTheme(theme.id)}
                        class="relative flex items-center justify-center h-14 rounded-sm border-2 font-bold overflow-hidden transition-all duration-200
                               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 {theme.classes}
                               {selected
                                   ? 'ring-2 ring-primary ring-offset-2 ring-offset-background border-transparent'
                                   : 'opacity-80 hover:opacity-100 hover:-translate-y-px active:scale-[0.98] border-transparent'}"
                >
                    {theme.label}
                    {#if selected}
                        <div in:scale={{ start: 0.4, duration: 200 }} class="absolute top-1.5 right-1.5 bg-primary rounded-full p-0.5">
                            <Check class="size-3 text-primary-foreground" />
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    </SettingRow>

    <SettingRow
            stacked
            label={i18n.t('settings.general_section.accent_color')}
            description={i18n.t('settings.general_section.accent_color_desc')}
    >
        <div class="flex flex-wrap items-center gap-x-3 gap-y-3">
            {#each colorPresets as preset (preset.value)}
                {@const selected = accent.toLowerCase() === preset.value.toLowerCase()}
                <button
                        type="button"
                        onclick={() => setPresetColor(preset.value)}
                        aria-label={preset.name}
                        aria-pressed={selected}
                        title={preset.name}
                        class="grid place-items-center size-9 rounded-full shadow-sm transition-all duration-200 active:scale-90 hover:scale-110
                               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-background
                               {selected ? 'ring-2 ring-offset-2 ring-offset-background scale-105' : ''}"
                        style="background-color: {preset.value}; --tw-ring-color: {preset.value}"
                >
                    {#if selected}
                        <Check class="size-4 text-white drop-shadow" strokeWidth={3} />
                    {/if}
                </button>
            {/each}

            <div class="h-6 w-px bg-border/60 mx-1 hidden sm:block"></div>

            <!-- Custom colour: rainbow swatch until a non-preset colour is active, then it shows that colour. -->
            <label
                    title="Custom"
                    class="relative grid place-items-center size-9 rounded-full shadow-sm cursor-pointer transition-all duration-200 hover:scale-110 active:scale-90
                           focus-within:ring-2 focus-within:ring-primary/50 focus-within:ring-offset-2 focus-within:ring-offset-background
                           {!isPreset && accent ? 'ring-2 ring-offset-2 ring-offset-background scale-105' : ''}"
                    style="background: {!isPreset && accent
                        ? accent
                        : 'conic-gradient(from 0deg, #f43f5e, #f59e0b, #22c55e, #06b6d4, #3b82f6, #a855f7, #f43f5e)'};
                        {!isPreset && accent ? `--tw-ring-color: ${accent}` : ''}"
            >
                <input
                        type="color"
                        value={accent || '#ffffff'}
                        onchange={handleCustomColor}
                        aria-label="Custom colour"
                        class="absolute inset-0 size-full opacity-0 cursor-pointer"
                />
                {#if !isPreset && accent}
                    <Check class="size-4 text-white drop-shadow pointer-events-none" strokeWidth={3} />
                {/if}
            </label>

            <span class="text-xs font-mono font-semibold uppercase tracking-wide text-muted-foreground tabular-nums">{accent}</span>
        </div>
    </SettingRow>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-5 border-b border-border/40">
        <div class="space-y-0.5 pr-4">
            <Label class="text-[15px] font-semibold leading-snug">{i18n.t('settings.ui_section.title_language')}</Label>
            <p class="text-sm text-muted-foreground leading-snug">{i18n.t('settings.ui_section.title_language_desc')}</p>
        </div>
        <ResponsiveSelect
                bind:value={config.titleLanguage}
                items={titleLanguageItems}
                class="rounded-sm h-11 w-full sm:max-w-65 capitalize"
                onValueChange={onSave}
        />
    </div>

    <SettingRow
            label={i18n.t('settings.ui_section.default_home')}
            description={i18n.t('settings.ui_section.default_home_desc')}
    >
        <SegmentedControl
                label={i18n.t('settings.ui_section.default_home')}
                bind:value={config.defaultHomeSection}
                options={homeSectionItems}
                onValueChange={() => onSave()}
                class="capitalize"
        />
    </SettingRow>

    <SettingRow
            inline
            for="disableCardTrailers"
            label={i18n.t('settings.ui_section.disable_card_trailers')}
            description={i18n.t('settings.ui_section.disable_card_trailers_desc')}
    >
        <Switch id="disableCardTrailers" bind:checked={config.disableCardTrailers} onCheckedChange={onSave} />
    </SettingRow>
</SettingsSection>