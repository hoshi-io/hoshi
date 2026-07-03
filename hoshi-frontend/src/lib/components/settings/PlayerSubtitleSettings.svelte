<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Switch } from "$lib/components/ui/switch";
    import { RotateCcw, MonitorPlay } from "lucide-svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import { AspectRatio } from "$lib/components/ui/aspect-ratio";

    import {
        SUBTITLE_FONTS,
        SubtitleSettings,
        type SubtitleSettingsData
    } from "@/components/player/subtitles/SubtitleSettings.svelte";
    import {i18n} from "@/stores/i18n.svelte";

    const subtitleSettings = new SubtitleSettings();

    const outlineStyles: { value: SubtitleSettingsData['outlineStyle']; label: string }[] = [
        { value: 'none', label: i18n.t('player.none') },
        { value: 'outline', label: i18n.t('player.outline') },
        { value: 'drop-shadow', label: i18n.t('player.drop_shadow') },
        { value: 'raised', label: i18n.t('player.raised') },
        { value: 'depressed', label: i18n.t('player.depressed') }
    ];

    const textAligns: { value: SubtitleSettingsData['textAlign']; label: string }[] = [
        { value: 'left', label: i18n.t('player.left') },
        { value: 'center', label: i18n.t('player.center') },
        { value: 'right', label: i18n.t('player.right') }
    ];
</script>

<div class="focus-visible:outline-none mt-4 grid grid-cols-1 lg:grid-cols-[1fr_400px] xl:grid-cols-[1fr_500px] gap-8 items-start">

    <div class="flex flex-col border-r border-border/40 lg:pr-8">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.font_family")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.font_family_desc")}</p>
            </div>
            <ResponsiveSelect
                    value={subtitleSettings.fontFamily}
                    items={SUBTITLE_FONTS}
                    class="rounded-sm h-11 w-full sm:max-w-[200px]"
                    onValueChange={(val) => { subtitleSettings.fontFamily = val; subtitleSettings.save(); }}
            />
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.font_size")} (em)</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.font_size_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[200px]">
                <Input type="number" step="0.1" min="0.5" max="5" bind:value={subtitleSettings.fontSize} onchange={() => subtitleSettings.save()} class="rounded-sm h-11" />
            </div>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.text_color")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.text_color_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[200px] flex gap-2">
                <Input type="color" bind:value={subtitleSettings.color} onchange={() => subtitleSettings.save()} class="w-12 h-11 p-1 rounded-sm cursor-pointer shrink-0" />
                <Input type="text" bind:value={subtitleSettings.color} onchange={() => subtitleSettings.save()} class="rounded-sm h-11 flex-1" />
            </div>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.bold")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.bold_desc")}</p>
            </div>
            <Switch
                    checked={subtitleSettings.fontWeight === 'bold'}
                    onCheckedChange={(checked) => { subtitleSettings.fontWeight = checked ? 'bold' : 'normal'; subtitleSettings.save(); }}
                    class="shrink-0 scale-125"
            />
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.italic")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.italic_desc")}</p>
            </div>
            <Switch bind:checked={subtitleSettings.italic} onCheckedChange={() => subtitleSettings.save()} class="shrink-0 scale-125" />
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.outline_shadow")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.outline_shadow_desc")}</p>
            </div>
            <ResponsiveSelect
                    value={subtitleSettings.outlineStyle}
                    items={outlineStyles}
                    class="rounded-sm h-11 w-full sm:max-w-[200px]"
                    onValueChange={(val) => { subtitleSettings.outlineStyle = val as SubtitleSettingsData['outlineStyle']; subtitleSettings.save(); }}
            />
        </div>

        {#if subtitleSettings.outlineStyle !== 'none'}
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t("settings.player_section.outline_color")}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.outline_color_desc")}</p>
                </div>
                <div class="w-full sm:max-w-[200px] flex gap-2">
                    <Input type="color" bind:value={subtitleSettings.outlineColor} onchange={() => subtitleSettings.save()} class="w-12 h-11 p-1 rounded-sm cursor-pointer shrink-0" />
                    <Input type="text" bind:value={subtitleSettings.outlineColor} onchange={() => subtitleSettings.save()} class="rounded-sm h-11 flex-1" />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t("settings.player_section.outline_width")} (px)</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.outline_width_desc")}</p>
                </div>
                <div class="w-full sm:max-w-[200px]">
                    <Input type="number" min="0" max="10" bind:value={subtitleSettings.outlineWidth} onchange={() => subtitleSettings.save()} class="rounded-sm h-11" />
                </div>
            </div>
        {/if}

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.background_color")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.background_color_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[200px]">
                <Input type="text" bind:value={subtitleSettings.bgColor} onchange={() => subtitleSettings.save()} class="rounded-sm h-11" />
            </div>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.vertical_alignment")} (%)</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.vertical_alignment_desc")}</p>
            </div>
            <div class="w-full sm:max-w-[200px]">
                <Input type="number" min="0" max="100" bind:value={subtitleSettings.positionY} onchange={() => subtitleSettings.save()} class="rounded-sm h-11" />
            </div>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
            <div class="space-y-1 pr-4 flex-1">
                <Label class="text-base font-bold">{i18n.t("settings.player_section.text_alignment")}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t("settings.player_section.text_alignment_desc")}</p>
            </div>
            <ResponsiveSelect
                    value={subtitleSettings.textAlign}
                    items={textAligns}
                    class="rounded-sm h-11 w-full sm:max-w-[200px]"
                    onValueChange={(val) => { subtitleSettings.textAlign = val as SubtitleSettingsData['textAlign']; subtitleSettings.save(); }}
            />
        </div>

        <div class="flex justify-start pt-6 pb-6 lg:pb-0">
            <button
                    type="button"
                    class="inline-flex items-center gap-2 rounded-lg font-bold px-4 py-2 bg-muted text-muted-foreground hover:bg-muted/70 transition-all text-sm"
                    onclick={() => subtitleSettings.reset()}
            >
                <RotateCcw class="size-4" />{i18n.t("settings.player_section.reset_settings")}
            </button>
        </div>
    </div>

    <div class="sticky top-6 w-full order-first lg:order-last mb-6 lg:mb-0">
        <div class="bg-black/90 rounded-lg overflow-hidden border border-border/50 shadow-xl ring-1 ring-border/20">
            <AspectRatio ratio={16 / 9} class="relative w-full" style="container-type: inline-size; font-size: 0.83cqw;">
                <div class="absolute inset-0 flex items-center justify-center opacity-20 pointer-events-none">
                    <MonitorPlay class="size-16 text-muted-foreground" style="font-size: 1rem;" />
                </div>

                <div class="absolute inset-0 bg-gradient-to-t from-black/40 to-transparent pointer-events-none"></div>

                <div
                        class="absolute left-1/2 -translate-x-1/2 w-full pointer-events-none"
                        style={subtitleSettings.wrapperStyle}
                >
                    <span style={subtitleSettings.containerStyle}>
                        This is a live preview of your soft subtitles.
                        <br />
                        It updates instantly as you adjust the settings!
                    </span>
                </div>
            </AspectRatio>
        </div>
        <p class="text-xs text-muted-foreground mt-3 text-center">
            This settings will only work with soft subtitles.
            Text size might not be the same on the real player.
        </p>
    </div>

</div>