<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Input } from "$lib/components/ui/input";
    import type { PlayerConfig, SubtitleConfig } from "@/api/config/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { platform } from "@tauri-apps/plugin-os";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import SettingRow from "@/components/settings/parts/SettingRow.svelte";
    import SegmentedControl from "@/components/settings/parts/SegmentedControl.svelte";
    import SettingSlider from "@/components/settings/parts/SettingSlider.svelte";
    import ColorInput from "@/components/settings/parts/ColorInput.svelte";
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

    const thumbIntervalOptions = [
        { value: "5",  label: i18n.t('settings.player_section.seconds', { num: 5 }) },
        { value: "10", label: i18n.t('settings.player_section.seconds', { num: 10 }) },
        { value: "20", label: i18n.t('settings.player_section.seconds', { num: 20 }) },
        { value: "30", label: i18n.t('settings.player_section.seconds', { num: 30 }) }
    ];

    const thumbWidthOptions = [
        { value: "160", label: i18n.t('settings.player_section.thumb_size_small') },
        { value: "240", label: i18n.t('settings.player_section.thumb_size_medium') },
        { value: "320", label: i18n.t('settings.player_section.thumb_size_large') }
    ];

    function handleThumbIntervalChange(val: string) {
        playerConfig.thumbnailIntervalSeconds = parseInt(val);
        onSave();
    }

    function handleThumbWidthChange(val: string) {
        playerConfig.thumbnailWidth = parseInt(val);
        onSave();
    }

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

    // ------------------------------------------------------------------ layout

    let activeTab = $state("player_general");

    const tabs = [
        { id: "player_general", label: () => i18n.t("settings.general") },
        { id: "player_thumbnails", label: () => i18n.t("settings.player_section.thumbnails") },
        { id: "player_subtitles", label: () => i18n.t("watch.settings.subtitles") }
    ];

    const fieldWidth = "w-full sm:w-64";

    // subtitle grid cells
    // cell: label on top, control below (selects, segmented controls, inputs)
    // cellRow: label left, control right (sliders, colors, switches)
    const cell = "flex flex-col justify-between gap-3 rounded-xl border border-border/50 bg-muted/10 p-4";
    const cellRow = "flex items-center justify-between gap-4 rounded-xl border border-border/50 bg-muted/10 p-4";
    const cellLabel = "text-sm font-medium";
    const cellDesc = "mt-0.5 text-xs text-muted-foreground";

    const anime4kOff = $derived(playerConfig.anime4kTier === "off");
    const thumbsOff = $derived(!playerConfig.thumbnailsEnabled);
</script>

