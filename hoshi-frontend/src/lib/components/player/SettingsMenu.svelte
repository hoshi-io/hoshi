<script lang="ts">
    import { fly, fade } from "svelte/transition";
    import {
        PuzzleIcon,
        Server,
        Mic2,
        AudioLines,
        Captions,
        Gauge,
        ChevronRight,
        ChevronLeft,
        Check
    } from "lucide-svelte";
    import { Switch } from "@/components/ui/switch";
    import type { WatchState } from "@/app/watch.svelte.js";
    import type { PlaybackTrack } from "@/app/watch.svelte.js";

    let { pageState }: { pageState: WatchState } = $props();

    type SectionId = "source" | "server" | "audio" | "subtitles" | "quality";
    let activeSection = $state<SectionId | null>(null);

    // Language codes (mpv typically reports ISO 639-2, e.g. "jpn"/"eng", but
    // some extensions send ISO 639-1 like "ja"/"en") don't mean much to most
    // people at a glance, so resolve them to a display name when we can.
    // Falls back to the raw code if the runtime's Intl data doesn't
    // recognize it, rather than throwing.
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
            // Some engines just echo back an unrecognized subtag instead of
            // throwing — treat that as "couldn't resolve it" too.
            if (!name || name.toLowerCase() === normalized) return null;
            return name;
        } catch {
            return null;
        }
    }

    /// Some sources hand us the raw subtitle filename as "title" (e.g.
    /// "0_en.srt") rather than anything descriptive — that's worse than no
    /// title at all, so treat anything that looks like a subtitle filename
    /// as noise and fall through to the language name instead.
    function isFilenameLikeTitle(title: string): boolean {
        return /\.(srt|vtt|ass|ssa|sub|ttml)$/i.test(title.trim());
    }

    /// Best label for a single track, ignoring sibling tracks (duplicates
    /// are disambiguated separately in buildTrackLabels).
    function baseTrackLabel(track: PlaybackTrack): string {
        if (track.title && !isFilenameLikeTitle(track.title)) return track.title;
        if (track.lang) return languageName(track.lang) || track.lang;
        // Junk filename title beats nothing at all if there's no lang either.
        return track.title || `Track ${track.id}`;
    }

    /// Sources sometimes provide two tracks for the same language (e.g. a
    /// dialogue track and a separate songs/signs track) with nothing else to
    /// tell them apart, so number the repeats: "English", "English (2)", ...
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

    /// Label for a quality/resolution variant, e.g. "1080p · 6.0 Mbps".
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

    // Derived active selection labels for the main menu rows
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

<!-- Main Container elevated higher with mb-8 and rounded-sm edges -->
<div
        in:fly={{ y: 12, duration: 200 }}
        out:fade={{ duration: 150 }}
        class="absolute bottom-full right-0 mb-8 w-72 bg-neutral-900/95 border border-white/15 rounded-sm shadow-2xl backdrop-blur-xl text-sm overflow-hidden flex flex-col p-1.5 z-50 text-white"
>
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
            <div class="flex flex-col max-h-64 overflow-y-auto">
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
</div>