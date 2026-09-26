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
        ChevronLeft,
        Cpu,
        Palette,
        Type,
        AlignCenter
    } from "lucide-svelte";
    import { Switch } from "@/components/ui/switch";
    import SettingsRow from "@/components/player/settings/SettingsRow.svelte";
    import SettingsOptionList from "@/components/player/settings/SettingsOptionList.svelte";
    import type { WatchState } from "@/app/watch.svelte.js";
    import type { PlaybackTrack } from "@/app/watch.svelte.js";

    let { pageState, isMobile = false }: { pageState: WatchState, isMobile?: boolean } = $props();

    type SectionId = "source" | "server" | "audio" | "subtitles" | "quality" | "video" | "subtitleStyle";
    type OptionId = "hwdec" | "scaleAlgorithm" | "font" | "fontSize" | "alignment";

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
        pageState.extensionItems.find(i => i.value === pageState.selectedExtension)?.label || "Default"
    );
    const currentServer = $derived(
        pageState.serverItems.find(i => i.value === pageState.selectedServer)?.label || "Default"
    );
    const audioLabels = $derived.by(() => buildTrackLabels(pageState.audioTracks));
    const subtitleLabels = $derived.by(() => buildTrackLabels(pageState.subtitleTracks));

    const currentAudio = $derived.by(() => {
        const track = pageState.audioTracks.find(t => t.selected);
        return track ? audioLabels.get(track.id) ?? baseTrackLabel(track) : "Default";
    });
    const currentSub = $derived.by(() => {
        const track = pageState.subtitleTracks.find(t => t.selected);
        return track ? subtitleLabels.get(track.id) ?? baseTrackLabel(track) : "Off";
    });
    const currentQuality = $derived.by(() => {
        const track = pageState.videoTracks.find(t => t.selected);
        return track ? qualityLabel(track) : "Auto";
    });

    const activeSubId = $derived.by(() => {
        const selected = pageState.subtitleTracks.find(t => t.selected);
        return selected ? String(selected.id) : "off";
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
    const hwdecOptions = [
        { id: "auto-safe", label: "Auto (safe)" },
        { id: "auto", label: "Auto" },
        { id: "no", label: "Software only" }
    ];
    const scaleOptions = [
        { id: "bilinear", label: "Fast" },
        { id: "spline36", label: "Balanced" },
        { id: "ewa_lanczossharp", label: "Sharp" }
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
        { id: "36", label: "Small" },
        { id: "55", label: "Medium" },
        { id: "70", label: "Large" },
        { id: "85", label: "Extra large" }
    ];
    const alignmentOptions = [
        { id: "auto", label: "Auto" },
        { id: "left", label: "Left" },
        { id: "center", label: "Center" },
        { id: "right", label: "Right" }
    ];

    function labelFor(options: { id: string; label: string }[], id: string | undefined, fallback: string) {
        return options.find(o => o.id === id)?.label ?? fallback;
    }

    const currentHwdec = $derived(labelFor(hwdecOptions, appConfig.data?.player.hwdec, "Auto (safe)"));
    const currentScale = $derived(labelFor(scaleOptions, appConfig.data?.player.scaleAlgorithm, "Balanced"));
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
</script>

{#snippet menuContent()}
    {#if activeOption === "hwdec" && appConfig.data}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold">Hardware decoding</span>
            </button>
            <SettingsOptionList
                    options={hwdecOptions}
                    activeId={appConfig.data.player.hwdec}
                    onSelect={(id) => selectSubOption(() => appConfig.update({ player: { hwdec: id } }))}
            />
        </div>

    {:else if activeOption === "scaleAlgorithm" && appConfig.data}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold">Upscale quality</span>
            </button>
            <SettingsOptionList
                    options={scaleOptions}
                    activeId={appConfig.data.player.scaleAlgorithm}
                    onSelect={(id) => selectSubOption(() => appConfig.update({ player: { scaleAlgorithm: id } }))}
            />
        </div>

    {:else if activeOption === "font" && appConfig.data}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold">Font</span>
            </button>
            <div class="flex flex-col {isMobile ? '' : 'max-h-64 overflow-y-auto'}">
                <SettingsOptionList
                        options={fontOptions}
                        activeId={appConfig.data.subtitles.font}
                        onSelect={(id) => selectSubOption(() => appConfig.update({ subtitles: { font: id } }))}
                />
            </div>
        </div>

    {:else if activeOption === "fontSize" && appConfig.data}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold">Font size</span>
            </button>
            <SettingsOptionList
                    options={fontSizeOptions}
                    activeId={String(appConfig.data.subtitles.fontSize)}
                    onSelect={(id) => selectSubOption(() => appConfig.update({ subtitles: { fontSize: Number(id) } }))}
            />
        </div>

    {:else if activeOption === "alignment" && appConfig.data}
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold">Alignment</span>
            </button>
            <SettingsOptionList
                    options={alignmentOptions}
                    activeId={appConfig.data.subtitles.justify}
                    onSelect={(id) => selectSubOption(() => appConfig.update({ subtitles: { justify: id as any } }))}
            />
        </div>

    {:else if activeSection === null}
        <!-- Main Settings Menu -->
        <div in:fly={{ x: -8, duration: 150 }} class="flex flex-col py-0.5">

            {#if pageState.extensionItems.length > 0}
                <SettingsRow icon={PuzzleIcon} label="Source" value={currentSource} onclick={() => activeSection = "source"} />
            {/if}

            {#if pageState.serverItems.length > 0}
                <SettingsRow icon={Server} label="Server" value={currentServer} onclick={() => activeSection = "server"} />
            {/if}

            {#if pageState.supportsDub}
                <div class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-accent transition text-left">
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-muted group-hover:bg-accent transition-colors">
                            <Mic2 class="w-4 h-4 text-foreground/80 group-hover:text-foreground" />
                        </div>
                        <span class="text-sm font-medium text-foreground">Dub audio</span>
                    </div>
                    <Switch checked={pageState.isDub} onCheckedChange={() => pageState.toggleDub()} />
                </div>
            {/if}

            {#if appConfig.data?.player}
                <SettingsRow icon={Cpu} label="Video" onclick={() => activeSection = "video"} />
            {/if}

            {#if appConfig.data?.subtitles}
                <SettingsRow icon={Palette} label="Subtitle style" onclick={() => activeSection = "subtitleStyle"} />
            {/if}

            {#if (pageState.extensionItems.length > 0 || pageState.serverItems.length > 0 || pageState.supportsDub) && (pageState.videoTracks.length > 1 || pageState.audioTracks.length > 0 || pageState.subtitleTracks.length > 0)}
                <div class="h-px bg-border my-1 mx-2"></div>
            {/if}

            {#if pageState.videoTracks.length > 1}
                <SettingsRow icon={Gauge} label="Quality" value={currentQuality} onclick={() => activeSection = "quality"} />
            {/if}

            {#if pageState.audioTracks.length > 1}
                <SettingsRow icon={AudioLines} label="Audio track" value={currentAudio} onclick={() => activeSection = "audio"} />
            {/if}

            {#if pageState.subtitleTracks.length > 0}
                <SettingsRow icon={Captions} label="Subtitles" value={currentSub} onclick={() => activeSection = "subtitles"} />
            {/if}
        </div>

    {:else}
        <!-- Submenu View -->
        <div in:fly={{ x: 8, duration: 150 }} class="flex flex-col py-0.5">
            <button
                    onclick={goBack}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-muted-foreground" />
                <span class="text-sm font-semibold capitalize">{activeSection}</span>
            </button>

            <div class="flex flex-col {isMobile ? '' : 'max-h-64 overflow-y-auto'}">
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
                    <SettingsRow icon={Cpu} label="Hardware decoding" value={currentHwdec} onclick={() => activeOption = "hwdec"} />
                    <SettingsRow icon={Gauge} label="Upscale quality" value={currentScale} onclick={() => activeOption = "scaleAlgorithm"} />

                    <div class="flex items-center justify-between px-3 py-2.5">
                        <span class="text-sm font-medium text-foreground">Smooth motion</span>
                        <Switch
                                checked={appConfig.data.player.interpolation}
                                onCheckedChange={(v) => appConfig.update({ player: { interpolation: v } })}
                        />
                    </div>

                    <div class="flex items-center justify-between px-3 py-2.5">
                        <span class="text-sm font-medium text-foreground">Reduce banding</span>
                        <Switch
                                checked={appConfig.data.player.deband}
                                onCheckedChange={(v) => appConfig.update({ player: { deband: v } })}
                        />
                    </div>

                {:else if activeSection === "subtitleStyle" && appConfig.data}
                    <div class="flex flex-col {isMobile ? '' : 'max-h-72 overflow-y-auto'}">
                        <SettingsRow icon={Type} label="Font" value={currentFont} onclick={() => activeOption = "font"} />
                        <SettingsRow icon={Type} label="Font size" value={currentFontSize} onclick={() => activeOption = "fontSize"} />

                        <div class="px-3">
                            <label class="flex flex-col gap-1 py-1.5">
                                <span class="text-xs text-muted-foreground">Size scale ({appConfig.data.subtitles.scale.toFixed(2)}x)</span>
                                <input type="range" min="0.5" max="2" step="0.05"
                                       value={appConfig.data.subtitles.scale}
                                       oninput={(e) => appConfig.update({ subtitles: { scale: Number(e.currentTarget.value) } })}
                                />
                            </label>
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Text color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.color}
                                   oninput={(e) => appConfig.update({ subtitles: { color: e.currentTarget.value } })}
                            />
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Outline color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.borderColor}
                                   oninput={(e) => appConfig.update({ subtitles: { borderColor: e.currentTarget.value } })}
                            />
                        </div>

                        <div class="px-3">
                            <label class="flex flex-col gap-1 py-1.5">
                                <span class="text-xs text-muted-foreground">Outline size ({appConfig.data.subtitles.borderSize})</span>
                                <input type="range" min="0" max="6" step="0.5"
                                       value={appConfig.data.subtitles.borderSize}
                                       oninput={(e) => appConfig.update({ subtitles: { borderSize: Number(e.currentTarget.value) } })}
                                />
                            </label>
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Background box</span>
                            <Switch
                                    checked={appConfig.data.subtitles.backgroundColor !== null}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { backgroundColor: v ? "#000000AA" : null } })}
                            />
                        </div>
                        {#if appConfig.data.subtitles.backgroundColor !== null}
                            <div class="flex items-center justify-between px-3 py-1.5">
                                <span class="text-sm text-foreground">Background color</span>
                                <input type="color" class="w-8 h-8 rounded bg-transparent"
                                       value={appConfig.data.subtitles.backgroundColor.slice(0, 7)}
                                       oninput={(e) => appConfig.update({ subtitles: { backgroundColor: e.currentTarget.value + "AA" } })}
                                />
                            </div>
                        {/if}

                        <div class="px-3">
                            <label class="flex flex-col gap-1 py-1.5">
                                <span class="text-xs text-muted-foreground">Position ({appConfig.data.subtitles.position}%)</span>
                                <input type="range" min="0" max="100" step="1"
                                       value={appConfig.data.subtitles.position}
                                       oninput={(e) => appConfig.update({ subtitles: { position: Number(e.currentTarget.value) } })}
                                />
                            </label>
                        </div>

                        <SettingsRow icon={AlignCenter} label="Alignment" value={currentAlignment} onclick={() => activeOption = "alignment"} />

                        <div class="px-3">
                            <label class="flex flex-col gap-1 py-1.5">
                                <span class="text-xs text-muted-foreground">Delay ({appConfig.data.subtitles.delay.toFixed(1)}s)</span>
                                <input type="range" min="-10" max="10" step="0.1"
                                       value={appConfig.data.subtitles.delay}
                                       oninput={(e) => appConfig.update({ subtitles: { delay: Number(e.currentTarget.value) } })}
                                />
                            </label>
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Shadow color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.shadowColor.slice(0, 7)}
                                   oninput={(e) => appConfig.update({ subtitles: { shadowColor: e.currentTarget.value + "FF" } })}
                            />
                        </div>
                        <div class="px-3">
                            <label class="flex flex-col gap-1 py-1.5">
                                <span class="text-xs text-muted-foreground">Shadow offset ({appConfig.data.subtitles.shadowOffset})</span>
                                <input type="range" min="0" max="6" step="0.5"
                                       value={appConfig.data.subtitles.shadowOffset}
                                       oninput={(e) => appConfig.update({ subtitles: { shadowOffset: Number(e.currentTarget.value) } })}
                                />
                            </label>
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Force my style on styled subs</span>
                            <Switch
                                    checked={appConfig.data.subtitles.forceStyle}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { forceStyle: v } })}
                            />
                        </div>

                        <div class="flex items-center justify-between px-3 py-1.5">
                            <span class="text-sm text-foreground">Hide sound descriptions</span>
                            <Switch
                                    checked={appConfig.data.subtitles.sdhFilter}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilter: v, sdhFilterHarder: v ? appConfig.data!.subtitles.sdhFilterHarder : false } })}
                            />
                        </div>
                        {#if appConfig.data.subtitles.sdhFilter}
                            <div class="flex items-center justify-between px-3 py-1.5 pl-6">
                                <span class="text-xs text-muted-foreground">More aggressive</span>
                                <Switch
                                        checked={appConfig.data.subtitles.sdhFilterHarder}
                                        onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilterHarder: v } })}
                                />
                            </div>
                        {/if}
                    </div>

                {:else if activeSection === "quality"}
                    <SettingsOptionList
                            options={pageState.videoTracks.map(t => ({ id: String(t.id), label: qualityLabel(t) }))}
                            activeId={activeVideoId}
                            onSelect={(id) => selectOption(() => pageState.setVideoTrack(Number(id)))}
                    />

                {:else if activeSection === "subtitles"}
                    <SettingsOptionList
                            options={[{ id: "off", label: "Off" }, ...pageState.subtitleTracks.map(t => ({ id: String(t.id), label: subtitleLabels.get(t.id) ?? baseTrackLabel(t) }))]}
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
            class="absolute bottom-full right-0 mb-8 w-72 bg-popover border border-border rounded-sm shadow-2xl text-sm overflow-hidden flex flex-col p-1.5 z-50 text-popover-foreground"
    >
        {@render menuContent()}
    </div>
{:else}
    <div class="flex flex-col text-sm text-foreground w-full">
        {@render menuContent()}
    </div>
{/if}