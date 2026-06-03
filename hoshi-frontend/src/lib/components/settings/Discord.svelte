<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { MessageSquare } from "lucide-svelte";
    import type { DiscordConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: DiscordConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section class="space-y-2">
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight flex items-center gap-2">
            <MessageSquare class="size-6 text-[#5865F2]" />
            {i18n.t('settings.discord')}
        </h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.discord_section.discord_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold cursor-pointer" for="discordEnabled">
                {i18n.t('settings.discord_section.enabled')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.discord_section.enabled_desc')}</p>
        </div>
        <Switch
                id="discordEnabled"
                bind:checked={config.enabled}
                onCheckedChange={onSave}
                class="shrink-0 scale-125"
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40 transition-opacity {!config.enabled ? 'opacity-50' : ''}">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold {config.enabled ? 'cursor-pointer' : 'cursor-not-allowed'}" for="showTitle">
                {i18n.t('settings.discord_section.show_title')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.discord_section.show_title_desc')}</p>
        </div>
        <Switch
                id="showTitle"
                bind:checked={config.showTitle}
                disabled={!config.enabled}
                onCheckedChange={onSave}
                class="shrink-0 scale-125"
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 transition-opacity {!config.enabled ? 'opacity-50' : ''}">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold {config.enabled ? 'cursor-pointer' : 'cursor-not-allowed'}" for="hideNsfw">
                {i18n.t('settings.discord_section.hide_nsfw')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.discord_section.hide_nsfw_desc')}</p>
        </div>
        <Switch
                id="hideNsfw"
                bind:checked={config.hideNsfw}
                disabled={!config.enabled}
                onCheckedChange={onSave}
                class="shrink-0 scale-125"
        />
    </div>
</section>