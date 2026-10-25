<script lang="ts">
    import type { PlayerConfig, SubtitleConfig } from '@/api/config/types.ts';
    import { i18n } from '@/stores/i18n.svelte.js';
    import TvSettingRow from '@/tv/TvSettingRow.svelte';
    import TvSwitch from '@/tv/TvSwitch.svelte';
    import TvSegmented from '@/tv/TvSegmented.svelte';
    import TvSlider from '@/tv/TvSlider.svelte';
    import TvColorSwatches from '@/tv/TvColorSwatches.svelte';

    let {
        playerConfig = $bindable(),
        subConfig = $bindable(),
        onSave
    }: {
        playerConfig: PlayerConfig;
        subConfig: SubtitleConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    const save = () => onSave();

    // ------------------------------------------------------------------ options
    const sec = (n: number) => ({ value: String(n), label: i18n.t('settings.player_section.seconds', { num: n }) });
    const seekSteps = [5, 10, 15, 30].map(sec);
    const thumbIntervalOptions = [5, 10, 20, 30].map(sec);

    const hwdecOptions = $derived([
        { value: 'auto-safe', label: i18n.t('watch.settings.video_section.hwdec_auto_safe') },
        { value: 'auto', label: i18n.t('watch.settings.video_section.hwdec_auto') },
        { value: 'no', label: i18n.t('watch.settings.video_section.hwdec_software') }
    ]);

    const scaleOptions = $derived([
        { value: 'bilinear', label: i18n.t('watch.settings.video_section.scale_fast') },
        { value: 'spline36', label: i18n.t('watch.settings.video_section.scale_balanced') },
        { value: 'ewa_lanczossharp', label: i18n.t('watch.settings.video_section.scale_sharp') }
    ]);

    const anime4kTierOptions = $derived([
        { value: 'off', label: i18n.t('settings.player_section.anime4k_off') },
        { value: 'fast', label: i18n.t('settings.player_section.anime4k_fast') },
        { value: 'hq', label: i18n.t('settings.player_section.anime4k_hq') }
    ]);

    const anime4kModeOptions = [
        { value: 'a', label: 'A' },
        { value: 'b', label: 'B' },
        { value: 'c', label: 'C' }
    ];

    const thumbWidthOptions = $derived([
        { value: '160', label: i18n.t('settings.player_section.thumb_size_small') },
        { value: '240', label: i18n.t('settings.player_section.thumb_size_medium') },
        { value: '320', label: i18n.t('settings.player_section.thumb_size_large') }
    ]);

    const fontOptions = [
        { value: 'sans-serif', label: 'Sans-serif' },
        { value: 'serif', label: 'Serif' },
        { value: 'monospace', label: 'Monospace' },
        { value: 'Arial', label: 'Arial' },
        { value: 'Roboto', label: 'Roboto' },
        { value: 'Open Sans', label: 'Open Sans' },
        { value: 'Trebuchet MS', label: 'Trebuchet MS' },
        { value: 'Georgia', label: 'Georgia' },
        { value: 'Comic Sans MS', label: 'Comic Sans MS' }
    ];

    const fontSizeOptions = $derived([
        { value: '36', label: i18n.t('watch.settings.subtitle_style_section.size_small') },
        { value: '55', label: i18n.t('watch.settings.subtitle_style_section.size_medium') },
        { value: '70', label: i18n.t('watch.settings.subtitle_style_section.size_large') },
        { value: '85', label: i18n.t('watch.settings.subtitle_style_section.size_extra_large') }
    ]);

    const alignmentOptions = $derived([
        { value: 'auto', label: i18n.t('watch.settings.subtitle_style_section.align_auto') },
        { value: 'left', label: i18n.t('watch.settings.subtitle_style_section.align_left') },
        { value: 'center', label: i18n.t('watch.settings.subtitle_style_section.align_center') },
        { value: 'right', label: i18n.t('watch.settings.subtitle_style_section.align_right') }
    ]);

    // ------------------------------------------------------------------ tabs
    let activeTab = $state('player_general');
    const tabs = $derived([
        { value: 'player_general', label: i18n.t('settings.general') },
        { value: 'player_thumbnails', label: i18n.t('settings.player_section.thumbnails') },
        { value: 'player_subtitles', label: i18n.t('watch.settings.subtitles') }
    ]);

    const anime4kOff = $derived(playerConfig.anime4kTier === 'off');
    const thumbsOff = $derived(!playerConfig.thumbnailsEnabled);

    const inputClass =
        'h-12 w-64 scroll-my-24 rounded-md border-2 border-border/60 bg-card px-4 text-lg outline-none ' +
        'focus:border-primary focus:ring-4 focus:ring-primary/40';
</script>

{#snippet groupHeader(title: string)}
    <h3 class="px-1 pb-1 pt-8 text-xs font-bold uppercase tracking-[0.14em] text-muted-foreground">{title}</h3>
{/snippet}

<div>
    <header class="space-y-1 pb-4">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.player')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.player_section.player_desc')}</p>
    </header>

    <div class="pb-2">
        <TvSegmented label={i18n.t('settings.player')} bind:value={activeTab} options={tabs} />
    </div>

    <!-- ============================================================ GENERAL -->
    {#if activeTab === 'player_general'}
        <div>
            <TvSettingRow
                    label={i18n.t('settings.player_section.preferred_sub_lang')}
                    description={i18n.t('settings.player_section.preferred_sub_lang_desc')}
            >
                <input data-sn type="text" bind:value={playerConfig.preferredSubLang} onchange={save} placeholder="en, es, ja" class={inputClass} />
            </TvSettingRow>

            <TvSettingRow
                    label={i18n.t('settings.player_section.preferred_dub_lang')}
                    description={i18n.t('settings.player_section.preferred_dub_lang_desc')}
            >
                <input data-sn type="text" bind:value={playerConfig.preferredDubLang} onchange={save} placeholder="ja, en" class={inputClass} />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    label={i18n.t('settings.player_section.seek_step')}
                    description={i18n.t('settings.player_section.seek_step_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.seek_step')}
                        value={String(playerConfig.seekStep)}
                        options={seekSteps}
                        onchange={(v) => { playerConfig.seekStep = parseInt(v); onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.autoplay')} description={i18n.t('settings.player_section.autoplay_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.autoplay')} bind:checked={playerConfig.autoplayNextEpisode} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.resume_playback')} description={i18n.t('settings.player_section.resume_playback_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.resume_playback')} bind:checked={playerConfig.resumeFromLastPos} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.auto_skip_intro')} description={i18n.t('settings.player_section.auto_skip_intro_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.auto_skip_intro')} bind:checked={playerConfig.autoSkipIntro} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.auto_skip_outro')} description={i18n.t('settings.player_section.auto_skip_outro_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.auto_skip_outro')} bind:checked={playerConfig.autoSkipOutro} onchange={save} />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    label={i18n.t('settings.player_section.hwdec')}
                    description={i18n.t('settings.player_section.hwdec_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.hwdec')}
                        value={playerConfig.hwdec}
                        options={hwdecOptions}
                        onchange={(v) => { playerConfig.hwdec = v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    label={i18n.t('settings.player_section.scale_algorithm')}
                    description={i18n.t('settings.player_section.scale_algorithm_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.scale_algorithm')}
                        value={playerConfig.scaleAlgorithm}
                        options={scaleOptions}
                        onchange={(v) => { playerConfig.scaleAlgorithm = v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.interpolation')} description={i18n.t('settings.player_section.interpolation_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.interpolation')} bind:checked={playerConfig.interpolation} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.deband')} description={i18n.t('settings.player_section.deband_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.deband')} bind:checked={playerConfig.deband} onchange={save} />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    label={i18n.t('settings.player_section.anime4k')}
                    description={i18n.t('settings.player_section.anime4k_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.anime4k')}
                        value={playerConfig.anime4kTier}
                        options={anime4kTierOptions}
                        onchange={(v) => { playerConfig.anime4kTier = v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    disabled={anime4kOff}
                    label={i18n.t('settings.player_section.anime4k_mode')}
                    description={i18n.t('settings.player_section.anime4k_mode_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.anime4k_mode')}
                        disabled={anime4kOff}
                        value={playerConfig.anime4kMode}
                        options={anime4kModeOptions}
                        onchange={(v) => { playerConfig.anime4kMode = v; onSave(); }}
                />
            </TvSettingRow>
        </div>

        <!-- ============================================================ THUMBNAILS -->
    {:else if activeTab === 'player_thumbnails'}
        <div>
            <TvSettingRow label={i18n.t('settings.player_section.thumbnails')} description={i18n.t('settings.player_section.thumbnails_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.thumbnails')} bind:checked={playerConfig.thumbnailsEnabled} onchange={save} />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    disabled={thumbsOff}
                    label={i18n.t('settings.player_section.thumb_interval')}
                    description={i18n.t('settings.player_section.thumb_interval_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.thumb_interval')}
                        disabled={thumbsOff}
                        value={String(playerConfig.thumbnailIntervalSeconds)}
                        options={thumbIntervalOptions}
                        onchange={(v) => { playerConfig.thumbnailIntervalSeconds = parseInt(v); onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow
                    stacked
                    disabled={thumbsOff}
                    label={i18n.t('settings.player_section.thumb_size')}
                    description={i18n.t('settings.player_section.thumb_size_desc')}
            >
                <TvSegmented
                        label={i18n.t('settings.player_section.thumb_size')}
                        disabled={thumbsOff}
                        value={String(playerConfig.thumbnailWidth)}
                        options={thumbWidthOptions}
                        onchange={(v) => { playerConfig.thumbnailWidth = parseInt(v); onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_hls')} description={i18n.t('settings.player_section.thumb_hls_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.thumb_hls')} disabled={thumbsOff} bind:checked={playerConfig.thumbnailHls} onchange={save} />
            </TvSettingRow>

            <TvSettingRow disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_torrent')} description={i18n.t('settings.player_section.thumb_torrent_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.thumb_torrent')} disabled={thumbsOff} bind:checked={playerConfig.thumbnailTorrent} onchange={save} />
            </TvSettingRow>

            <TvSettingRow disabled={thumbsOff} label={i18n.t('settings.player_section.thumb_precise')} description={i18n.t('settings.player_section.thumb_precise_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.thumb_precise')} disabled={thumbsOff} bind:checked={playerConfig.thumbnailPreciseSeek} onchange={save} />
            </TvSettingRow>
        </div>

        <!-- ============================================================ SUBTITLES -->
    {:else if activeTab === 'player_subtitles'}
        <div>
            {@render groupHeader(i18n.t('settings.player_section.sub_group_text'))}

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_font')} description={i18n.t('settings.player_section.sub_font_desc')}>
                <TvSegmented
                        label={i18n.t('settings.player_section.sub_font')}
                        value={subConfig.font}
                        options={fontOptions}
                        onchange={(v) => { subConfig.font = v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_font_size')} description={i18n.t('settings.player_section.sub_font_size_desc')}>
                <TvSegmented
                        label={i18n.t('settings.player_section.sub_font_size')}
                        value={String(subConfig.fontSize)}
                        options={fontSizeOptions}
                        onchange={(v) => { subConfig.fontSize = parseInt(v); onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_scale')}>
                <TvSlider
                        label={i18n.t('settings.player_section.sub_scale')}
                        bind:value={subConfig.scale}
                        min={0.5}
                        max={2}
                        step={0.05}
                        format={(v) => `${v.toFixed(2)}×`}
                        oncommit={save}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_color')} description={i18n.t('settings.player_section.sub_color_desc')}>
                <TvColorSwatches
                        label={i18n.t('settings.player_section.sub_color')}
                        value={subConfig.color}
                        onchange={(c) => { subConfig.color = c; onSave(); }}
                />
            </TvSettingRow>

            {@render groupHeader(i18n.t('settings.player_section.sub_group_outline'))}

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_border_color')}>
                <TvColorSwatches
                        label={i18n.t('settings.player_section.sub_border_color')}
                        value={subConfig.borderColor}
                        onchange={(c) => { subConfig.borderColor = c; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_border_size')}>
                <TvSlider
                        label={i18n.t('settings.player_section.sub_border_size')}
                        bind:value={subConfig.borderSize}
                        min={0}
                        max={10}
                        step={0.5}
                        oncommit={save}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_shadow_color')}>
                <TvColorSwatches
                        label={i18n.t('settings.player_section.sub_shadow_color')}
                        value={subConfig.shadowColor}
                        onchange={(c) => { subConfig.shadowColor = c; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_shadow_offset')}>
                <TvSlider
                        label={i18n.t('settings.player_section.sub_shadow_offset')}
                        bind:value={subConfig.shadowOffset}
                        min={0}
                        max={10}
                        step={0.5}
                        oncommit={save}
                />
            </TvSettingRow>

            {@render groupHeader(i18n.t('settings.player_section.sub_group_placement'))}

            <TvSettingRow stacked label={i18n.t('settings.player_section.sub_justify')}>
                <TvSegmented
                        label={i18n.t('settings.player_section.sub_justify')}
                        value={subConfig.justify}
                        options={alignmentOptions}
                        onchange={(v) => { subConfig.justify = v as 'auto' | 'left' | 'center' | 'right'; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_delay')}>
                <TvSlider
                        label={i18n.t('settings.player_section.sub_delay')}
                        bind:value={subConfig.delay}
                        min={-20}
                        max={20}
                        step={0.5}
                        format={(v) => `${v > 0 ? '+' : ''}${v.toFixed(1)}`}
                        oncommit={save}
                />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_position')} description={i18n.t('settings.player_section.sub_position_desc')}>
                <TvSlider
                        label={i18n.t('settings.player_section.sub_position')}
                        bind:value={subConfig.position}
                        min={0}
                        max={100}
                        step={1}
                        oncommit={save}
                />
            </TvSettingRow>

            {@render groupHeader(i18n.t('settings.player_section.sub_group_behavior'))}

            <TvSettingRow label={i18n.t('settings.player_section.sub_force_style')} description={i18n.t('settings.player_section.sub_force_style_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.sub_force_style')} bind:checked={subConfig.forceStyle} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_sdh_filter')} description={i18n.t('settings.player_section.sub_sdh_filter_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.sub_sdh_filter')} bind:checked={subConfig.sdhFilter} onchange={save} />
            </TvSettingRow>

            <TvSettingRow label={i18n.t('settings.player_section.sub_sdh_filter_harder')} description={i18n.t('settings.player_section.sub_sdh_filter_harder_desc')}>
                <TvSwitch label={i18n.t('settings.player_section.sub_sdh_filter_harder')} bind:checked={subConfig.sdhFilterHarder} onchange={save} />
            </TvSettingRow>
        </div>
    {/if}
</div>