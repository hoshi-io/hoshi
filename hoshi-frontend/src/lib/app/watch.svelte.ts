import { page } from "$app/state";
import { untrack } from "svelte";

import { contentApi } from "@/api/content/content";
import { extensionsApi } from "@/api/extensions/extensions";
import { extensions as extensionsStore } from "@/stores/extensions.svelte.js";
import { type CoreError } from "@/api/client";
import { progressApi } from "@/api/progress/progress";
import { appConfig } from "@/stores/config.svelte.js";
import { i18n } from "@/stores/i18n.svelte.js";
import { primaryMetadata } from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { invoke } from "@tauri-apps/api/core";

import type { Extension } from "@/api/extensions/types";

export interface SubtitleSource {
    url: string;
    title?: string;
    lang?: string;
}

export interface ChapterMark {
    title?: string;
    time: number;
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
    chapters        = $state<ChapterMark[]>([]);
    initialTime     = $state(0);

    isMappingError = $derived(!!this.error?.key?.includes("match"));

    private currentLoadedCid    = $state<string | null>(null);
    private currentLoadedEp     = $state<number | null>(null);
    private destroyed = false;

    constructor() {
        invoke("lock_orientation", { orientation: "landscape" }).catch(() => {});

        // Best-effort guess pending the actual app architecture: mpv's
        // `initialize()` is idempotent on the backend, so calling it once
        // here (rather than per-episode in loadPlay) is safe either way.
        // What's NOT decided yet is whether mpv should live only for the
        // duration of this page (shutdown_player in destroy(), below) or
        // persist across navigation as an app-shell singleton. Revisit once
        // that's settled — if it's app-shell-owned, both this call and the
        // shutdown_player call in destroy() should move out of here.
        invoke("initialize_player").catch(() => {});

        $effect(() => {
            const { cid, epNumber } = this;
            if (cid && epNumber && (cid !== this.currentLoadedCid || epNumber !== this.currentLoadedEp)) {
                untrack(() => this.loadPageData(cid, epNumber));
            }
        });

        // TODO(media session): navigator.mediaSession used to be driven off
        // the <video> element's own state. There's no DOM media element
        // anymore (mpv renders behind the webview via GtkGLArea on Linux),
        // so this needs to be rethought — at minimum re-populating
        // metadata (title/artist/artwork) from `this.animeData` still seems
        // possible, but action handlers (play/pause/seek) would need to
        // invoke the corresponding Tauri commands instead of manipulating a
        // media element. Left out entirely for now rather than half-done.
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

            // No more proxy/blob-url dance: mpv fetches the stream and its
            // subtitles itself, headers and all, so we just hand it the raw
            // url + headers directly. See playback.rs for why this replaces
            // buildTauriProxyUrl rather than sitting alongside it.
            await invoke("load_stream", {
                url: data.source.url,
                headers: this.toHeaderList(rawHeaders),
                subtitles: this.subtitles,
                chapters: this.chapters,
                startPosition: initialTime > 0 ? initialTime : undefined,
            });

            this.isStreamLoaded = true;

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

    // TODO(backend): progress persistence, Discord RPC, and the
    // auto-"mark as watching/completed" list update all used to live here,
    // driven by the old player component's onTimeUpdate/onPlay/onPause/
    // onSeek/onEnded callbacks (removed along with hls.js/<video>). There's
    // no equivalent signal yet now that mpv owns playback natively — no
    // Tauri event for position/pause/eof, and nothing here polling
    // get_position/get_duration. Once that exists, this needs to come back,
    // but per your note, ideally implemented on the Rust side: it already
    // has direct access to mpv's property-change/eof-reached events and
    // doesn't need a webview round trip just to know the playhead moved.
    // Auto-advance to the next episode (old onEnded -> goto(...)) falls in
    // the same bucket: it's frontend-appropriate (navigation isn't mpv's
    // job) but still needs an "eof-reached" signal from somewhere.

    destroy() {
        if (this.destroyed) return;
        this.destroyed = true;
        invoke("unlock_orientation").catch(() => {});
        // See constructor note: shutdown_player pairing with the
        // initialize_player call there is a guess, not a confirmed
        // lifecycle decision.
        invoke("shutdown_player").catch(() => {});
    }
}