import { page } from "$app/state";
import { goto } from "$app/navigation";
import { untrack } from "svelte";

import { contentApi } from "@/api/content/content";
import { extensionsApi } from "@/api/extensions/extensions";
import { extensions as extensionsStore } from "@/stores/extensions.svelte.js";
import { buildTauriProxyUrl, proxyApi } from "@/api/proxy";
import { type CoreError } from "@/api/client";
import { progressApi } from "@/api/progress/progress";
import { listApi } from "@/api/list/list";
import { appConfig } from "@/stores/config.svelte.js";
import { i18n } from "@/stores/i18n.svelte.js";
import { discordApi } from "@/api/discord/discord";
import { primaryMetadata } from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { invoke } from "@tauri-apps/api/core";

import type { Subtitle, Chapter } from "@/components/player/types";
import {listStore} from "@/app/list.svelte";
import {layoutState} from "@/stores/layout.svelte";
import type {Extension} from "@/api/extensions/types";

export class PlayerState {
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
    selectedExtension   = $state<string | null>(null);
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
    m3u8Url         = $state<string | null>(null);
    subtitles       = $state<Subtitle[]>([]);
    chapters        = $state<Chapter[]>([]);
    initialTime     = $state(0);

    isMappingError = $derived(!!this.error?.key?.includes("match"));

    isPaused            = $state(true);
    currentDuration     = $state(0);
    lastCurrentTime     = $state(0);
    lastSyncTime        = $state(0);
    hasUpdatedList      = $state(false);
    discordStatusUpdated = $state(false);

    private currentLoadedCid    = $state<string | null>(null);
    private currentLoadedEp     = $state<number | null>(null);
    private subtitleBlobUrls: string[] = [];
    private destroyed = false;

    constructor() {
        invoke("lock_orientation", { orientation: "landscape" }).catch(() => {});

        $effect(() => {
            const { cid, epNumber } = this;
            if (cid && epNumber && (cid !== this.currentLoadedCid || epNumber !== this.currentLoadedEp)) {
                untrack(() => this.loadPageData(cid, epNumber));
            }
        });

        $effect(() => () => this.revokeSubtitleBlobs());

        $effect(() => {
            if (this.m3u8Url && "mediaSession" in navigator && this.animeData) {
                const meta = primaryMetadata(this.animeData, appConfig.data?.content?.preferredMetadataProvider);
                const coverImage = meta?.coverImage || meta?.bannerImage || "";

                const setMediaSession = (artworkArray: MediaImage[]) => {
                    navigator.mediaSession.metadata = new MediaMetadata({
                        title: this.episodeTitle || i18n.t("watch.episode"),
                        artist: this.animeTitle,
                        album: "Hoshi",
                        artwork: artworkArray,
                    });
                };

                if (coverImage) {
                    fetch(coverImage)
                        .then(res => res.blob())
                        .then(blob => {
                            const reader = new FileReader();
                            reader.onloadend = () => {
                                setMediaSession([{ src: reader.result as string }]);
                            };
                            reader.readAsDataURL(blob);
                        })
                        .catch(() => setMediaSession([{ src: coverImage }]));
                } else {
                    setMediaSession([]);
                }
            }

            return () => {
                if ("mediaSession" in navigator) navigator.mediaSession.metadata = null;
            };
        });
    }

