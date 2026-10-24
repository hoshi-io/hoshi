<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Switch } from "$lib/components/ui/switch";
    import type { DiscordConfig } from "@/api/config/types";
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import SettingRow from "@/components/settings/parts/SettingRow.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: DiscordConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<SettingsSection
        title={i18n.t('settings.discord')}
        description={i18n.t('settings.discord_section.discord_desc')}
>
    <SettingRow
            inline
            for="discordEnabled"
            label={i18n.t('settings.discord_section.enabled')}
            description={i18n.t('settings.discord_section.enabled_desc')}
    >
        <Switch id="discordEnabled" bind:checked={config.enabled} onCheckedChange={onSave} />
    </SettingRow>

    <SettingRow
            inline
            for="showTitle"
            disabled={!config.enabled}
            label={i18n.t('settings.discord_section.show_title')}
            description={i18n.t('settings.discord_section.show_title_desc')}
    >
        <Switch id="showTitle" bind:checked={config.showTitle} disabled={!config.enabled} onCheckedChange={onSave} />
    </SettingRow>

    <SettingRow
            inline
            for="hideNsfw"
            disabled={!config.enabled}
            label={i18n.t('settings.discord_section.hide_nsfw')}
            description={i18n.t('settings.discord_section.hide_nsfw_desc')}
    >
        <Switch id="hideNsfw" bind:checked={config.hideNsfw} disabled={!config.enabled} onCheckedChange={onSave} />
    </SettingRow>
</SettingsSection>