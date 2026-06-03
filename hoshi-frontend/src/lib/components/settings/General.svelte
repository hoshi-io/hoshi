<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";

    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type {ContentConfig, GeneralConfig} from "@/api/config/types";
    import LanguageSelector from "@/components/LanguageSelector.svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

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
        { value: "kitsu", label: "Kitsu" }
    ];
</script>

<section class="space-y-2">
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.general')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.general_section.general_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold" for="language">{i18n.t('settings.general_section.language')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.general_section.language_desc')}</p>
        </div>
        <div class="w-full sm:max-w-[200px]">
            <LanguageSelector
                    class="w-full h-11 rounded-sm bg-muted/20 border-transparent hover:bg-muted/30 transition-colors"
                    onLanguageChange={(code) => {
                    config.language = code;
                    onSave();
                }}
            />
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold cursor-pointer" for="showAdultContent">{i18n.t('settings.general_section.show_nsfw')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.general_section.show_nsfw_desc')}</p>
        </div>
        <Switch id="showAdultContent" bind:checked={config.showAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 transition-opacity { !config.showAdultContent ? 'opacity-50' : '' }">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold { config.showAdultContent ? 'cursor-pointer' : 'cursor-not-allowed' }" for="blurAdultContent">
                {i18n.t('settings.general_section.blur_nsfw')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.general_section.blur_nsfw_desc')}</p>
        </div>
        <Switch
                id="blurAdultContent"
                bind:checked={config.blurAdultContent}
                disabled={!config.showAdultContent}
                onCheckedChange={onSave}
                class="shrink-0"
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.content_section.metadata_provider')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.content_section.metadata_provider_desc')}</p>
        </div>
        <ResponsiveSelect
                bind:value={contentConfig.preferredMetadataProvider}
                items={metadataProviders}
                class="rounded-sm h-11 w-full sm:max-w-md capitalize"
                onValueChange={onSave}
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoUpdateProgress">{i18n.t('settings.content_section.auto_update_progress')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.content_section.auto_update_progress_desc')}</p>
        </div>
        <Switch id="autoUpdateProgress" bind:checked={contentConfig.autoUpdateProgress} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>