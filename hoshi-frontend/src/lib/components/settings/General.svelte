<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";

    import { Switch } from "$lib/components/ui/switch";
    import type { ContentConfig, GeneralConfig } from "@/api/config/types";
    import LanguageSelector from "@/components/LanguageSelector.svelte";
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import SettingRow from "@/components/settings/parts/SettingRow.svelte";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";

    let {
        config = $bindable(),
        contentConfig = $bindable(),
        onSave
    }: {
        config: GeneralConfig,
        contentConfig: ContentConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const metadataProviders = [
        { value: "anilist", label: "AniList" },
        { value: "myanimelist", label: "MyAnimeList" },
        { value: "kitsu", label: "Kitsu" },
        { value: "simkl", label: "Simkl" }
    ];

    const scoreFormats = [
        { value: "point10", label: "0 – 10" },
        { value: "point100", label: "0 – 100" },
        { value: "point10Decimal", label: "0.0 – 10.0" },
        { value: "point5Stars", label: "★★★★★" }
    ];
</script>

<SettingsSection
        title={i18n.t('settings.general')}
        description={i18n.t('settings.general_section.general_desc')}
>
    <SettingRow
            label={i18n.t('settings.general_section.language')}
            description={i18n.t('settings.general_section.language_desc')}
    >
        <div class="w-full sm:w-52">
            <LanguageSelector class="w-full h-10 rounded-md bg-muted/40 border-transparent hover:bg-muted/60 transition-colors"/>
        </div>
    </SettingRow>

    <SettingRow
            inline
            for="showAdultContent"
            label={i18n.t('settings.general_section.show_nsfw')}
            description={i18n.t('settings.general_section.show_nsfw_desc')}
    >
        <Switch id="showAdultContent" bind:checked={config.showAdultContent} onCheckedChange={onSave} />
    </SettingRow>

    <SettingRow
            inline
            for="blurAdultContent"
            disabled={!config.showAdultContent}
            label={i18n.t('settings.general_section.blur_nsfw')}
            description={i18n.t('settings.general_section.blur_nsfw_desc')}
    >
        <Switch
                id="blurAdultContent"
                bind:checked={config.blurAdultContent}
                disabled={!config.showAdultContent}
                onCheckedChange={onSave}
        />
    </SettingRow>

    <SettingRow
            label={i18n.t('settings.content_section.metadata_provider')}
            description={i18n.t('settings.content_section.metadata_provider_desc')}
    >
        <SegmentedControl
                label={i18n.t('settings.content_section.metadata_provider')}
                bind:value={contentConfig.preferredMetadataProvider}
                options={metadataProviders}
                onValueChange={() => onSave()}
        />
    </SettingRow>

    <SettingRow
            label={i18n.t('settings.content_section.score_format')}
            description={i18n.t('settings.content_section.score_format_desc')}
    >
        <SegmentedControl
                label={i18n.t('settings.content_section.score_format')}
                bind:value={contentConfig.scoreFormat}
                options={scoreFormats}
                onValueChange={() => onSave()}
        />
    </SettingRow>

    <SettingRow
            inline
            for="autoUpdateProgress"
            label={i18n.t('settings.content_section.auto_update_progress')}
            description={i18n.t('settings.content_section.auto_update_progress_desc')}
    >
        <Switch id="autoUpdateProgress" bind:checked={contentConfig.autoUpdateProgress} onCheckedChange={onSave} />
    </SettingRow>
</SettingsSection>