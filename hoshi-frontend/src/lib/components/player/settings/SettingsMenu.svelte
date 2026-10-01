<script lang="ts">
    import { fly, fade } from "svelte/transition";
    import { appConfig } from "@/stores/config.svelte.js";
    import {
        PuzzleIcon,
        Server,
        Mic2,
        AudioLines,
        Captions,
        Gauge,
        Cpu,
        Palette,
        Type,
        AlignCenter,
        Activity,
        Contrast,
        Sparkles,
        SlidersHorizontal,
        Maximize,
        Maximize2,
        Baseline,
        Brush,
        Ruler,
        Square,
        PaintBucket,
        MoveVertical,
        Timer,
        Droplet,
        Layers,
        Lock,
        EyeOff
    } from "lucide-svelte";
    import SettingsRow from "@/components/player/settings/SettingsRow.svelte";
    import SettingsOptionList from "@/components/player/settings/SettingsOptionList.svelte";
    import SettingsHeader from "@/components/player/settings/SettingsHeader.svelte";
    import SettingsSwitchRow from "@/components/player/settings/SettingsSwitchRow.svelte";
    import SettingsSliderRow from "@/components/player/settings/SettingsSliderRow.svelte";
    import SettingsColorRow from "@/components/player/settings/SettingsColorRow.svelte";
    import type { WatchState } from "@/app/watch.svelte.js";
    import type { PlaybackTrack } from "@/app/watch.svelte.js";
    import {i18n} from "@/stores/i18n.svelte.js";

    let { pageState, isMobile = false }: { pageState: WatchState, isMobile?: boolean } = $props();

    type SectionId = "source" | "server" | "audio" | "subtitles" | "quality" | "video" | "subtitleStyle";
    type OptionId = "hwdec" | "scaleAlgorithm" | "font" | "fontSize" | "alignment" | "anime4kTier" | "anime4kMode";

    let activeSection = $state<SectionId | null>(null);
    let activeOption = $state<OptionId | null>(null);

    let langDisplayNames: Intl.DisplayNames | null = null;
    try {
        langDisplayNames = new Intl.DisplayNames(["en"], { type: "language" });
    } catch {
        langDisplayNames = null;
    }

    function languageName(code: string): string | null {
        const normalized = code.trim().toLowerCase();
        if (!normalized || normalized === "und" || !langDisplayNames) return null;
        try {
            const name = langDisplayNames.of(normalized);
            if (!name || name.toLowerCase() === normalized) return null;
            return name;
        } catch {
            return null;
        }
    }

    function isFilenameLikeTitle(title: string): boolean {
        return /\.(srt|vtt|ass|ssa|sub|ttml)$/i.test(title.trim());
    }

    function baseTrackLabel(track: PlaybackTrack): string {
        if (track.title && !isFilenameLikeTitle(track.title)) return track.title;
        if (track.lang) return languageName(track.lang) || track.lang;
        return track.title || `Track ${track.id}`;
    }

    function buildTrackLabels(tracks: PlaybackTrack[]): Map<number, string> {
        const bases = tracks.map(t => ({ id: t.id, base: baseTrackLabel(t) }));
        const totals = new Map<string, number>();
        for (const { base } of bases) totals.set(base, (totals.get(base) ?? 0) + 1);

        const seen = new Map<string, number>();
        const labels = new Map<number, string>();
        for (const { id, base } of bases) {
            if ((totals.get(base) ?? 1) <= 1) {
                labels.set(id, base);
                continue;
            }
            const index = (seen.get(base) ?? 0) + 1;
            seen.set(base, index);
            labels.set(id, index === 1 ? base : `${base} (${index})`);
        }
        return labels;
    }

    function qualityLabel(track: PlaybackTrack): string {
        if (track.title) return track.title;
        const parts: string[] = [];
        if (track.h) parts.push(`${track.h}p`);
        if (track.bitrate) {
            const mbps = track.bitrate / 1_000_000;
            parts.push(mbps >= 1 ? `${mbps.toFixed(1)} Mbps` : `${Math.round(track.bitrate / 1000)} kbps`);
        }
        return parts.length ? parts.join(" · ") : `Track ${track.id}`;
    }

    // --- Track-based section data ---
    const currentSource = $derived(
        pageState.extensionItems.find(i => i.value === pageState.selectedExtension)?.label || i18n.t("watch.settings.default")
    );
    const currentServer = $derived(
        pageState.serverItems.find(i => i.value === pageState.selectedServer)?.label || i18n.t("watch.settings.default")
    );
    // Torrent sources have no servers to pick from.
    const showServer = $derived(!pageState.isTorrent && pageState.serverItems.length > 0);
    const audioLabels = $derived.by(() => buildTrackLabels(pageState.audioTracks));
    const subtitleLabels = $derived.by(() => buildTrackLabels(pageState.subtitleTracks));

    const currentAudio = $derived.by(() => {
        const track = pageState.audioTracks.find(t => t.selected);
        return track ? audioLabels.get(track.id) ?? baseTrackLabel(track) : i18n.t("watch.settings.default");
    });
    const currentSub = $derived.by(() => {
        const track = pageState.subtitleTracks.find(t => t.selected);
        return track ? subtitleLabels.get(track.id) ?? baseTrackLabel(track) : i18n.t("watch.settings.off");
    });
    const currentQuality = $derived.by(() => {
        const track = pageState.videoTracks.find(t => t.selected);
        return track ? qualityLabel(track) : i18n.t("watch.settings.auto");
    });

    const activeSubId = $derived.by(() => {
        const selected = pageState.subtitleTracks.find(t => t.selected);
        return selected ? String(selected.id) : i18n.t("watch.settings.off");
    });
    const activeAudioId = $derived.by(() => {
        const selected = pageState.audioTracks.find(t => t.selected);
        return selected ? String(selected.id) : "";
    });
    const activeVideoId = $derived.by(() => {
        const selected = pageState.videoTracks.find(t => t.selected);
        return selected ? String(selected.id) : "";
    });

    // --- Plain "choose one" options for player/subtitle settings ---
    const anime4kTierOptions = [
        { id: "off", label: "Off" },
        { id: "fast", label: "Fast" },
        { id: "hq", label: "HQ" },
    ];
    const anime4kModeOptions = [
        { id: "a", label: "A" },
        { id: "b", label: "B" },
        { id: "c", label: "C" },
    ];

    const hwdecOptions = [
        { id: "auto-safe", label: i18n.t("watch.settings.video_section.hwdec_auto_safe") },
        { id: "auto", label: i18n.t("watch.settings.video_section.hwdec_auto") },
        { id: "no", label: i18n.t("watch.settings.video_section.hwdec_software") }
    ];
    const scaleOptions = [
        { id: "bilinear", label: i18n.t("watch.settings.video_section.scale_fast") },
        { id: "spline36", label: i18n.t("watch.settings.video_section.scale_balanced") },
        { id: "ewa_lanczossharp", label: i18n.t("watch.settings.video_section.scale_sharp") }
    ];
    const fontOptions = [
        { id: "sans-serif", label: "Sans-serif" },
        { id: "serif", label: "Serif" },
        { id: "monospace", label: "Monospace" },
        { id: "Arial", label: "Arial" },
        { id: "Roboto", label: "Roboto" },
        { id: "Open Sans", label: "Open Sans" },
        { id: "Trebuchet MS", label: "Trebuchet MS" },
        { id: "Georgia", label: "Georgia" },
        { id: "Comic Sans MS", label: "Comic Sans MS" }
    ];
    const fontSizeOptions = [
        { id: "36", label: i18n.t("watch.settings.subtitle_style_section.size_small") },
        { id: "55", label: i18n.t("watch.settings.subtitle_style_section.size_medium") },
        { id: "70", label: i18n.t("watch.settings.subtitle_style_section.size_large") },
        { id: "85", label: i18n.t("watch.settings.subtitle_style_section.size_extra_large") }
    ];
    const alignmentOptions = [
        { id: "auto", label: i18n.t("watch.settings.subtitle_style_section.align_auto") },
        { id: "left", label: i18n.t("watch.settings.subtitle_style_section.align_left") },
        { id: "center", label: i18n.t("watch.settings.subtitle_style_section.align_center") },
        { id: "right", label: i18n.t("watch.settings.subtitle_style_section.align_right") }
    ];

    function labelFor(options: { id: string; label: string }[], id: string | undefined, fallback: string) {
        return options.find(o => o.id === id)?.label ?? fallback;
    }

    const currentHwdec = $derived(labelFor(hwdecOptions, appConfig.data?.player.hwdec, "Auto (safe)"));
    const currentScale = $derived(labelFor(scaleOptions, appConfig.data?.player.scaleAlgorithm, "Balanced"));
    const currentAnime4kMode = $derived(labelFor(anime4kModeOptions, appConfig.data?.player.anime4kMode, "Off"));
    const currentAnime4kTier = $derived(labelFor(anime4kTierOptions, appConfig.data?.player.anime4kTier, "Fast"));
    const currentFont = $derived(labelFor(fontOptions, appConfig.data?.subtitles.font, appConfig.data?.subtitles.font ?? "Sans-serif"));
    const currentFontSize = $derived(labelFor(fontSizeOptions, String(appConfig.data?.subtitles.fontSize ?? ""), "Medium"));
    const currentAlignment = $derived(labelFor(alignmentOptions, appConfig.data?.subtitles.justify, "Auto"));

    function selectOption(action: () => void) {
        action();
        activeSection = null;
        activeOption = null;
    }

    function selectSubOption(action: () => void) {
        action();
        activeOption = null;
    }

    function goBack() {
        if (activeOption) {
            activeOption = null;
        } else {
            activeSection = null;
        }
    }

    // --- Titles + option views (replaces the copy-pasted per-option markup) ---
    const sectionTitle = $derived.by(() => {
        switch (activeSection) {
            case "source": return i18n.t("watch.settings.source");
            case "server": return i18n.t("watch.settings.server");
            case "audio": return i18n.t("watch.settings.audio_track");
            case "subtitles": return i18n.t("watch.settings.subtitles");
            case "quality": return i18n.t("watch.settings.quality");
            case "video": return i18n.t("watch.settings.video");
            case "subtitleStyle": return i18n.t("watch.settings.subtitle_style");
            default: return "";
        }
    });

    type OptionView = {
        title: string;
        options: { id: string; label: string }[];
        activeId: string;
        apply: (id: string) => void;
    };

    const optionView = $derived.by((): OptionView | null => {
        const cfg = appConfig.data;
        if (!activeOption || !cfg) return null;
        switch (activeOption) {
            case "hwdec":
                return { title: i18n.t("watch.settings.video_section.hardware_decoding"), options: hwdecOptions, activeId: cfg.player.hwdec, apply: (id) => appConfig.update({ player: { hwdec: id } }) };
            case "scaleAlgorithm":
                return { title: i18n.t("watch.settings.video_section.upscale_quality"), options: scaleOptions, activeId: cfg.player.scaleAlgorithm, apply: (id) => appConfig.update({ player: { scaleAlgorithm: id } }) };
            case "anime4kTier":
                return { title: "Anime4K Tier", options: anime4kTierOptions, activeId: cfg.player.anime4kTier, apply: (id) => appConfig.update({ player: { anime4kTier: id } }) };
            case "anime4kMode":
                return { title: "Anime4K Mode", options: anime4kModeOptions, activeId: cfg.player.anime4kMode, apply: (id) => appConfig.update({ player: { anime4kMode: id } }) };
            case "font":
                return { title: i18n.t("watch.settings.subtitle_style_section.font"), options: fontOptions, activeId: cfg.subtitles.font, apply: (id) => appConfig.update({ subtitles: { font: id } }) };
            case "fontSize":
                return { title: i18n.t("watch.settings.subtitle_style_section.font_size"), options: fontSizeOptions, activeId: String(cfg.subtitles.fontSize), apply: (id) => appConfig.update({ subtitles: { fontSize: Number(id) } }) };
            case "alignment":
                return { title: i18n.t("watch.settings.subtitle_style_section.alignment"), options: alignmentOptions, activeId: cfg.subtitles.justify, apply: (id) => appConfig.update({ subtitles: { justify: id as any } }) };
        }
    });

    // One scroll container per view (the old subtitle-style view nested two).
    const scrollClass = $derived(
        isMobile ? "" : activeSection === "subtitleStyle" && !activeOption ? "max-h-80 overflow-y-auto" : "max-h-64 overflow-y-auto"
    );
