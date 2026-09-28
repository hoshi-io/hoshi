import { page } from "$app/state";
import { untrack } from "svelte";

import { contentApi } from "@/api/content/content";
import { extensionsApi } from "@/api/extensions/extensions";
import { extensions as extensionsStore } from "@/stores/extensions.svelte.js";
import { type CoreError } from "@/api/client";
import { progressApi } from "@/api/progress/progress";
import { listApi } from "@/api/list/list";
import { listStore } from "@/app/list.svelte.js";
import { appConfig } from "@/stores/config.svelte.js";
import { i18n } from "@/stores/i18n.svelte.js";
import {type Metadata, primaryMetadata} from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { goto } from "$app/navigation";

import type { Extension } from "@/api/extensions/types";
import type { PlayerConfig, SubtitleConfig } from "@/api/config/types.js";
import { type as getOsType } from "@tauri-apps/plugin-os";

export interface EpisodeChapter {
    start: number;
    end: number;
    title: string;
}

interface TorrentStreamInfo { sessionId: string; url: string; totalSize: number }

interface ResolvedStream {
    url: string;
    headers: Record<string, string>;
    subtitles: any[];
    chapters: EpisodeChapter[];
    torrentSessionId?: string;
    torrent?: any;
}

export interface PlaybackTrack {
    id: number;
    track_type: "audio" | "video" | "sub" | string;
    selected: boolean;
    title?: string | null;
    lang?: string | null;
    codec?: string | null;
    bitrate?: number | null;
    w?: number | null;
    h?: number | null;
}

interface NowPlaying {
    cid: string;
    episode: number;
    title: string;
    episodeTitle: string;
    coverImage: string | null;
    nsfw: boolean;
    totalEpisodes: number;
}

export class WatchState {
    params      = $derived(page.params as Record<string, string>);
    cid         = $derived(this.params.cid || "");
    epNumber    = $derived(Number(this.params.number));

    isLoadingMeta   = $state(true);
    animeData       = $state<FullContent | null>(null);
    episodeTitle    = $state("");

    animeTitle = $derived.by(() => {
        if (!this.animeData) return "";
        const meta = primaryMetadata(this.animeData, appConfig.data?.content?.preferredMetadataProvider);
        const pref = appConfig.data?.ui?.titleLanguage || "romaji";
        return meta?.titleI18n?.[pref] || meta?.title || "";
    });

    totalEpisodes = $derived.by(() => {
        if (!this.animeData) return 0;
        const meta = primaryMetadata(this.animeData);
        return meta?.epsOrChapters || 0;
    });

    hasNext = $derived(this.totalEpisodes > 0 && this.epNumber < this.totalEpisodes);
    hasPrev = $derived(this.epNumber > 1);

    extensions = $state<Extension[]>([]);
    selectedExtension = $state<string | null>(null);
    servers             = $state<string[]>([]);
    supportsDub         = $state(false);
    selectedServer      = $state<string | null>(null);
    isDub               = $state(false);

    extensionItems = $derived((() => {
        const nameCounts = new Map<string, number>();
        for (const ext of this.extensions) {
            nameCounts.set(ext.name, (nameCounts.get(ext.name) ?? 0) + 1);
        }

        return this.extensions.map(ext => {
            const isDuplicate = (nameCounts.get(ext.name) ?? 0) > 1;
            const label = isDuplicate && ext.source
                ? `${ext.name} (${ext.source})`
                : ext.name;

            return { value: ext.id, label };
        });
    })());

    serverItems     = $derived(this.servers.map(srv => ({ value: srv, label: srv })));

    /// True while a torrent extension is the selected source.
    isTorrent       = $derived(extensionsStore.torrent.some(e => e.id === this.selectedExtension));

    isLoadingPlay   = $state(false);
    error           = $state<CoreError | null>(null);
    isStreamLoaded  = $state(false);
    subtitles       = $state<SubtitleSource[]>([]);
    chapters        = $state<EpisodeChapter[]>([]);
    initialTime     = $state(0);
    currentDuration = $state(0);

    // ---- live playback control state, driven by player controls + events -
    isPaused    = $state(false);
    currentTime = $state(0);
    volume      = $state(100);
    isMuted     = $state(false);
    tracks      = $state<PlaybackTrack[]>([]);

