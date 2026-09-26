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
        ChevronRight,
        ChevronLeft,
        Check,
        Cpu,
        Palette
    } from "lucide-svelte";
    import { Switch } from "@/components/ui/switch";
    import type { WatchState } from "@/app/watch.svelte.js";
    import type { PlaybackTrack } from "@/app/watch.svelte.js";

    let { pageState, isMobile = false }: { pageState: WatchState, isMobile?: boolean } = $props();

    type SectionId = "source" | "server" | "audio" | "subtitles" | "quality" | "video" | "subtitleStyle";
    let activeSection = $state<SectionId | null>(null);

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

    function selectOption(action: () => void) {
        action();
        activeSection = null;
    }
</script>

{#snippet menuContent()}
    {#if activeSection === null}
        <!-- Main Settings Menu -->
        <div
                in:fly={{ x: -8, duration: 150 }}
                class="flex flex-col py-0.5"
        >

            <!-- Source / Extension Row -->
            {#if pageState.extensionItems.length > 0}
                <button
                        onclick={() => activeSection = "source"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <PuzzleIcon class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Source</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-white/50 truncate max-w-[90px]">{currentSource}</span>
                        <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                    </div>
                </button>
            {/if}

            <!-- Server Row -->
            {#if pageState.serverItems.length > 0}
                <button
                        onclick={() => activeSection = "server"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Server class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Server</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-white/50 truncate max-w-[90px]">{currentServer}</span>
                        <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                    </div>
                </button>
            {/if}

            <!-- Dub Toggle Row -->
            {#if pageState.supportsDub}
                <div class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left">
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Mic2 class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Dub audio</span>
                    </div>
                    <Switch
                            checked={pageState.isDub}
                            onCheckedChange={() => pageState.toggleDub()}
                    />
                </div>
            {/if}

            {#if appConfig.data?.player}
                <button
                        onclick={() => activeSection = "video"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Cpu class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Video</span>
                    </div>
                    <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                </button>
            {/if}

            {#if appConfig.data?.subtitles}
                <!-- Subtitle Style Row -->
                <button
                        onclick={() => activeSection = "subtitleStyle"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Palette class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Subtitle style</span>
                    </div>
                    <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                </button>
            {/if}

            <!-- Divider -->
            {#if (pageState.extensionItems.length > 0 || pageState.serverItems.length > 0 || pageState.supportsDub) && (pageState.videoTracks.length > 1 || pageState.audioTracks.length > 0 || pageState.subtitleTracks.length > 0)}
                <div class="h-px bg-white/10 my-1 mx-2"></div>
            {/if}

            <!-- Quality Row -->
            {#if pageState.videoTracks.length > 1}
                <button
                        onclick={() => activeSection = "quality"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Gauge class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Quality</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-white/50 truncate max-w-[90px]">{currentQuality}</span>
                        <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                    </div>
                </button>
            {/if}

            <!-- Audio Track Row -->
            {#if pageState.audioTracks.length > 1}
                <button
                        onclick={() => activeSection = "audio"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <AudioLines class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Audio track</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-white/50 truncate max-w-[90px]">{currentAudio}</span>
                        <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                    </div>
                </button>
            {/if}

            <!-- Subtitles Row -->
            {#if pageState.subtitleTracks.length > 0}
                <button
                        onclick={() => activeSection = "subtitles"}
                        class="group flex items-center justify-between w-full px-3 py-2.5 rounded-sm hover:bg-white/10 transition text-left"
                >
                    <div class="flex items-center gap-3">
                        <div class="flex items-center justify-center w-7 h-7 rounded-sm bg-white/10 group-hover:bg-white/20 transition-colors">
                            <Captions class="w-4 h-4 text-white/80 group-hover:text-white" />
                        </div>
                        <span class="text-sm font-medium text-white/90">Subtitles</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-white/50 truncate max-w-[90px]">{currentSub}</span>
                        <ChevronRight class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
                    </div>
                </button>
            {/if}
        </div>
    {:else}
        <!-- Submenu View -->
        <div
                in:fly={{ x: 8, duration: 150 }}
                class="flex flex-col py-0.5"
        >
            <!-- Header -->
            <button
                    onclick={() => activeSection = null}
                    class="flex items-center gap-2.5 w-full px-3 py-2 mb-1 rounded-sm border-b border-white/10 text-white hover:bg-white/10 transition text-left"
            >
                <ChevronLeft class="w-4 h-4 text-white/60" />
                <span class="text-sm font-semibold capitalize">{activeSection}</span>
            </button>

            <!-- Option List -->
            <div class="flex flex-col {isMobile ? '' : 'max-h-64 overflow-y-auto'}">
                {#if activeSection === "source"}
                    {#each pageState.extensionItems as item}
                        {@const isActive = item.value === pageState.selectedExtension}
                        <button
                                onclick={() => selectOption(() => pageState.selectExtension(item.value))}
                                class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-primary" />
                                {/if}
                            </div>
                            <span class="flex-1 text-sm truncate">{item.label}</span>
                        </button>
                    {/each}

                {:else if activeSection === "server"}
                    {#each pageState.serverItems as item}
                        {@const isActive = item.value === pageState.selectedServer}
                        <button
                                onclick={() => selectOption(() => pageState.selectServer(item.value))}
                                class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-primary" />
                                {/if}
                            </div>
                            <span class="flex-1 text-sm truncate">{item.label}</span>
                        </button>
                    {/each}

                {:else if activeSection === "audio"}
                    {#each pageState.audioTracks as track}
                        {@const isActive = String(track.id) === activeAudioId}
                        <button
                                onclick={() => selectOption(() => pageState.setAudioTrack(track.id))}
                                class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-primary" />
                                {/if}
                            </div>
                            <span class="flex-1 text-sm truncate">{audioLabels.get(track.id) ?? baseTrackLabel(track)}</span>
                        </button>
                    {/each}
                {:else if activeSection === "video" && appConfig.data}
                    <div class="flex flex-col gap-1 px-3 py-2">
                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Hardware decoding</span>
                            <select
                                    class="bg-white/10 rounded-sm px-2 py-1.5 text-sm text-white/90 outline-none"
                                    value={appConfig.data.player.hwdec}
                                    onchange={(e) => appConfig.update({ player: { hwdec: e.currentTarget.value } })}
                            >
                                <option value="auto-safe">Auto (safe)</option>
                                <option value="auto">Auto</option>
                                <option value="no">Software only</option>
                            </select>
                        </label>

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Upscale quality</span>
                            <select
                                    class="bg-white/10 rounded-sm px-2 py-1.5 text-sm text-white/90 outline-none"
                                    value={appConfig.data.player.scaleAlgorithm}
                                    onchange={(e) => appConfig.update({ player: { scaleAlgorithm: e.currentTarget.value } })}
                            >
                                <option value="bilinear">Fast</option>
                                <option value="spline36">Balanced</option>
                                <option value="ewa_lanczossharp">Sharp</option>
                            </select>
                        </label>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Smooth motion</span>
                            <Switch
                                    checked={appConfig.data.player.interpolation}
                                    onCheckedChange={(v) => appConfig.update({ player: { interpolation: v } })}
                            />
                        </div>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Reduce banding</span>
                            <Switch
                                    checked={appConfig.data.player.deband}
                                    onCheckedChange={(v) => appConfig.update({ player: { deband: v } })}
                            />
                        </div>
                    </div>

                {:else if activeSection === "subtitleStyle" && appConfig.data?.subtitles}
                    <div class="flex flex-col gap-1 px-3 py-2 {isMobile ? '' : 'max-h-72 overflow-y-auto'}">
                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Font</span>
                            <input type="text"
                                   class="bg-white/10 rounded-sm px-2 py-1.5 text-sm text-white/90 outline-none"
                                   placeholder="sans-serif"
                                   value={appConfig.data.subtitles.font}
                                   onchange={(e) => appConfig.update({ subtitles: { font: e.currentTarget.value || "sans-serif" } })}
                            />
                        </label>
                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Font size</span>
                            <select
                                    class="bg-white/10 rounded-sm px-2 py-1.5 text-sm text-white/90 outline-none"
                                    value={appConfig.data.subtitles.fontSize}
                                    onchange={(e) => appConfig.update({ subtitles: { fontSize: Number(e.currentTarget.value) } })}
                            >
                                <option value="36">Small</option>
                                <option value="55">Medium</option>
                                <option value="70">Large</option>
                                <option value="85">Extra large</option>
                            </select>
                        </label>

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Size scale ({appConfig.data.subtitles.scale.toFixed(2)}x)</span>
                            <input type="range" min="0.5" max="2" step="0.05"
                                   value={appConfig.data.subtitles.scale}
                                   oninput={(e) => appConfig.update({ subtitles: { scale: Number(e.currentTarget.value) } })}
                            />
                        </label>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Text color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.color}
                                   oninput={(e) => appConfig.update({ subtitles: { color: e.currentTarget.value } })}
                            />
                        </div>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Outline color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.borderColor}
                                   oninput={(e) => appConfig.update({ subtitles: { borderColor: e.currentTarget.value } })}
                            />
                        </div>

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Outline size ({appConfig.data.subtitles.borderSize})</span>
                            <input type="range" min="0" max="6" step="0.5"
                                   value={appConfig.data.subtitles.borderSize}
                                   oninput={(e) => appConfig.update({ subtitles: { borderSize: Number(e.currentTarget.value) } })}
                            />
                        </label>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Background box</span>
                            <Switch
                                    checked={appConfig.data.subtitles.backgroundColor !== null}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { backgroundColor: v ? "#000000AA" : null } })}
                            />
                        </div>
                        {#if appConfig.data.subtitles.backgroundColor !== null}
                            <div class="flex items-center justify-between py-1.5">
                                <span class="text-sm text-white/90">Background color</span>
                                <input type="color" class="w-8 h-8 rounded bg-transparent"
                                       value={appConfig.data.subtitles.backgroundColor.slice(0, 7)}
                                       oninput={(e) => appConfig.update({ subtitles: { backgroundColor: e.currentTarget.value + "AA" } })}
                                />
                            </div>
                        {/if}

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Position ({appConfig.data.subtitles.position}%)</span>
                            <input type="range" min="0" max="100" step="1"
                                   value={appConfig.data.subtitles.position}
                                   oninput={(e) => appConfig.update({ subtitles: { position: Number(e.currentTarget.value) } })}
                            />
                        </label>

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Alignment</span>
                            <select
                                    class="bg-white/10 rounded-sm px-2 py-1.5 text-sm text-white/90 outline-none"
                                    value={appConfig.data.subtitles.justify}
                                    onchange={(e) => appConfig.update({ subtitles: { justify: e.currentTarget.value as any } })}
                            >
                                <option value="auto">Auto</option>
                                <option value="left">Left</option>
                                <option value="center">Center</option>
                                <option value="right">Right</option>
                            </select>
                        </label>

                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Delay ({appConfig.data.subtitles.delay.toFixed(1)}s)</span>
                            <input type="range" min="-10" max="10" step="0.1"
                                   value={appConfig.data.subtitles.delay}
                                   oninput={(e) => appConfig.update({ subtitles: { delay: Number(e.currentTarget.value) } })}
                            />
                        </label>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Shadow color</span>
                            <input type="color" class="w-8 h-8 rounded bg-transparent"
                                   value={appConfig.data.subtitles.shadowColor.slice(0, 7)}
                                   oninput={(e) => appConfig.update({ subtitles: { shadowColor: e.currentTarget.value + "FF" } })}
                            />
                        </div>
                        <label class="flex flex-col gap-1 py-1.5">
                            <span class="text-xs text-white/60">Shadow offset ({appConfig.data.subtitles.shadowOffset})</span>
                            <input type="range" min="0" max="6" step="0.5"
                                   value={appConfig.data.subtitles.shadowOffset}
                                   oninput={(e) => appConfig.update({ subtitles: { shadowOffset: Number(e.currentTarget.value) } })}
                            />
                        </label>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Force my style on styled subs</span>
                            <Switch
                                    checked={appConfig.data.subtitles.forceStyle}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { forceStyle: v } })}
                            />
                        </div>

                        <div class="flex items-center justify-between py-1.5">
                            <span class="text-sm text-white/90">Hide sound descriptions</span>
                            <Switch
                                    checked={appConfig.data.subtitles.sdhFilter}
                                    onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilter: v, sdhFilterHarder: v ? appConfig.data!.subtitles.sdhFilterHarder : false } })}
                            />
                        </div>
                        {#if appConfig.data.subtitles.sdhFilter}
                            <div class="flex items-center justify-between py-1.5 pl-3">
                                <span class="text-xs text-white/70">More aggressive</span>
                                <Switch
                                        checked={appConfig.data.subtitles.sdhFilterHarder}
                                        onCheckedChange={(v) => appConfig.update({ subtitles: { sdhFilterHarder: v } })}
                                />
                            </div>
                        {/if}
                    </div>

                {:else if activeSection === "quality"}
                    {#each pageState.videoTracks as track}
                        {@const isActive = String(track.id) === activeVideoId}
                        <button
                                onclick={() => selectOption(() => pageState.setVideoTrack(track.id))}
                                class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-primary" />
                                {/if}
                            </div>
                            <span class="flex-1 text-sm truncate">{qualityLabel(track)}</span>
                        </button>
                    {/each}

                {:else if activeSection === "subtitles"}
                    {@const isOff = activeSubId === "off"}
                    <button
                            onclick={() => selectOption(() => pageState.setSubtitleTrack(null))}
                            class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isOff ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                    >
                        <div class="flex items-center justify-center w-5 h-5 shrink-0">
                            {#if isOff}
                                <Check class="w-4 h-4 text-primary" />
                            {/if}
                        </div>
                        <span class="flex-1 text-sm truncate">Off</span>
                    </button>

                    {#each pageState.subtitleTracks as track}
                        {@const isActive = String(track.id) === activeSubId}
                        <button
                                onclick={() => selectOption(() => pageState.setSubtitleTrack(track.id))}
                                class="flex items-center gap-3 w-full px-3 py-2.5 rounded-sm text-left transition {isActive ? 'text-primary bg-white/5 font-medium' : 'text-white/80 hover:bg-white/10'}"
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-primary" />
                                {/if}
                            </div>
                            <span class="flex-1 text-sm truncate">{subtitleLabels.get(track.id) ?? baseTrackLabel(track)}</span>
                        </button>
                    {/each}
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
            class="absolute bottom-full right-0 mb-8 w-72 bg-neutral-900/95 border border-white/15 rounded-sm shadow-2xl backdrop-blur-xl text-sm overflow-hidden flex flex-col p-1.5 z-50 text-white"
    >
        {@render menuContent()}
    </div>
{:else}
    <div class="flex flex-col text-sm text-white w-full">
        {@render menuContent()}
    </div>
{/if}