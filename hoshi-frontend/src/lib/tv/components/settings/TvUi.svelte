<script lang="ts">
    import { Check } from 'lucide-svelte';
    import type { UiConfig } from '@/api/config/types.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { themeManager } from '@/stores/theme.svelte.js';
    import TvSettingRow from '@/tv/components/TvSettingRow.svelte';
    import TvSegmented from '@/tv/components/TvSegmented.svelte';
    import TvColorSwatches from '@/tv/components/TvColorSwatches.svelte';

    let {
        config = $bindable(),
        onSave
    }: {
        config: UiConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white' }
    ];

    const accentPresets = [
        '#a855f7', '#3b82f6', '#06b6d4', '#22c55e', '#f59e0b', '#f97316', '#f43f5e', '#ec4899'
    ];

    const titleLanguageItems = $derived([
        { value: 'romaji', label: i18n.t('settings.ui_section.title_language_romaji') },
        { value: 'english', label: i18n.t('settings.ui_section.title_language_english') },
        { value: 'native', label: i18n.t('settings.ui_section.title_language_native') },
        { value: 'chinese', label: i18n.t('settings.ui_section.title_language_chinese') }
    ]);
</script>

<div>
    <header class="space-y-1 pb-2">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.interface')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.ui_section.interface_desc')}</p>
    </header>

    <TvSettingRow
            stacked
            label={i18n.t('settings.general_section.theme')}
            description={i18n.t('settings.general_section.theme_desc')}
    >
        <div role="radiogroup" aria-label={i18n.t('settings.general_section.theme')} class="grid w-full grid-cols-3 gap-4 p-1">
            {#each themes as theme (theme.id)}
                {@const selected = themeManager.theme === theme.id}
                <button
                        type="button"
                        data-sn
                        role="radio"
                        aria-checked={selected}
                        onclick={() => themeManager.setTheme(theme.id)}
                        class="relative flex h-16 scroll-my-24 items-center justify-center rounded-lg border-2 text-lg font-bold outline-none
                               transition-transform duration-150
                               focus:scale-105 focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background
                               {theme.classes}
                               {selected ? 'border-primary' : 'border-border/60 opacity-80'}"
                >
                    {theme.label}
                    {#if selected}
                        <span class="absolute right-2 top-2 rounded-full bg-primary p-1">
                            <Check class="size-3.5 text-primary-foreground" strokeWidth={3} />
                        </span>
                    {/if}
                </button>
            {/each}
        </div>
    </TvSettingRow>

    <TvSettingRow
            stacked
            label={i18n.t('settings.general_section.accent_color')}
            description={i18n.t('settings.general_section.accent_color_desc')}
    >
        <TvColorSwatches
                label={i18n.t('settings.general_section.accent_color')}
                value={themeManager.accentColor || ''}
                presets={accentPresets}
                onchange={(c) => themeManager.setAccentColor(c)}
        />
    </TvSettingRow>

    <TvSettingRow
            stacked
            label={i18n.t('settings.ui_section.title_language')}
            description={i18n.t('settings.ui_section.title_language_desc')}
    >
        <TvSegmented
                label={i18n.t('settings.ui_section.title_language')}
                bind:value={config.titleLanguage}
                options={titleLanguageItems}
                onchange={() => onSave()}
        />
    </TvSettingRow>
</div>