    bufferedTime = $state(0);
    bufferedFraction = $derived(
        this.currentDuration > 0 ? Math.min(this.bufferedTime / this.currentDuration, 1) : 0
    );

    audioTracks = $derived(this.tracks.filter(t => t.track_type === "audio"));
    subtitleTracks = $derived(this.tracks.filter(t => t.track_type === "sub"));
    videoTracks = $derived(this.tracks.filter(t => t.track_type === "video"));

    isMappingError = $derived(!!this.error?.key?.includes("match"));

    private currentLoadedCid    = $state<string | null>(null);
    private currentLoadedEp     = $state<number | null>(null);
    private destroyed = false;

    // ---- progress/list sync driven by player://* events ----------------
    private lastSyncTime  = 0;
    private hasUpdatedList = false;
    private unlistenFns: UnlistenFn[] = [];

    // ---- config -> mpv property sync, each deduped independently --------
    private appliedLangPrefs: string | null = null;
    private appliedVideoKey: string | null = null;
    private appliedSubStyleKey: string | null = null;

    torrentSessionId = $state<string | null>(null);
    selectedTorrent = $state<any | null>(null);
    private forcedTorrent: any | null = null;
    private loadToken = 0;

    torrentTitle = $state<string | null>(null);

    constructor() {
        invoke("lock_orientation", { orientation: "landscape" }).catch(() => {});
        invoke("enter_fullscreen").catch(() => {});

        // Android has no initialize_player command — the mpv core boots
        // itself off the first native surfaceCreated callback (see
        // player_surface::android on the Rust side), not off a frontend call.
        if (getOsType() !== "android") {
            invoke("initialize_player").catch(() => {});
        }

        // Episode/content loading — reacts only to route params.
        $effect(() => {
            const { cid, epNumber } = this;
            if (cid && epNumber && (cid !== this.currentLoadedCid || epNumber !== this.currentLoadedEp)) {
                untrack(() => this.loadPageData(cid, epNumber));
            }
        });

        // Video pipeline + language preference sync — reacts only to
        // `appConfig.data.player` changing, not to playback progress.
        $effect(() => {
            const player = appConfig.data?.player;
            if (!player) return;
            untrack(() => this.applyVideoOptions(player));
            untrack(() => this.applyLangPreferences());
        });

        // Subtitle style sync — reacts only to `appConfig.data.subtitles`
        // changing, kept separate so a video-setting tweak doesn't
        // redundantly resend subtitle properties and vice versa.
        $effect(() => {
            const subs = appConfig.data?.subtitles;
            if (!subs) return;
            untrack(() => this.applySubtitleStyle(subs));
        });

        // Intro/outro auto-skip — the one effect that's *supposed* to run
        // on every position tick, since it needs the current time.
        $effect(() => {
            const skippable = this.currentSkippable;
            const player = appConfig.data?.player;

            if (skippable && player) {
                const shouldAutoSkip =
                    (skippable.type === "intro" && player.autoSkipIntro) ||
                    (skippable.type === "outro" && player.autoSkipOutro);

                if (shouldAutoSkip) {
                    untrack(() => this.executeSkip(skippable.chapter));
                }
            }
        });

        untrack(() => this.attachPlayerListeners());
    }

    private async attachPlayerListeners() {
        this.unlistenFns.push(
            await listen<number>("player://position", (e) => this.handlePlayerProgress(e.payload)),
            await listen<number>("player://buffered", (e) => { this.bufferedTime = e.payload; }),
            await listen<void>("player://eof", () => this.handleEof()),
            await listen<boolean>("player://pause-changed", (e) => { this.isPaused = e.payload; }),
            await listen<string>("player://error", (e) => this.handlePlayerError(e.payload)),
        );
    }

    private handlePlayerError(raw: string) {
        console.error("Player error", raw);
        this.error = { key: "watch.stream_load_failed", message: raw };
        this.isLoadingPlay = false;
        this.isStreamLoaded = false;
    }

    // ---- transport controls ---------------------------------------------

    async togglePlay() {
        try {
            this.isPaused = await invoke<boolean>("toggle_pause");
        } catch (e) {
            console.error("Failed to toggle pause", e);
        }
    }

    async setVolume(volume: number) {
        this.volume = volume;
        try {
            await invoke("set_volume", { volume });
            if (this.isMuted && volume > 0) {
                this.isMuted = false;
                await invoke("set_muted", { muted: false });
            }
        } catch (e) {
            console.error("Failed to set volume", e);
        }
    }