{#snippet groupHeader(title: string)}
    <h4 class="px-1 pb-1 pt-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {title}
    </h4>
{/snippet}

{#snippet anime4kLink()}
    <button
            type="button"
            class="underline hover:text-foreground transition-colors"
            onclick={() => openUrl('https://github.com/bloc97/Anime4K')}
    >
        {i18n.t('settings.player_section.anime4k_learn_more')}
    </button>
{/snippet}

<SettingsSection variant="bare" title={i18n.t('settings.player')} description={i18n.t('settings.player_section.player_desc')}>
    <Tabs.Root bind:value={activeTab} class="w-full">
        <div class="pt-1 pb-4">
            <Tabs.List class="flex w-full sm:inline-flex sm:w-auto h-auto gap-1 p-1 rounded-lg bg-muted/50">
                {#each tabs as tab (tab.id)}
                    <Tabs.Trigger
                            value={tab.id}
                            class="flex-1 sm:flex-none h-9 min-w-0 px-5 rounded-md text-sm font-semibold text-muted-foreground
                                   transition-all duration-200 hover:text-foreground
                                   data-[state=active]:bg-background data-[state=active]:text-foreground
                                   data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border/60
                                   focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                    >
                        <span class="truncate">{tab.label()}</span>
                    </Tabs.Trigger>
                {/each}
            </Tabs.List>
        </div>

        <!-- GENERAL -->
        <Tabs.Content value="player_general" class="focus-visible:outline-none mt-0">
            {#if activeTab === "player_general"}
                <div in:fly={{ y: 8, duration: 220, easing: cubicOut }} class="space-y-4">
                    <!-- Playback -->
                    <div>
                        <SettingRow label={i18n.t('settings.player_section.preferred_sub_lang')} description={i18n.t('settings.player_section.preferred_sub_lang_desc')}>
                            <Input bind:value={playerConfig.preferredSubLang} onchange={onSave} placeholder="en, es, ja" class="rounded-sm h-11 {fieldWidth}" />
                        </SettingRow>

                        <SettingRow label={i18n.t('settings.player_section.preferred_dub_lang')} description={i18n.t('settings.player_section.preferred_dub_lang_desc')}>
                            <Input bind:value={playerConfig.preferredDubLang} onchange={onSave} placeholder="ja, en" class="rounded-sm h-11 {fieldWidth}" />
                        </SettingRow>

                        <SettingRow label={i18n.t('settings.player_section.seek_step')} description={i18n.t('settings.player_section.seek_step_desc')}>
                            <SegmentedControl
                                    label={i18n.t('settings.player_section.seek_step')}
                                    value={playerConfig.seekStep.toString()}
                                    options={seekSteps}
                                    onValueChange={handleSeekStepChange}
                            />
                        </SettingRow>

                        <SettingRow inline for="autoNext" label={i18n.t('settings.player_section.autoplay')} description={i18n.t('settings.player_section.autoplay_desc')}>
                            <Switch id="autoNext" bind:checked={playerConfig.autoplayNextEpisode} onCheckedChange={onSave} />
                        </SettingRow>

                        <SettingRow inline for="resumeFromLastPos" label={i18n.t('settings.player_section.resume_playback')} description={i18n.t('settings.player_section.resume_playback_desc')}>
                            <Switch id="resumeFromLastPos" bind:checked={playerConfig.resumeFromLastPos} onCheckedChange={onSave} />
                        </SettingRow>

                        <SettingRow inline for="autoSkipIntro" label={i18n.t('settings.player_section.auto_skip_intro')} description={i18n.t('settings.player_section.auto_skip_intro_desc')}>
                            <Switch id="autoSkipIntro" bind:checked={playerConfig.autoSkipIntro} onCheckedChange={onSave} />
                        </SettingRow>

                        <SettingRow inline for="autoSkipOutro" label={i18n.t('settings.player_section.auto_skip_outro')} description={i18n.t('settings.player_section.auto_skip_outro_desc')}>
                            <Switch id="autoSkipOutro" bind:checked={playerConfig.autoSkipOutro} onCheckedChange={onSave} />
                        </SettingRow>
                    </div>

                    <!-- Video -->
                    <div>
                        <SettingRow label={i18n.t('settings.player_section.hwdec')} description={i18n.t('settings.player_section.hwdec_desc')}>
                            <div class="w-full sm:w-60">
                                <ResponsiveSelect value={playerConfig.hwdec} items={hwdecOptions} class="rounded-sm h-11" onValueChange={handleHwdecChange} />
                            </div>
                        </SettingRow>

                        <SettingRow label={i18n.t('settings.player_section.scale_algorithm')} description={i18n.t('settings.player_section.scale_algorithm_desc')}>
                            <SegmentedControl
                                    label={i18n.t('settings.player_section.scale_algorithm')}
                                    value={playerConfig.scaleAlgorithm}
                                    options={scaleOptions}
                                    onValueChange={handleScaleChange}
                            />
                        </SettingRow>

                        <SettingRow inline for="interpolation" label={i18n.t('settings.player_section.interpolation')} description={i18n.t('settings.player_section.interpolation_desc')}>
                            <Switch id="interpolation" bind:checked={playerConfig.interpolation} onCheckedChange={onSave} />
                        </SettingRow>

                        <SettingRow inline for="deband" label={i18n.t('settings.player_section.deband')} description={i18n.t('settings.player_section.deband_desc')}>
                            <Switch id="deband" bind:checked={playerConfig.deband} onCheckedChange={onSave} />
                        </SettingRow>

                        <SettingRow label={i18n.t('settings.player_section.anime4k')} description={i18n.t('settings.player_section.anime4k_desc')} extra={anime4kLink}>
                            <SegmentedControl
                                    label={i18n.t('settings.player_section.anime4k')}
                                    value={playerConfig.anime4kTier}
                                    options={anime4kTierOptions}
                                    onValueChange={handleAnime4kTierChange}
                            />
                        </SettingRow>

                        <SettingRow disabled={anime4kOff} label={i18n.t('settings.player_section.anime4k_mode')} description={i18n.t('settings.player_section.anime4k_mode_desc')}>
                            <SegmentedControl
                                    label={i18n.t('settings.player_section.anime4k_mode')}
                                    value={playerConfig.anime4kMode}
                                    options={anime4kModeOptions}
                                    onValueChange={handleAnime4kModeChange}
                            />
                        </SettingRow>
                    </div>

                    <!-- Keyboard shortcuts -->
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-x-10 px-5 py-2 rounded-xl border border-border/50 bg-muted/10">
                        {#each keyboardShortcuts as shortcut}
                            <div class="flex items-center justify-between gap-4 py-2.5">
                                <span class="text-sm font-medium">{shortcut.label}</span>
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
                </div>
            {/if}
        </Tabs.Content>

        <!-- THUMBNAILS -->
        <Tabs.Content value="player_thumbnails" class="focus-visible:outline-none mt-0">
            {#if activeTab === "player_thumbnails"}
                <div in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
                    <SettingRow inline for="thumbnailsEnabled" label={i18n.t('settings.player_section.thumbnails')} description={i18n.t('settings.player_section.thumbnails_desc')}>
                        <Switch id="thumbnailsEnabled" bind:checked={playerConfig.thumbnailsEnabled} onCheckedChange={onSave} />
                    </SettingRow>

                    <SettingRow disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_interval')} description={i18n.t('settings.player_section.thumb_interval_desc')}>
                        <SegmentedControl
                                label={i18n.t('settings.player_section.thumb_interval')}
                                value={playerConfig.thumbnailIntervalSeconds.toString()}
                                options={thumbIntervalOptions}
                                onValueChange={handleThumbIntervalChange}
                        />
                    </SettingRow>

                    <SettingRow disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_size')} description={i18n.t('settings.player_section.thumb_size_desc')}>
                        <SegmentedControl
                                label={i18n.t('settings.player_section.thumb_size')}
                                value={playerConfig.thumbnailWidth.toString()}
                                options={thumbWidthOptions}
                                onValueChange={handleThumbWidthChange}
                        />
                    </SettingRow>

                    <SettingRow inline for="thumbnailHls" disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_hls')} description={i18n.t('settings.player_section.thumb_hls_desc')}>
                        <Switch id="thumbnailHls" bind:checked={playerConfig.thumbnailHls} disabled={thumbsOff} onCheckedChange={onSave} />
                    </SettingRow>

                    <SettingRow inline for="thumbnailTorrent" disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_torrent')} description={i18n.t('settings.player_section.thumb_torrent_desc')}>
                        <Switch id="thumbnailTorrent" bind:checked={playerConfig.thumbnailTorrent} disabled={thumbsOff} onCheckedChange={onSave} />
                    </SettingRow>

                    <SettingRow inline for="thumbnailPreciseSeek" disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_precise')} description={i18n.t('settings.player_section.thumb_precise_desc')}>
                        <Switch id="thumbnailPreciseSeek" bind:checked={playerConfig.thumbnailPreciseSeek} disabled={thumbsOff} onCheckedChange={onSave} />
                    </SettingRow>
                </div>
            {/if}
        </Tabs.Content>

        <!-- SUBTITLES -->
        <Tabs.Content value="player_subtitles" class="focus-visible:outline-none mt-0">
            {#if activeTab === "player_subtitles"}
                <div in:fly={{ y: 8, duration: 220, easing: cubicOut }} class="space-y-6">
                    <!-- Text -->
                    <div>
                        {@render groupHeader(i18n.t('settings.player_section.sub_group_text'))}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <div class={cell}>
                                <div>
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_font')}</div>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_font_desc')}</p>
                                </div>
                                <ResponsiveSelect value={subConfig.font} items={fontOptions} class="rounded-sm h-11 w-full" onValueChange={handleFontChange} />
                            </div>

                            <div class={cell}>
                                <div>
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_font_size')}</div>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_font_size_desc')}</p>
                                </div>
                                <SegmentedControl
                                        label={i18n.t('settings.player_section.sub_font_size')}
                                        class="self-start"
                                        value={subConfig.fontSize.toString()}
                                        options={fontSizeOptions}
                                        onValueChange={handleFontSizeChange}
                                />
                            </div>

                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_scale')}</div>
                                </div>
                                <SettingSlider bind:value={subConfig.scale} min={0.5} max={2} step={0.05} format={(v) => `${v.toFixed(2)}×`} oncommit={onSave} />
                            </div>
                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_color')}</div>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_color_desc')}</p>
                                </div>
                                <ColorInput bind:value={subConfig.color} onchange={onSave} label={i18n.t('settings.player_section.sub_color')} />
                            </div>
                        </div>
                    </div>

                    <!-- Outline & shadow -->
                    <div>
                        {@render groupHeader(i18n.t('settings.player_section.sub_group_outline'))}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_border_color')}</div>
                                </div>
                                <ColorInput bind:value={subConfig.borderColor} onchange={onSave} label={i18n.t('settings.player_section.sub_border_color')} />
                            </div>
                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_border_size')}</div>
                                </div>
                                <SettingSlider bind:value={subConfig.borderSize} min={0} max={10} step={0.5} oncommit={onSave} />
                            </div>
                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_shadow_color')}</div>
                                </div>
                                <ColorInput bind:value={subConfig.shadowColor} onchange={onSave} label={i18n.t('settings.player_section.sub_shadow_color')} />
                            </div>
                            <div class="{cellRow}">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_shadow_offset')}</div>
                                </div>
                                <SettingSlider bind:value={subConfig.shadowOffset} min={0} max={10} step={0.5} oncommit={onSave} />
                            </div>
                        </div>
                    </div>

                    <!-- Placement & timing -->
                    <div>
                        {@render groupHeader(i18n.t('settings.player_section.sub_group_placement'))}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <div class={cell}>
                                <div class={cellLabel}>{i18n.t('settings.player_section.sub_justify')}</div>
                                <SegmentedControl
                                        label={i18n.t('settings.player_section.sub_justify')}
                                        class="self-start"
                                        value={subConfig.justify}
                                        options={alignmentOptions}
                                        onValueChange={handleAlignmentChange}
                                />
                            </div>

                            <div class={cell}>
                                <div class={cellLabel}>{i18n.t('settings.player_section.sub_delay')}</div>
                                <Input type="number" bind:value={subConfig.delay} onchange={onSave} class="rounded-sm h-11 w-full" />
                            </div>

                            <div class="{cellRow} md:col-span-2">
                                <div class="min-w-0">
                                    <div class={cellLabel}>{i18n.t('settings.player_section.sub_position')}</div>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_position_desc')}</p>
                                </div>
                                <SettingSlider bind:value={subConfig.position} min={0} max={100} step={1} oncommit={onSave} />
                            </div>
                        </div>
                    </div>

                    <!-- Behavior -->
                    <div>
                        {@render groupHeader(i18n.t('settings.player_section.sub_group_behavior'))}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <div class="{cellRow} md:col-span-2">
                                <div class="min-w-0">
                                    <label for="forceStyle" class={cellLabel}>{i18n.t('settings.player_section.sub_force_style')}</label>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_force_style_desc')}</p>
                                </div>
                                <Switch id="forceStyle" bind:checked={subConfig.forceStyle} onCheckedChange={onSave} />
                            </div>
                            <div class={cellRow}>
                                <div class="min-w-0">
                                    <label for="sdhFilter" class={cellLabel}>{i18n.t('settings.player_section.sub_sdh_filter')}</label>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_sdh_filter_desc')}</p>
                                </div>
                                <Switch id="sdhFilter" bind:checked={subConfig.sdhFilter} onCheckedChange={onSave} />
                            </div>
                            <div class={cellRow}>
                                <div class="min-w-0">
                                    <label for="sdhFilterHarder" class={cellLabel}>{i18n.t('settings.player_section.sub_sdh_filter_harder')}</label>
                                    <p class={cellDesc}>{i18n.t('settings.player_section.sub_sdh_filter_harder_desc')}</p>
                                </div>
                                <Switch id="sdhFilterHarder" bind:checked={subConfig.sdhFilterHarder} onCheckedChange={onSave} />
                            </div>
                        </div>
                    </div>
                </div>
            {/if}
        </Tabs.Content>
    </Tabs.Root>
</SettingsSection>