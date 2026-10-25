<script lang="ts">
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { ContentConfig, GeneralConfig } from '@/api/config/types.ts';
    import LanguageSelector from '@/components/LanguageSelector.svelte';
    import TvSettingRow from '@/tv/TvSettingRow.svelte';
    import TvSwitch from '@/tv/TvSwitch.svelte';
    import TvSegmented from '@/tv/TvSegmented.svelte';

    let {
        config = $bindable(),
        contentConfig = $bindable(),
        onSave
    }: {
        config: GeneralConfig;
        contentConfig: ContentConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    const metadataProviders = [
        { value: 'anilist', label: 'AniList' },
        { value: 'myanimelist', label: 'MyAnimeList' },
        { value: 'kitsu', label: 'Kitsu' },
        { value: 'simkl', label: 'Simkl' }
    ];

    const scoreFormats = [
        { value: 'point10', label: '0 – 10' },
        { value: 'point100', label: '0 – 100' },
        { value: 'point10Decimal', label: '0.0 – 10.0' },
        { value: 'point5Stars', label: '★★★★★' }
    ];

    // LanguageSelector is a desktop select and is not TV-adapted yet. This wrapper only makes it
    // reachable with a remote: OK on the wrapper focuses the trigger and opens it.
    function openLanguage(e: KeyboardEvent) {
        if (e.key !== 'Enter') return;
        e.preventDefault();
        const trigger = (e.currentTarget as HTMLElement).querySelector<HTMLElement>(
            'button, [role="combobox"]'
        );
        if (!trigger) return;
        trigger.focus();
        // Some selects open on pointerdown rather than click.
        trigger.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' }));
        trigger.click();
    }
</script>

<div>
    <header class="space-y-1 pb-2">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.general')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.general_section.general_desc')}</p>
    </header>

    <TvSettingRow
            label={i18n.t('settings.general_section.language')}
            description={i18n.t('settings.general_section.language_desc')}
    >
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
                data-sn
                tabindex="0"
                role="button"
                onkeydown={openLanguage}
                class="w-56 scroll-my-24 rounded-md outline-none focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background"
        >
            <LanguageSelector
                    class="h-11 w-full rounded-md border-transparent bg-muted/40"
                    onLanguageChange={(code) => {
                        config.language = code;
                        onSave();
                    }}
            />
        </div>
    </TvSettingRow>

    <TvSettingRow
            label={i18n.t('settings.general_section.show_nsfw')}
            description={i18n.t('settings.general_section.show_nsfw_desc')}
    >
        <TvSwitch
                label={i18n.t('settings.general_section.show_nsfw')}
                bind:checked={config.showAdultContent}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    <TvSettingRow
            disabled={!config.showAdultContent}
            label={i18n.t('settings.general_section.blur_nsfw')}
            description={i18n.t('settings.general_section.blur_nsfw_desc')}
    >
        <TvSwitch
                label={i18n.t('settings.general_section.blur_nsfw')}
                bind:checked={config.blurAdultContent}
                disabled={!config.showAdultContent}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    <TvSettingRow
            stacked
            label={i18n.t('settings.content_section.metadata_provider')}
            description={i18n.t('settings.content_section.metadata_provider_desc')}
    >
        <TvSegmented
                label={i18n.t('settings.content_section.metadata_provider')}
                bind:value={contentConfig.preferredMetadataProvider}
                options={metadataProviders}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    <TvSettingRow
            stacked
            label={i18n.t('settings.content_section.score_format')}
            description={i18n.t('settings.content_section.score_format_desc')}
    >
        <TvSegmented
                label={i18n.t('settings.content_section.score_format')}
                bind:value={contentConfig.scoreFormat}
                options={scoreFormats}
                onchange={() => onSave()}
        />
    </TvSettingRow>

    <TvSettingRow
            label={i18n.t('settings.content_section.auto_update_progress')}
            description={i18n.t('settings.content_section.auto_update_progress_desc')}
    >
        <TvSwitch
                label={i18n.t('settings.content_section.auto_update_progress')}
                bind:checked={contentConfig.autoUpdateProgress}
                onchange={() => onSave()}
        />
    </TvSettingRow>
</div>