    async toggleMute() {
        this.isMuted = !this.isMuted;
        try {
            await invoke("set_muted", { muted: this.isMuted });
        } catch (e) {
            console.error("Failed to set muted", e);
        }
    }

    async seek(target: number, relative = false) {
        try {
            await invoke("seek", { target, relative });
        } catch (e) {
            console.error("Failed to seek", e);
        }
    }

    async seekRelative(deltaSeconds: number) {
        await this.seek(deltaSeconds, true);
    }

    private async refreshTracks() {
        const token = this.loadToken;
        try {
            const tracks = await invoke<PlaybackTrack[]>("get_tracks");
            if (token === this.loadToken) this.tracks = tracks;
        } catch (e) {
            console.error("Failed to fetch tracks", e);
            if (token === this.loadToken) this.tracks = [];
        }
    }

    /// Chapters embedded in the file (torrent releases, or any source that
    /// supplied none) only exist inside mpv, so read them back from it.
    /// Assumes a `get_chapters` command wrapping `PlaybackHandle::chapters()`.
    private async refreshEmbeddedChapters() {
        if (this.chapters.length > 0) return;
        const token = this.loadToken;
        try {
            const list = await invoke<{ index: number; title: string | null; time: number }[]>("get_chapters");
            if (token !== this.loadToken || list.length === 0) return;

            const duration = this.currentDuration > 0
                ? this.currentDuration
                : await invoke<number>("get_duration").catch(() => 0);
            if (token !== this.loadToken) return;

            this.chapters = list.map((c, i) => ({
                start: c.time,
                end: list[i + 1]?.time ?? Math.max(duration, c.time),
                title: c.title ?? "",
            }));
        } catch (e) {
            console.error("Failed to fetch embedded chapters", e);
        }
    }

    async setAudioTrack(id: number | null) {
        try {
            await invoke("set_audio_track", { id });
        } catch (e) {
            console.error("Failed to set audio track", e);
        }
        await this.refreshTracks();
    }

    async setVideoTrack(id: number) {
        try {
            await invoke("set_video_track", { id });
        } catch (e) {
            console.error("Failed to set video track", e);
        }
        await this.refreshTracks();
    }

    async setSubtitleTrack(id: number | null) {
        try {
            await invoke("set_subtitle_track", { id });
        } catch (e) {
            console.error("Failed to set subtitle track", e);
        }
        await this.refreshTracks();
    }

    /// Switches server within the current extension — triggers a fresh
    /// `loadPlay()` since a different server means a different stream.
    async selectServer(server: string) {
        if (server === this.selectedServer) return;
        this.selectedServer = server;
        await this.loadPlay();
    }

    async toggleDub() {
        if (!this.supportsDub) return;
        this.isDub = !this.isDub;
        await this.loadPlay();
    }

    private durationFetchInFlight = false;

    private tracksFetchedThisLoad = false;

    private handlePlayerProgress(currentTime: number) {
        // While a new stream is still resolving/opening, mpv can keep
        // reporting ticks from the previous file. Using them would grab the
        // old file's duration/tracks and save its progress under the new
        // episode, so ignore everything until load_stream has returned.
        if (!this.isStreamLoaded) return;

        this.currentTime = currentTime;

        // The first tick means mpv has opened the file, so its tracks and
        // chapters are known. Don't tie this to duration: it can stay
        // unavailable for a while on streamed files.
        if (!this.tracksFetchedThisLoad) {
            this.tracksFetchedThisLoad = true;
            void this.refreshTracks().then(() => this.refreshEmbeddedChapters());
        }

        // duration is unavailable (mpv error -10, PROPERTY_UNAVAILABLE)
        // until mpv has actually opened and started decoding the file
        if (this.currentDuration <= 0 && !this.durationFetchInFlight) {
            this.durationFetchInFlight = true;
            const token = this.loadToken;
            invoke<number>("get_duration")
                .then((d) => { if (token === this.loadToken) this.currentDuration = d; })
                .catch(() => {})
                .finally(() => { this.durationFetchInFlight = false; });
        }

        if (!appConfig.data) return;
        const duration = this.currentDuration;

        if (Math.abs(currentTime - this.lastSyncTime) >= 10 || (this.lastSyncTime === 0 && currentTime > 2)) {
            this.lastSyncTime = currentTime;
            progressApi.updateAnimeProgress({
                cid: this.cid,
                episode: this.epNumber,
                timestampSeconds: Math.floor(currentTime),
                episodeDurationSeconds: duration > 0 ? Math.floor(duration) : undefined,
                completed: duration > 0 && currentTime / duration >= 0.9,
            }).catch(() => {});
        }

        if (!this.hasUpdatedList && duration > 0 && appConfig.data.content.autoUpdateProgress) {
            if (currentTime / duration >= 0.8) {
                this.hasUpdatedList = true;
                const status =
                    this.totalEpisodes > 0 && this.epNumber >= this.totalEpisodes
                        ? "COMPLETED"
                        : "CURRENT";
                listApi.upsert({ cid: this.cid, status, progress: this.epNumber }).catch(() => {});
                listStore.updateEntryProgressLocal(this.cid, this.epNumber, status);
            }
        }
    }