</script>

{#snippet menuContent()}
    {#if optionView}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <SettingsHeader title={optionView.title} onclick={goBack} />
            <div class="flex flex-col {scrollClass}">
                <SettingsOptionList
                        options={optionView.options}
                        activeId={optionView.activeId}
                        onSelect={(id) => selectSubOption(() => optionView.apply(id))}
                />
            </div>
        </div>

    {:else if activeSection === null}
        <!-- Main Settings Menu -->
        <div in:fly={{ x: -8, duration: 150 }} class="flex flex-col py-0.5">

            {#if pageState.extensionItems.length > 0}
                <SettingsRow icon={PuzzleIcon} label={i18n.t("watch.settings.source")} value={currentSource} onclick={() => activeSection = "source"} />
            {/if}

            {#if showServer}
                <SettingsRow icon={Server} label={i18n.t("watch.settings.server")} value={currentServer} onclick={() => activeSection = "server"} />
            {/if}

            {#if pageState.supportsDub}
                <SettingsSwitchRow
                        icon={Mic2}
                        label={i18n.t("watch.settings.dub_audio")}
                        checked={pageState.isDub}
                        onCheckedChange={() => pageState.toggleDub()}
                />
            {/if}

            {#if appConfig.data?.player}
                <SettingsRow icon={Cpu} label={i18n.t("watch.settings.video")} onclick={() => activeSection = "video"} />
            {/if}

            {#if appConfig.data?.subtitles}
                <SettingsRow icon={Palette} label={i18n.t("watch.settings.subtitle_style")} onclick={() => activeSection = "subtitleStyle"} />
            {/if}

            {#if (pageState.extensionItems.length > 0 || showServer || pageState.supportsDub) && (pageState.videoTracks.length > 1 || pageState.audioTracks.length > 0 || pageState.subtitleTracks.length > 0)}
                <div class="h-px bg-border my-1 mx-2"></div>
            {/if}

            {#if pageState.videoTracks.length > 1}
                <SettingsRow icon={Gauge} label={i18n.t("watch.settings.quality")} value={currentQuality} onclick={() => activeSection = "quality"} />
            {/if}

            {#if pageState.audioTracks.length > 1}
                <SettingsRow icon={AudioLines} label={i18n.t("watch.settings.audio_track")} value={currentAudio} onclick={() => activeSection = "audio"} />
            {/if}

            {#if pageState.subtitleTracks.length > 0}
                <SettingsRow icon={Captions} label={i18n.t("watch.settings.subtitles")} value={currentSub} onclick={() => activeSection = "subtitles"} />
            {/if}
        </div>

    {:else}
        <!-- Submenu View -->
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <SettingsHeader title={sectionTitle} onclick={goBack} />

            <div class="flex flex-col {scrollClass}">
                {#if activeSection === "source"}
                    <SettingsOptionList
                            options={pageState.extensionItems.map(i => ({ id: i.value, label: i.label }))}
                            activeId={pageState.selectedExtension}
                            onSelect={(id) => selectOption(() => pageState.selectExtension(id))}
                    />

                {:else if activeSection === "server"}
                    <SettingsOptionList
                            options={pageState.serverItems.map(i => ({ id: i.value, label: i.label }))}
                            activeId={pageState.selectedServer}
                            onSelect={(id) => selectOption(() => pageState.selectServer(id))}
                    />

                {:else if activeSection === "audio"}
                    <SettingsOptionList
                            options={pageState.audioTracks.map(t => ({ id: String(t.id), label: audioLabels.get(t.id) ?? baseTrackLabel(t) }))}
                            activeId={activeAudioId}
                            onSelect={(id) => selectOption(() => pageState.setAudioTrack(Number(id)))}
                    />

                {:else if activeSection === "video" && appConfig.data}
                    {@const player = appConfig.data.player}
                    <SettingsRow icon={Cpu} label={i18n.t("watch.settings.video_section.hardware_decoding")} value={currentHwdec} onclick={() => activeOption = "hwdec"} />
                    <SettingsRow icon={Maximize} label={i18n.t("watch.settings.video_section.upscale_quality")} value={currentScale} onclick={() => activeOption = "scaleAlgorithm"} />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <SettingsSwitchRow
                            icon={Activity}
                            label={i18n.t("watch.settings.video_section.smooth_motion")}
                            checked={player.interpolation}
                            onCheckedChange={(v) => appConfig.update({ player: { interpolation: v } })}
                    />
                    <SettingsSwitchRow
                            icon={Contrast}
                            label={i18n.t("watch.settings.video_section.reduce_banding")}
                            checked={player.deband}
                            onCheckedChange={(v) => appConfig.update({ player: { deband: v } })}
                    />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <SettingsRow icon={Sparkles} label="Anime4K" value={currentAnime4kTier} onclick={() => activeOption = "anime4kTier"} />
                    {#if player.anime4kTier !== "off"}
                        <SettingsRow icon={SlidersHorizontal} label="Anime4K Mode" value={currentAnime4kMode} onclick={() => activeOption = "anime4kMode"} />
                    {/if}

                {:else if activeSection === "subtitleStyle" && appConfig.data}
                    {@const sub = appConfig.data.subtitles}

                    <!-- Text -->
                    <SettingsRow icon={Type} label={i18n.t("watch.settings.subtitle_style_section.font")} value={currentFont} onclick={() => activeOption = "font"} />
                    <SettingsRow icon={Baseline} label={i18n.t("watch.settings.subtitle_style_section.font_size")} value={currentFontSize} onclick={() => activeOption = "fontSize"} />
                    <SettingsSliderRow
                            icon={Maximize2}
                            label={i18n.t("watch.settings.subtitle_style_section.size_scale", { "scale": sub.scale.toFixed(2) })}
                            value={sub.scale} min={0.5} max={2} step={0.05}
                            onInput={(v) => appConfig.update({ subtitles: { scale: v } })}
                    />
                    <SettingsColorRow
                            icon={Palette}
                            label={i18n.t("watch.settings.subtitle_style_section.text_color")}
                            value={sub.color}
                            onInput={(v) => appConfig.update({ subtitles: { color: v } })}
                    />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <!-- Outline -->
                    <SettingsColorRow
                            icon={Brush}
                            label={i18n.t("watch.settings.subtitle_style_section.outline_color")}
                            value={sub.borderColor}
                            onInput={(v) => appConfig.update({ subtitles: { borderColor: v } })}
                    />
                    <SettingsSliderRow
                            icon={Ruler}
                            label={i18n.t("watch.settings.subtitle_style_section.outline_size", { "size": sub.borderSize })}
                            value={sub.borderSize} min={0} max={6} step={0.5}
                            onInput={(v) => appConfig.update({ subtitles: { borderSize: v } })}
                    />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <!-- Background -->
                    <SettingsSwitchRow
                            icon={Square}
                            label={i18n.t("watch.settings.subtitle_style_section.background_box")}
                            checked={sub.backgroundColor !== null}
                            onCheckedChange={(v) => appConfig.update({ subtitles: { backgroundColor: v ? "#000000AA" : null } })}
                    />
                    {#if sub.backgroundColor !== null}
                        <SettingsColorRow
                                nested
                                label={i18n.t("watch.settings.subtitle_style_section.background_color")}
                                value={sub.backgroundColor.slice(0, 7)}
                                onInput={(v) => appConfig.update({ subtitles: { backgroundColor: v + "AA" } })}
                        />
                    {/if}

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <!-- Shadow -->
                    <SettingsColorRow
                            icon={Droplet}
                            label={i18n.t("watch.settings.subtitle_style_section.shadow_color")}
                            value={sub.shadowColor.slice(0, 7)}
                            onInput={(v) => appConfig.update({ subtitles: { shadowColor: v + "FF" } })}
                    />
                    <SettingsSliderRow
                            icon={Layers}
                            label={i18n.t("watch.settings.subtitle_style_section.shadow_offset", { "offset": sub.shadowOffset })}
                            value={sub.shadowOffset} min={0} max={6} step={0.5}
                            onInput={(v) => appConfig.update({ subtitles: { shadowOffset: v } })}
                    />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <!-- Position & timing -->
                    <SettingsSliderRow
                            icon={MoveVertical}
                            label={i18n.t("watch.settings.subtitle_style_section.position", { "percent": sub.position })}
                            value={sub.position} min={0} max={100} step={1}
                            onInput={(v) => appConfig.update({ subtitles: { position: v } })}
                    />
                    <SettingsRow icon={AlignCenter} label={i18n.t("watch.settings.subtitle_style_section.alignment")} value={currentAlignment} onclick={() => activeOption = "alignment"} />
                    <SettingsSliderRow
                            icon={Timer}
                            label={i18n.t("watch.settings.subtitle_style_section.delay", { "seconds": sub.delay.toFixed(1) })}
                            value={sub.delay} min={-10} max={10} step={0.1}
                            onInput={(v) => appConfig.update({ subtitles: { delay: v } })}
                    />

                    <div class="h-px bg-border my-1 mx-2"></div>

                    <!-- Filtering -->
                    <SettingsSwitchRow
                            icon={Lock}
                            label={i18n.t("watch.settings.subtitle_style_section.force_style")}
                            checked={sub.forceStyle}
                            onCheckedChange={(v) => appConfig.update({ subtitles: { forceStyle: v } })}
                    />
                    <SettingsSwitchRow
                            icon={EyeOff}
                            label={i18n.t("watch.settings.subtitle_style_section.hide_sdh")}
                            checked={sub.sdhFilter}
                            onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilter: v, sdhFilterHarder: v ? sub.sdhFilterHarder : false } })}
                    />
                    {#if sub.sdhFilter}
                        <SettingsSwitchRow
                                nested
                                label={i18n.t("watch.settings.subtitle_style_section.sdh_aggressive")}
                                checked={sub.sdhFilterHarder}
                                onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilterHarder: v } })}
                        />
                    {/if}

                {:else if activeSection === "quality"}
                    <SettingsOptionList
                            options={pageState.videoTracks.map(t => ({ id: String(t.id), label: qualityLabel(t) }))}
                            activeId={activeVideoId}
                            onSelect={(id) => selectOption(() => pageState.setVideoTrack(Number(id)))}
                    />

                {:else if activeSection === "subtitles"}
                    <SettingsOptionList
                            options={[{ id: "off", label: i18n.t("watch.settings.off") }, ...pageState.subtitleTracks.map(t => ({ id: String(t.id), label: subtitleLabels.get(t.id) ?? baseTrackLabel(t) }))]}
                            activeId={activeSubId}
                            onSelect={(id) => selectOption(() => pageState.setSubtitleTrack(id === "off" ? null : Number(id)))}
                    />
                {/if}
            </div>
        </div>
    {/if}
{/snippet}

{#if !isMobile}
    <!-- Desktop: Floating Popup -->
    <div
            in:fly={{ y: 12, duration: 200 }}
            out:fade={{ duration: 150 }}
            class="absolute bottom-full right-0 mb-12 w-102 bg-popover border border-border rounded-sm shadow-2xl text-sm overflow-hidden flex flex-col p-1.5 z-50 text-popover-foreground"
    >
        {@render menuContent()}
    </div>
{:else}
    <div class="flex flex-col text-sm text-foreground w-full">
        {@render menuContent()}
    </div>
{/if}