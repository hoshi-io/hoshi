<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { PlayerConfig, SubtitleConfig } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Play, Captions } from "lucide-svelte";
    import { platform } from "@tauri-apps/plugin-os";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import * as Kbd from "$lib/components/ui/kbd";
    import {openUrl} from "@tauri-apps/plugin-opener";

    let {
        playerConfig = $bindable(),
        subConfig = $bindable(),
        onSave
    }: {
        playerConfig: PlayerConfig,
        subConfig: SubtitleConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const os = platform();

    const seekSteps = [
        { value: "5",  label: i18n.t('settings.player_section.seconds', { num: 5 }) },
        { value: "10", label: i18n.t('settings.player_section.seconds', { num: 10 }) },
        { value: "15", label: i18n.t('settings.player_section.seconds', { num: 15 }) },
        { value: "30", label: i18n.t('settings.player_section.seconds', { num: 30 }) }
    ];

    const hwdecOptions = [
        { value: "auto-safe", label: i18n.t("watch.settings.video_section.hwdec_auto_safe") },
        { value: "auto", label: i18n.t("watch.settings.video_section.hwdec_auto") },
        { value: "no", label: i18n.t("watch.settings.video_section.hwdec_software") }
    ];

    const scaleOptions = [
        { value: "bilinear", label: i18n.t("watch.settings.video_section.scale_fast") },
        { value: "spline36", label: i18n.t("watch.settings.video_section.scale_balanced") },
        { value: "ewa_lanczossharp", label: i18n.t("watch.settings.video_section.scale_sharp") }
    ];

    const fontOptions = [
        { value: "sans-serif", label: "Sans-serif" },
        { value: "serif", label: "Serif" },
        { value: "monospace", label: "Monospace" },
        { value: "Arial", label: "Arial" },
        { value: "Roboto", label: "Roboto" },
        { value: "Open Sans", label: "Open Sans" },
        { value: "Trebuchet MS", label: "Trebuchet MS" },
        { value: "Georgia", label: "Georgia" },
        { value: "Comic Sans MS", label: "Comic Sans MS" }
    ];

    const fontSizeOptions = [
        { value: "36", label: i18n.t("watch.settings.subtitle_style_section.size_small") },
        { value: "55", label: i18n.t("watch.settings.subtitle_style_section.size_medium") },
        { value: "70", label: i18n.t("watch.settings.subtitle_style_section.size_large") },
        { value: "85", label: i18n.t("watch.settings.subtitle_style_section.size_extra_large") }
    ];

    const alignmentOptions = [
        { value: "auto", label: i18n.t("watch.settings.subtitle_style_section.align_auto") },
        { value: "left", label: i18n.t("watch.settings.subtitle_style_section.align_left") },
        { value: "center", label: i18n.t("watch.settings.subtitle_style_section.align_center") },
        { value: "right", label: i18n.t("watch.settings.subtitle_style_section.align_right") }
    ];

    const keyboardShortcuts = [
        { label: i18n.t('watch.player.play') + "/" + i18n.t('watch.player.pause'), keys: ["Space", "K"] },
        { label: i18n.t('watch.player.seek'), keys: ["→", "L"] },
        { label: i18n.t('watch.player.seek'), keys: ["←", "J"] },
        { label: i18n.t('watch.player.volume'), keys: ["↑", "↓"] },
        { label: i18n.t('watch.player.mute'), keys: ["M"] },
        { label: i18n.t('watch.player.fullscreen_enter'), keys: ["F"] },
        { label: i18n.t('watch.player.next_episode'), keys: ["N"] },
        { label: i18n.t('watch.player.previous_episode'), keys: ["P"] },
        { label: i18n.t('watch.player.close_settings'), keys: ["Esc"] },
    ];

    const anime4kTierOptions = [
        { value: "off", label: i18n.t('settings.player_section.anime4k_off') },
        { value: "fast", label: i18n.t('settings.player_section.anime4k_fast') },
        { value: "hq", label: i18n.t('settings.player_section.anime4k_hq') },
    ];

    const anime4kModeOptions = [
        { value: "a", label: "A" },
        { value: "b", label: "B" },
        { value: "c", label: "C" },
    ];

    function handleAnime4kTierChange(val: string) {
        playerConfig.anime4kTier = val;
        onSave();
    }

    function handleAnime4kModeChange(val: string) {
        playerConfig.anime4kMode = val;
        onSave();
    }

    function handleSeekStepChange(val: string) {
        playerConfig.seekStep = parseInt(val);
        onSave();
    }

    function handleHwdecChange(val: string) {
        playerConfig.hwdec = val;
        onSave();
    }

    function handleScaleChange(val: string) {
        playerConfig.scaleAlgorithm = val;
        onSave();
    }

    function handleFontChange(val: string) {
        subConfig.font = val;
        onSave();
    }

    function handleFontSizeChange(val: string) {
        subConfig.fontSize = parseInt(val);
        onSave();
    }

    function handleAlignmentChange(val: string) {
        subConfig.justify = val as "auto" | "left" | "center" | "right";
        onSave();
    }
</script>

<div class="space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.player')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.player_section.player_desc')}</p>
    </div>

    <Tabs.Root value="player_general" class="w-full">
        <Tabs.List class="grid w-full max-w-[400px] grid-cols-2 rounded-sm h-11 p-1 bg-muted/50">
            <Tabs.Trigger value="player_general" class="rounded-lg font-bold flex items-center gap-2">
                <Play class="size-4" /> {i18n.t("settings.general")}
            </Tabs.Trigger>
            <Tabs.Trigger value="player_subtitles" class="rounded-lg font-bold flex items-center gap-2">
                <Captions class="size-4" /> {i18n.t("watch.settings.subtitles")}
            </Tabs.Trigger>
        </Tabs.List>

        <!-- GENERAL PLAYER CONFIG -->
        <Tabs.Content value="player_general" class="focus-visible:outline-none mt-0">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_sub_lang')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_sub_lang_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input bind:value={playerConfig.preferredSubLang} onchange={onSave} placeholder="en, es, ja" class="rounded-sm h-11" />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_dub_lang')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_dub_lang_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input bind:value={playerConfig.preferredDubLang} onchange={onSave} placeholder="ja, en" class="rounded-sm h-11" />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.seek_step')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.seek_step_desc')}</p>
                </div>
                <ResponsiveSelect value={playerConfig.seekStep.toString()} items={seekSteps} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleSeekStepChange} />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoNext">{i18n.t('settings.player_section.autoplay')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.autoplay_desc')}</p>
                </div>
                <Switch id="autoNext" bind:checked={playerConfig.autoplayNextEpisode} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="resumeFromLastPos">{i18n.t('settings.player_section.resume_playback')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.resume_playback_desc')}</p>
                </div>
                <Switch id="resumeFromLastPos" bind:checked={playerConfig.resumeFromLastPos} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipIntro">{i18n.t('settings.player_section.auto_skip_intro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_intro_desc')}</p>
                </div>
                <Switch id="autoSkipIntro" bind:checked={playerConfig.autoSkipIntro} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSkipOutro">{i18n.t('settings.player_section.auto_skip_outro')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_outro_desc')}</p>
                </div>
                <Switch id="autoSkipOutro" bind:checked={playerConfig.autoSkipOutro} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.hwdec')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.hwdec_desc')}</p>
                </div>
                <ResponsiveSelect value={playerConfig.hwdec} items={hwdecOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleHwdecChange} />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.scale_algorithm')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.scale_algorithm_desc')}</p>
                </div>
                <ResponsiveSelect value={playerConfig.scaleAlgorithm} items={scaleOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleScaleChange} />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="interpolation">{i18n.t('settings.player_section.interpolation')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.interpolation_desc')}</p>
                </div>
                <Switch id="interpolation" bind:checked={playerConfig.interpolation} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="deband">{i18n.t('settings.player_section.deband')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.deband_desc')}</p>
                </div>
                <Switch id="deband" bind:checked={playerConfig.deband} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{i18n.t('settings.player_section.anime4k')}</Label>
                    <p class="text-sm text-muted-foreground">
                        {i18n.t('settings.player_section.anime4k_desc')}
                        <button
                                type="button"
                                class="underline hover:text-foreground transition-colors"
                                onclick={() => openUrl('https://github.com/bloc97/Anime4K')}
                        >
                            {i18n.t('settings.player_section.anime4k_learn_more')}
                        </button>
                    </p>
                </div>
                <ResponsiveSelect value={playerConfig.anime4kTier} items={anime4kTierOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleAnime4kTierChange} />
            </div>

            {#if playerConfig.anime4kTier !== "off"}
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{i18n.t('settings.player_section.anime4k_mode')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.anime4k_mode_desc')}</p>
                    </div>
                    <ResponsiveSelect value={playerConfig.anime4kMode} items={anime4kModeOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleAnime4kModeChange} />
                </div>
            {/if}

            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 border-t border-border/40 mt-2">
                {#each keyboardShortcuts as shortcut}
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
        </Tabs.Content>

        <!-- SUBTITLE CONFIG -->
        <Tabs.Content value="player_subtitles" class="focus-visible:outline-none mt-0">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-4 py-4">
                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_font')}</Label>
                    <p class="text-xs text-muted-foreground">{i18n.t('settings.player_section.sub_font_desc')}</p>
                    <ResponsiveSelect value={subConfig.font} items={fontOptions} class="rounded-sm h-10 w-full" onValueChange={handleFontChange} />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_font_size')}</Label>
                    <p class="text-xs text-muted-foreground">{i18n.t('settings.player_section.sub_font_size_desc')}</p>
                    <ResponsiveSelect value={subConfig.fontSize.toString()} items={fontSizeOptions} class="rounded-sm h-10 w-full" onValueChange={handleFontSizeChange} />
                </div>

                <div class="flex flex-col gap-1">
                    <div class="flex items-center justify-between">
                        <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_color')}</Label>
                        <input type="color" bind:value={subConfig.color} onchange={onSave} class="w-10 h-10 rounded bg-transparent cursor-pointer" />
                    </div>
                    <p class="text-xs text-muted-foreground">{i18n.t('settings.player_section.sub_color_desc')}</p>
                </div>

                <div class="flex items-center justify-between py-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_border_color')}</Label>
                    <input type="color" bind:value={subConfig.borderColor} onchange={onSave} class="w-10 h-10 rounded bg-transparent cursor-pointer" />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_border_size')}</Label>
                    <input type="range" min="0" max="10" step="0.5" bind:value={subConfig.borderSize} oninput={onSave} class="w-full accent-primary" />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_scale')}</Label>
                    <input type="range" min="0.5" max="2" step="0.05" bind:value={subConfig.scale} oninput={onSave} class="w-full accent-primary" />
                </div>

                <div class="flex items-center justify-between py-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_shadow_color')}</Label>
                    <input type="color" bind:value={subConfig.shadowColor} onchange={onSave} class="w-10 h-10 rounded bg-transparent cursor-pointer" />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_shadow_offset')}</Label>
                    <input type="range" min="0" max="10" step="0.5" bind:value={subConfig.shadowOffset} oninput={onSave} class="w-full accent-primary" />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_position')}</Label>
                    <p class="text-xs text-muted-foreground">{i18n.t('settings.player_section.sub_position_desc')}</p>
                    <input type="range" min="0" max="100" step="1" bind:value={subConfig.position} oninput={onSave} class="w-full accent-primary" />
                </div>

                <div class="flex flex-col gap-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_delay')}</Label>
                    <Input type="number" bind:value={subConfig.delay} onchange={onSave} class="rounded-sm h-10" />
                </div>

                <div class="flex flex-col gap-2 md:col-span-2">
                    <Label class="text-sm font-bold">{i18n.t('settings.player_section.sub_justify')}</Label>
                    <ResponsiveSelect value={subConfig.justify} items={alignmentOptions} class="rounded-sm h-10 w-full" onValueChange={handleAlignmentChange} />
                </div>
            </div>

            <div class="mt-4 space-y-2 border-t border-border/40 pt-4">
                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="forceStyle">{i18n.t('settings.player_section.sub_force_style')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.sub_force_style_desc')}</p>
                    </div>
                    <Switch id="forceStyle" bind:checked={subConfig.forceStyle} onCheckedChange={onSave} class="shrink-0 scale-125" />
                </div>

                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="sdhFilter">{i18n.t('settings.player_section.sub_sdh_filter')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.sub_sdh_filter_desc')}</p>
                    </div>
                    <Switch id="sdhFilter" bind:checked={subConfig.sdhFilter} onCheckedChange={onSave} class="shrink-0 scale-125" />
                </div>

                <div class="flex items-center justify-between py-4 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="sdhFilterHarder">{i18n.t('settings.player_section.sub_sdh_filter_harder')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.sub_sdh_filter_harder_desc')}</p>
                    </div>
                    <Switch id="sdhFilterHarder" bind:checked={subConfig.sdhFilterHarder} onCheckedChange={onSave} class="shrink-0 scale-125" />
                </div>
            </div>
        </Tabs.Content>

    </Tabs.Root>
</div>