    private handleEof() {
        if (this.hasNext && appConfig.data?.player.autoplayNextEpisode) {
            this.goToEpisode(this.epNumber + 1);
        }
    }

    private async applyLangPreferences() {
        const player = appConfig.data?.player;
        if (!player) return;
        const key = `${player.preferredSubLang}|${player.preferredDubLang}`;
        if (key === this.appliedLangPrefs) return;
        this.appliedLangPrefs = key;

        try {
            await invoke("set_lang_preferences", {
                subLang: player.preferredSubLang || undefined,
                dubLang: player.preferredDubLang || undefined,
            });
        } catch (e) {
            console.error("Failed to set lang preferences", e);
        }
    }

    private async applyVideoOptions(player: PlayerConfig) {
        const key = JSON.stringify([
            player.hwdec, player.scaleAlgorithm, player.interpolation, player.deband,
            player.anime4kMode, player.anime4kTier,
        ]);
        if (key === this.appliedVideoKey) return;
        this.appliedVideoKey = key;

        try {
            await invoke("set_player_options", {
                options: [
                    ["hwdec", player.hwdec],
                    ["scale", player.scaleAlgorithm],
                    ["cscale", player.scaleAlgorithm],
                    ["interpolation", player.interpolation ? "yes" : "no"],
                    ["deband", player.deband ? "yes" : "no"],
                    ["anime4kMode", player.anime4kMode],
                    ["anime4kTier", player.anime4kTier],
                ],
            });
        } catch (e) {
            console.error("Failed to apply video options", e);
        }
    }

    private async applySubtitleStyle(subs: SubtitleConfig) {
        const key = JSON.stringify(subs);
        if (key === this.appliedSubStyleKey) return;
        this.appliedSubStyleKey = key;

        try {
            await invoke("set_player_options", {
                options: [
                    ["sub-font", subs.font],
                    ["sub-font-size", String(subs.fontSize)],
                    ["sub-color", subs.color],
                    ["sub-border-color", subs.borderColor],
                    ["sub-border-size", String(subs.borderSize)],
                    ["sub-back-color", subs.backgroundColor ?? "#00000000"],
                    ["sub-shadow-color", subs.shadowColor],
                    ["sub-shadow-offset", String(subs.shadowOffset)],
                    ["sub-pos", String(subs.position)],
                    ["sub-scale", String(subs.scale)],
                    ["sub-justify", subs.justify],
                    ["sub-delay", String(subs.delay)],
                    ["sub-ass-override", subs.forceStyle ? "force" : "no"],
                    ["sub-filter-sdh", subs.sdhFilter ? "yes" : "no"],
                    ["sub-filter-sdh-harder", subs.sdhFilterHarder ? "yes" : "no"],
                ],
            });
        } catch (e) {
            console.error("Failed to apply subtitle style", e);
        }
    }

    goToEpisode(episode: number) {
        if (!Number.isFinite(episode) || episode < 1) return;
        if (this.totalEpisodes > 0 && episode > this.totalEpisodes) return;
        if (episode === this.epNumber) return;

        const segments = page.url.pathname.split("/");
        segments[segments.length - 1] = String(episode);
        goto(segments.join("/"));
    }

