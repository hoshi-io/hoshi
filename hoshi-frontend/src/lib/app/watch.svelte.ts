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
import { primaryMetadata } from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { Extension } from "@/api/extensions/types";

export interface SubtitleSource {
    url: string;
    title?: string;
    lang?: string;
}

export interface EpisodeChapter {
    start: number;
    end: number;
    title: string;
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

    audioTracks = $derived(this.tracks.filter(t => t.track_type === "audio"));
    subtitleTracks = $derived(this.tracks.filter(t => t.track_type === "sub"));

    isMappingError = $derived(!!this.error?.key?.includes("match"));

    private currentLoadedCid    = $state<string | null>(null);
    private currentLoadedEp     = $state<number | null>(null);
    private destroyed = false;

    // ---- progress/list sync driven by player://* events ----------------
    private lastSyncTime  = 0;
    private hasUpdatedList = false;
    private unlistenFns: UnlistenFn[] = [];

    constructor() {
        invoke("lock_orientation", { orientation: "landscape" }).catch(() => {});

        invoke("initialize_player").catch(() => {});

        $effect(() => {
            const { cid, epNumber } = this;
            if (cid && epNumber && (cid !== this.currentLoadedCid || epNumber !== this.currentLoadedEp)) {
                untrack(() => this.loadPageData(cid, epNumber));
            }
        });

        untrack(() => this.attachPlayerListeners());
    }

    private async attachPlayerListeners() {
        this.unlistenFns.push(
            await listen<number>("player://position", (e) => this.handlePlayerProgress(e.payload)),
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

    /// Refetches the current track list from mpv. Called once a stream is
    /// loaded, and again after any track change so `selected` reflects
    /// reality rather than being assumed client-side.
    private async refreshTracks() {
        try {
            this.tracks = await invoke<PlaybackTrack[]>("get_tracks");
        } catch (e) {
            console.error("Failed to fetch tracks", e);
            this.tracks = [];
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

    /// Set while a duration fetch is in flight, so handlePlayerProgress
    /// doesn't fire a new get_duration call on every single tick while
    /// waiting for the first one to resolve.
    private durationFetchInFlight = false;

    private handlePlayerProgress(currentTime: number) {
        this.currentTime = currentTime;

        // duration is unavailable (mpv error -10, PROPERTY_UNAVAILABLE)
        // until mpv has actually opened and started decoding the file
        if (this.currentDuration <= 0 && !this.durationFetchInFlight) {
            this.durationFetchInFlight = true;
            invoke<number>("get_duration")
                .then((d) => { this.currentDuration = d; })
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
        if (this.hasNext) {
            this.goToEpisode(this.epNumber + 1);
        }
    }

    goToEpisode(episode: number) {
        // TODO: hook up to SvelteKit's router (e.g. `goto` from
        // `$app/navigation`) with this content's route.
    }

    async loadPageData(targetCid: string, targetEp: number) {
        try {
            this.currentLoadedEp = targetEp;

            if (!extensionsStore.initialized) {
                await extensionsStore.load();
            }

            if (targetCid !== this.currentLoadedCid) {
                this.isLoadingMeta = true;
                const contentRes = await contentApi.get_by_cid(targetCid);
                this.animeData = contentRes;
                this.updateEpisodeTitle(targetEp);

                const globalExtensions = extensionsStore.anime;
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
        this.episodeTitle = unit?.title
            ? i18n.t("watch.episode_with_title", { num: ep, title: unit.title })
            : i18n.t("watch.episode_number", { num: ep });
    }

    async selectExtension(ext: string) {
        this.selectedExtension = ext;
        this.servers = [];
        this.supportsDub = false;
        this.selectedServer = null;
        this.isDub = false;

        const isSora = extensionsStore.anime.find(e => e.id === ext)?.source === 'sora';

        if (!isSora) {
            try {
                const s = await extensionsApi.getSettings(ext);
                this.servers = s.episodeServers ?? [];
                this.supportsDub = s.supportsDub ?? false;
                this.selectedServer = this.servers[0] ?? null;
            } catch {}
        }

        await this.loadPlay();
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
        this.currentTime = 0;
        this.isPaused = false;
        this.tracks = [];
        this.lastSyncTime = 0;
        this.hasUpdatedList = false;

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

            const isSora = extensionsStore.anime.find(e => e.id === this.selectedExtension)?.source === 'sora';

            if (isSora) {
                const res = await contentApi.listEpisodeServers(this.selectedExtension, this.cid, this.epNumber);
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

            const res = await contentApi.play(this.cid, this.selectedExtension, this.epNumber, opts);

            if (res.type?.toLowerCase() !== "video") {
                throw { key: "watch.no_stream" } as CoreError;
            }

            const data = res.data as any;
            const rawHeaders = data.headers ?? {};

            this.subtitles = (data.source.subtitles ?? []).map((s: any) => ({
                url: s.url,
                title: s.title,
                lang: s.lang,
            }));
            this.chapters = data.source.chapters ?? [];

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

            await invoke("load_stream", {
                url: data.source.url,
                headers: this.toHeaderList(rawHeaders),
                subtitles: this.subtitles,
                chapters: this.chapters,
                startPosition: initialTime > 0 ? initialTime : undefined,
                nowPlaying,
            });

            this.isStreamLoaded = true;
            await this.refreshTracks();

        } catch (e: any) {
            console.log(e);
            this.error = e.key ? e : { key: "errors.unknown_error" };
        } finally {
            this.isLoadingPlay = false;
        }
    }

    private toHeaderList(headers: Record<string, string>): { key: string; value: string }[] {
        return Object.entries(headers)
            .filter(([, value]) => !!value)
            .map(([key, value]) => ({ key, value }));
    }

    destroy() {
        if (this.destroyed) return;
        this.destroyed = true;

        for (const unlisten of this.unlistenFns) unlisten();
        this.unlistenFns = [];

        invoke("unlock_orientation").catch(() => {});
        invoke("clear_activity").catch(() => {});
        invoke("shutdown_player").catch(() => {});
    }
}