    async loadPageData(targetCid: string, targetEp: number) {
        try {
            this.discordStatusUpdated = false;
            this.currentLoadedEp = targetEp;

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

    async loadPlay() {
        if (!this.selectedExtension) return;

        this.isLoadingPlay = true;
        this.m3u8Url = null;
        this.error = null;
        this.revokeSubtitleBlobs();
        this.lastSyncTime = 0;
        this.hasUpdatedList = false;

        try {
            if (appConfig.data?.player.resumeFromLastPos) {
                try {
                    const res = await progressApi.getContentProgress(this.cid);
                    const prog = res.animeProgress.find((p: any) => p.episode === this.epNumber);
                    this.initialTime = prog?.timestampSeconds ?? 0;
                } catch {
                    this.initialTime = 0;
                }
            }

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
            const headers = data.headers ?? {};

            this.m3u8Url = buildTauriProxyUrl({ url: data.source.url, ...this.extractHeaders(headers) });
            this.chapters = data.source.chapters ?? [];
            this.subtitles = [];

            this.fetchSubtitles(data.source.subtitles ?? [], headers);

        } catch (e: any) {
            console.log(e);
            this.error = e.key ? e : { key: "errors.unknown_error" };
        } finally {
            this.isLoadingPlay = false;
        }
    }

    private async fetchSubtitles(rawSubs: any[], headers: Record<string, string>) {
        const results = await Promise.all(
            rawSubs.map(async (s: any) => {
                const proxyParams = { url: s.url, ...this.extractHeaders(headers) };
                try {
                    const blob = await proxyApi.fetch(proxyParams);
                    const isAss = s.url.toLowerCase().endsWith(".ass") || s.url.toLowerCase().endsWith(".ssa");
                    let finalBlob = blob;
                    if (isAss) {
                        const textData = await blob.text();
                        finalBlob = new Blob([this.convertAssToVtt(textData)], { type: "text/vtt" });
                    }
                    const blobUrl = URL.createObjectURL(finalBlob);
                    this.subtitleBlobUrls.push(blobUrl);
                    return { ...s, url: blobUrl, type: "vtt" };
                } catch {
                    return null;
                }
            })
        );
        this.subtitles = results.filter((s): s is Subtitle => s !== null);
    }

    onTimeUpdate(data: { currentTime: number; duration: number; paused: boolean }) {
        this.lastCurrentTime = data.currentTime;
        this.currentDuration = data.duration;
        this.isPaused = data.paused;
        this.handlePlayerProgress(data);
    }

    onPlay() {
        this.syncDiscord(false);
    }

    onPause() {
        this.syncDiscord(true);
    }

    onSeek(time: number) {
        this.lastCurrentTime = time;
        this.syncDiscord(this.isPaused);
    }

    onEnded() {
        discordApi.clearActivity();
        if (this.hasNext) goto(`/watch/${this.cid}/${this.epNumber + 1}`);
    }

    private handlePlayerProgress({ currentTime, duration }: { currentTime: number; duration: number }) {
        if (!appConfig.data) return;

        if (!this.discordStatusUpdated && duration > 0) {
            const meta = primaryMetadata(this.animeData, appConfig.data?.content?.preferredMetadataProvider);
            const coverImage = meta?.coverImage || "";
            const now = Math.floor(Date.now() / 1000);
            const start = now - Math.floor(currentTime);
            const end = start + Math.floor(duration);

            discordApi.setActivity({
                title: this.animeTitle,
                details: this.episodeTitle,
                imageUrl: coverImage,
                startTime: start,
                endTime: end,
                isVideo: true,
                isNsfw: this.animeData?.content?.nsfw ?? false,
            }).catch(() => {});

            this.discordStatusUpdated = true;
        }

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

    private async syncDiscord(paused: boolean) {
        if (!this.animeData) return;
        this.isPaused = paused;

        const meta = primaryMetadata(this.animeData, appConfig.data?.content?.preferredMetadataProvider);
        const nowInSeconds = Math.floor(Date.now() / 1000);
        const startTime = !paused ? nowInSeconds - Math.floor(this.lastCurrentTime) : null;
        const endTime =
            !paused && this.currentDuration > 0
                ? startTime! + Math.floor(this.currentDuration)
                : null;

        await discordApi.setActivity({
            title: this.animeTitle,
            details: this.episodeTitle,
            imageUrl: meta?.coverImage || null,
            startTime,
            endTime,
            isVideo: true,
            isNsfw: this.animeData.content.nsfw,
        }).catch(() => {});
    }

    private revokeSubtitleBlobs() {
        this.subtitleBlobUrls.forEach(u => URL.revokeObjectURL(u));
        this.subtitleBlobUrls = [];
    }

    private convertAssToVtt(assData: string): string {
        const lines = assData.split(/\r?\n/);
        let vtt = "WEBVTT\n\n";
        let isEvents = false;
        let format: string[] = [];

        for (let line of lines) {
            line = line.trim();
            if (line === "[Events]") { isEvents = true; continue; }
            if (!isEvents) continue;
            if (line.startsWith("Format:")) {
                format = line.substring(7).split(",").map(s => s.trim());
                continue;
            }
            if (line.startsWith("Dialogue:")) {
                const parts = line.substring(9).split(",");
                const startIdx = format.indexOf("Start");
                const endIdx = format.indexOf("End");
                const textIdx = format.indexOf("Text");
                if (startIdx === -1 || endIdx === -1 || textIdx === -1) continue;

                const start = this.formatAssTime(parts[startIdx]);
                const end = this.formatAssTime(parts[endIdx]);
                let text = parts.slice(textIdx).join(",");
                text = text.replace(/\{[^}]+\}/g, "").replace(/\\N/gi, "\n");
                vtt += `${start} --> ${end}\n${text}\n\n`;
            }
        }
        return vtt;
    }

    private formatAssTime(assTime: string): string {
        const [hms, msPart = "00"] = assTime.trim().split(".");
        const [h, m, s] = hms.split(":");
        return `${h.padStart(2, "0")}:${m.padStart(2, "0")}:${s.padStart(2, "0")}.${msPart.padEnd(3, "0").substring(0, 3)}`;
    }

    private extractHeaders(headers: Record<string, string>) {
        return {
            referer: headers["Referer"],
            origin: headers["Origin"],
            userAgent: headers["User-Agent"],
        };
    }

    destroy() {
        if (this.destroyed) return;
        this.destroyed = true;
        discordApi.clearActivity().catch(() => {});
        invoke("unlock_orientation").catch(() => {});
    }
}