    async loadPageData(targetCid: string, targetEp: number) {
        try {
            // forced choice only applies to the episode/extension it was made for
// (add this line at the top of loadPageData() and selectExtension())
            this.forcedTorrent = null;
            this.currentLoadedEp = targetEp;

            if (!extensionsStore.initialized) {
                await extensionsStore.load();
            }

            if (targetCid !== this.currentLoadedCid) {
                this.isLoadingMeta = true;
                const contentRes = await contentApi.get_by_cid(targetCid);
                this.animeData = contentRes;
                this.updateEpisodeTitle(targetEp);

                const globalExtensions = [
                    ...extensionsStore.anime,
                    ...extensionsStore.torrent
                ];
                const contentExtensions = contentRes.extensionSources?.map((e: any) => e.extensionName) || [];
                this.extensions = globalExtensions;
                this.currentLoadedCid = targetCid;

                if (this.extensions.length > 0) {
                    const initialExt =
                        this.extensions.find(e => contentExtensions.includes(e.id)) ??
                        this.extensions[0];
                    await this.selectExtension(initialExt.id);
                }

                this.isLoadingMeta = false;
            } else {
                this.currentLoadedEp = targetEp;
                this.updateEpisodeTitle(targetEp);
                await this.loadPlay();
            }

        } catch (e: any) {
            console.error("Error in loadPageData:", e);
            this.error = e.key ? e : { key: "errors.unknown_error", message: e.message };
            this.isLoadingMeta = false;
        }
    }

    private updateEpisodeTitle(ep: number) {
        const unit = this.animeData?.contentUnits?.find((u: any) => u.unitNumber === ep);

        const isGenericTitle = unit?.title?.trim().toLowerCase() === `episode ${ep}`;

        this.episodeTitle = (unit?.title && !isGenericTitle)
            ? i18n.t("watch.episode_with_title", { num: ep, title: unit.title })
            : i18n.t("watch.episode_number", { num: ep });
    }

    async selectExtension(ext: string) {
        // forced choice only applies to the episode/extension it was made for
// (add this line at the top of loadPageData() and selectExtension())
        this.forcedTorrent = null;
        this.selectedExtension = ext;
        this.servers = [];
        this.supportsDub = false;
        this.selectedServer = null;
        this.isDub = false;

        const isSora = extensionsStore.anime.find(e => e.id === ext)?.source === 'sora';
        const isTorrent = extensionsStore.torrent.some(e => e.id === this.selectedExtension);

        if (!isSora && !isTorrent) {
            try {
                const s = await extensionsApi.getSettings(ext);
                // a newer selection (e.g. a torrent extension) took over
                // while this request was in flight; don't apply stale servers
                if (this.selectedExtension !== ext) return;
                this.servers = s.episodeServers ?? [];
                this.supportsDub = s.supportsDub ?? false;
                this.selectedServer = this.servers[0] ?? null;
            } catch {}
        }

        await this.loadPlay();
    }

    private async resolveAnimeStream(): Promise<ResolvedStream> {
        const isSora = extensionsStore.anime.find(e => e.id === this.selectedExtension)?.source === 'sora';

        if (isSora) {
            const res = await contentApi.listEpisodeServers(this.selectedExtension!, this.cid, this.epNumber);
            this.servers = res.servers ?? [];
            this.supportsDub = false;
            this.isDub = false;
            if (!this.selectedServer || !this.servers.includes(this.selectedServer)) {
                this.selectedServer = this.servers[0] ?? null;
            }
        }

        const opts: { server?: string; category?: string } = {};
        if (this.selectedServer) opts.server = this.selectedServer;
        if (this.supportsDub && this.isDub) opts.category = "dub";

        const res = await contentApi.play(this.cid, this.selectedExtension!, this.epNumber, opts);
        if (res.type?.toLowerCase() !== "video") throw { key: "watch.no_stream" } as CoreError;

        const data = res.data as any;
        return {
            url: data.source.url,
            headers: data.headers ?? {},
            subtitles: (data.source.subtitles ?? []).map((s: any) => {
                const { lang, variant } = parseSubtitleLabel(s.language ?? "");
                return { url: s.url, title: s.language, lang, variant, isDefault: !!s.is_default };
            }),
            chapters: data.source.chapters ?? [],
        };
    }

    private async resolveTorrentStream(token: number): Promise<ResolvedStream | null> {
        let torrent = this.forcedTorrent;

        if (!torrent) {
            const query = buildTorrentQuery(this.animeData, this.epNumber);
            if (!query) throw { key: "watch.no_metadata_for_query" } as CoreError;

            const picked = await invoke<{ torrent: any | null }>("auto_select_torrent", {
                id: this.selectedExtension, query, filters: {}, page: 1,
            });
            if (!picked.torrent) throw { key: "watch.no_torrent_match" } as CoreError;
            torrent = picked.torrent;
        }

        const info = await invoke<TorrentStreamInfo>("start_torrent_stream", {
            id: this.selectedExtension,
            contentId: torrent.id,
            magnet: torrent.magnet ?? null,
        });

        if (token !== this.loadToken) {
            void this.stopTorrentSession(info.sessionId);
            return null;
        }

        return { url: info.url, headers: {}, subtitles: [], chapters: [],
            torrentSessionId: info.sessionId, torrent };
    }

    private async stopTorrentSession(sessionId: string) {
        await invoke("stop_torrent_stream", { sessionId }).catch(() => {});
    }

    /// Core flow: get the resume position, ask the extension for a source,
    /// then hand url + headers + subtitles + chapters + start position to
    /// mpv in one `load_stream` call.
    async loadPlay() {
        if (!this.selectedExtension) return;

        this.isLoadingPlay = true;
        this.isStreamLoaded = false;
        this.error = null;
        this.subtitles = [];
        this.chapters = [];
        this.currentDuration = 0;
        this.durationFetchInFlight = false;
        this.tracksFetchedThisLoad = false;
        this.currentTime = 0;
        this.isPaused = false;
        this.tracks = [];
        this.bufferedTime = 0;
        this.lastSyncTime = 0;
        this.hasUpdatedList = false;
        this.skippedChapterStarts = new Set();
        const token = ++this.loadToken;
        const previousSession = this.torrentSessionId;
        this.torrentTitle = null;
        this.selectedTorrent = null;

        try {
            let initialTime = 0;
            if (appConfig.data?.player.resumeFromLastPos) {
                try {
                    const res = await progressApi.getContentProgress(this.cid);
                    const prog = res.animeProgress.find((p: any) => p.episode === this.epNumber);
                    initialTime = prog?.timestampSeconds ?? 0;
                } catch {
                    initialTime = 0;
                }
            }
            this.initialTime = initialTime;

            const isTorrent = extensionsStore.torrent.some(e => e.id === this.selectedExtension);
            const stream = isTorrent
                ? await this.resolveTorrentStream(token)
                : await this.resolveAnimeStream();

            // superseded by a newer loadPlay while we were resolving
            if (!stream || token !== this.loadToken) return;

            this.torrentSessionId = stream.torrentSessionId ?? null;
            this.torrentTitle = stream.torrentTitle ?? null;
            this.selectedTorrent = stream.torrent ?? null;

            this.subtitles = stream.subtitles;
            this.chapters = stream.chapters;

            const meta = primaryMetadata(this.animeData, appConfig.data?.content?.preferredMetadataProvider);
            const nowPlaying: NowPlaying = {
                cid: this.cid,
                episode: this.epNumber,
                title: this.animeTitle,
                episodeTitle: this.episodeTitle,
                coverImage: meta?.coverImage ?? null,
                nsfw: this.animeData?.content?.nsfw ?? false,
                totalEpisodes: this.totalEpisodes,
            };

            await invoke("wait_for_player_ready").catch(() => {});
            await invoke("load_stream", {
                url: stream.url,
                headers: this.toHeaderList(stream.headers),
                subtitles: this.subtitles,
                chapters: this.chapters,
                startPosition: initialTime > 0 ? initialTime : undefined,
                nowPlaying,
            });

            // mpv now owns the new stream and has closed the old one
            if (previousSession && previousSession !== this.torrentSessionId) {
                void this.stopTorrentSession(previousSession);
            }

            this.isStreamLoaded = true;
            await this.refreshTracks();

            const preferredId = this.pickPreferredSubtitleTrack();
            if (preferredId !== null) await this.setSubtitleTrack(preferredId);
        } catch (e: any) {
            console.log(e);
            this.error = e.key ? e : { key: "errors.unknown_error" };
        } finally {
            // don't clobber the spinner of a newer load
            if (token === this.loadToken) this.isLoadingPlay = false;
        }
    }

    async chooseTorrent(torrent: any) {
        this.forcedTorrent = torrent;
        await this.loadPlay();
    }

// copy the exact buildTorrentQuery call from resolveTorrentStream
    torrentSearchQuery(): string {
        return buildTorrentQuery(this.animeData, this.epNumber, false);
    }

    private skippedChapterStarts = new Set<number>();

    currentSkippable = $derived.by(() => {
        const time = this.currentTime;
        for (const chapter of this.chapters) {
            if (time < chapter.start || time >= chapter.end) continue;
            if (this.skippedChapterStarts.has(chapter.start)) continue;

            const label = chapter.title.toLowerCase();
            const isIntro = /intro|opening|^op\b/.test(label);
            const isOutro = /outro|ending|^ed\b/.test(label);

            if (isIntro) return { chapter, type: "intro" as const };
            if (isOutro) return { chapter, type: "outro" as const };
        }
        return null;
    });

    manualSkipChapter = $derived.by(() => {
        const skippable = this.currentSkippable;
        const player = appConfig.data?.player;
        if (!skippable || !player) return null;

        if (skippable.type === "intro" && !player.autoSkipIntro) return skippable.chapter;
        if (skippable.type === "outro" && !player.autoSkipOutro) return skippable.chapter;

        return null;
    });

    // 3. Shared action: handles the actual skipping
    executeSkip(chapter: EpisodeChapter) {
        this.skippedChapterStarts.add(chapter.start);
        this.seek(chapter.end, false);
    }

    private toHeaderList(headers: Record<string, string>): { key: string; value: string }[] {
        return Object.entries(headers)
            .filter(([, value]) => !!value)
            .map(([key, value]) => ({ key, value }));
    }

    destroy() {
        if (this.destroyed) return;
        this.destroyed = true;
        this.loadToken++;

        for (const unlisten of this.unlistenFns) unlisten();
        this.unlistenFns = [];

        const sessionId = this.torrentSessionId;
        this.torrentSessionId = null;

        invoke("unlock_orientation").catch(() => {});
        invoke("exit_fullscreen").catch(() => {});
        invoke("clear_activity").catch(() => {});
        invoke("stop_playback")
            .catch(() => {})
            .finally(() => { if (sessionId) void this.stopTorrentSession(sessionId); });
    }

    private pickPreferredSubtitleTrack(): number | null {
        const prefs = (appConfig.data?.player.preferredSubLang ?? "")
            .split(",").map(s => s.trim().toLowerCase()).filter(Boolean);

        const findTrackId = (source: SubtitleSource) =>
            this.subtitleTracks.find(t => t.title === source.title)?.id ?? null;

        for (const pref of prefs) {
            const match = this.subtitles.find(s => s.lang === pref && !s.variant);
            if (match) {
                const id = findTrackId(match);
                if (id !== null) return id;
            }
        }
        const def = this.subtitles.find(s => s.isDefault);
        if (def) return findTrackId(def);

        return null;
    }

}

export interface SubtitleSource {
    url: string;
    title?: string;
    lang: string | null;
    variant: "forced" | "songs" | "sdh" | null;
    isDefault: boolean;
}

const LANGUAGE_NAME_TO_CODE: Record<string, string> = {
    english: "en",
    spanish: "es",
    "latin american spanish": "es-419",
    japanese: "ja",
    portuguese: "pt",
    "brazilian portuguese": "pt-br",
    french: "fr",
    german: "de",
    italian: "it",
};

function parseSubtitleLabel(raw: string): { lang: string | null; variant: SubtitleSource["variant"] } {
    const [namePart, ...rest] = raw.split(" - ");
    const modifier = rest.join(" - ").trim().toLowerCase();

    let variant: SubtitleSource["variant"] = null;
    if (modifier.includes("forced")) variant = "forced";
    else if (modifier.includes("song")) variant = "songs";
    else if (modifier.includes("sdh") || modifier.includes("hearing")) variant = "sdh";

    return { lang: LANGUAGE_NAME_TO_CODE[namePart.trim().toLowerCase()] ?? null, variant };
}

function getAnilistMetadata(data: FullContent | null): Metadata | null {
    if (!data) return null;
    return data.metadata.find(m => m.sourceName?.toLowerCase() === "anilist") ?? null;
}

function buildTorrentQuery(data: FullContent | null, epNumber: number, isBatch: boolean): string {
    const anilist = getAnilistMetadata(data);
    const title = anilist?.titleI18n?.romaji || anilist?.title || "";
    if (!title) return "";
    isBatch = false;
    return isBatch ? `${title} batch` : `${title} ${epNumber}`;
}