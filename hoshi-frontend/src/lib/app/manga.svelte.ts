import { goto } from "$app/navigation";
import { buildTauriProxyUrl, proxyApi } from "@/api/proxy";
import { isTauri } from "@/api/client";
import { appConfig } from "@/stores/config.svelte.js";
import { progressApi } from "@/api/progress/progress";
import { contentApi } from "@/api/content/content";
import type { MangaConfig } from "@/api/config/types";

import { BaseReaderState } from "./reader.svelte";
import {extensions} from "@/stores/extensions.svelte";
import {extensionsApi} from "@/api/extensions/extensions";

export type ImageEntry = {
    url: string;
    id: string;
    proxyParams?: { url: string; referer?: string; origin?: string; userAgent?: string };
};

export class MangaReaderState extends BaseReaderState {
    images      = $state<ImageEntry[]>([]);
    imageStatus = $state<Record<string, "loading" | "loaded" | "error">>({});

    currentGroupIndex = $state(0);
    animDir           = $state(1);
    skipAnimation     = $state(false);
    showOverlay       = $state(false);

    touchStartX = $state(0);
    touchStartY = $state(0);
    isSwiping   = $state(false);

    private mangaConfig  = $derived(appConfig.data?.manga);
    layout       = $derived(this.mangaConfig?.layout       ?? "scroll");
    pagesPerView = $derived(this.mangaConfig?.pagesPerView ?? 1);
    direction    = $derived(this.mangaConfig?.direction    ?? "ltr");
    fitMode      = $derived(this.mangaConfig?.fitMode      ?? "width");
    gapX         = $derived(this.mangaConfig?.gapX         ?? 0);
    gapY         = $derived(this.mangaConfig?.gapY         ?? 0);

    groupedImages = $derived.by(() => {
        if (this.pagesPerView === 1)
            return this.images.map(img => [img, null] as [ImageEntry, null]);

        const groups: [ImageEntry | null, ImageEntry | null][] = [];
        for (let i = 0; i < this.images.length; i += 2) {
            const img1 = this.images[i];
            const img2 = this.images[i + 1] ?? null;
            groups.push(this.direction === "rtl" ? [img2, img1] : [img1, img2]);
        }
        return groups;
    });

    get isEmpty(): boolean {
        return !this.isLoading && !this.error && this.images.length === 0;
    }

    private currentChapterIndex = $derived(
        this.allChapters.findIndex(c => Number(c.number ?? c.unitNumber) === this.chapterNumber)
    );
    nextChapterNum = $derived(
        this.currentChapterIndex >= 0 && this.currentChapterIndex < this.allChapters.length - 1
            ? (this.allChapters[this.currentChapterIndex + 1].unitNumber ?? this.allChapters[this.currentChapterIndex + 1].number)
            : null
    );
    prevChapterNum = $derived(
        this.currentChapterIndex > 0
            ? (this.allChapters[this.currentChapterIndex - 1].unitNumber ?? this.allChapters[this.currentChapterIndex - 1].number)
            : null
    );

    currentProgress = $derived(
        this.groupedImages.length > 0
            ? `${this.currentGroupIndex + 1} / ${this.groupedImages.length}`
            : null
    );

    private blobCache = new Map<string, string>();

    constructor() {
        super();

        $effect(() => {
            const handler = this.handleKeyDown.bind(this);
            window.addEventListener("keydown", handler);
            return () => window.removeEventListener("keydown", handler);
        });

        // Progress on page turn (paged mode)
        $effect(() => {
            if (this.layout === "paged" && this.groupedImages.length > 0) {
                const ratio = (this.currentGroupIndex + 1) / this.groupedImages.length;
                this.handleProgress(ratio);
            }
        });

        // Preload next images
        $effect(() => {
            this.preloadNextImages(this.currentGroupIndex);
        });

        // Revoke blobs on destroy
        $effect(() => () => this.revokeBlobs());
    }

    private handleKeyDown(e: KeyboardEvent) {
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

        switch (e.key) {
            case "ArrowRight":
                e.preventDefault();
                this.turnPage(this.direction === "rtl" ? "prev" : "next");
                break;
            case "ArrowLeft":
                e.preventDefault();
                this.turnPage(this.direction === "rtl" ? "next" : "prev");
                break;
        }
    }

    protected async loadChapter(currentCid: string, currentExt: string, currentChapterNum: number) {
        this.isLoading = true;
        this.error = null;
        this.skipAnimation = true;
        this.currentGroupIndex = 0;
        this.animDir = 1;
        this.imageStatus = {};
        this.revokeBlobs();
        document.getElementById("reader-main-container")?.scrollTo(0, 0);

        try {
            const [, playRes] = await Promise.all([
                this.fetchShared(currentCid, currentExt, currentChapterNum),
                contentApi.play(currentCid, currentExt, currentChapterNum),
            ]);

            if (playRes.type.toLowerCase() !== "reader") {
                throw { key: "reader.no_data" } as import("@/api/client").CoreError;
            }

            const data: any = playRes.data;
            const rawImages = Array.isArray(data) ? data : (data.pages || data.images || []);
            const globalHeaders = data.headers ?? {};

            let tachiyomiHeaders: Record<string, string> = {};
            if (extensions.isTachiyomi(currentExt) && rawImages.length > 0) {
                const firstUrl = typeof rawImages[0] === "string" ? rawImages[0] : rawImages[0].url;
                try {
                    tachiyomiHeaders = await extensionsApi.getImageRequestHeaders(currentExt, firstUrl, this.allChapters[this.currentChapterIndex].id);
                    console.log(tachiyomiHeaders, this.allChapters[this.currentChapterIndex].id)
                } catch (e) {
                    console.warn("Could not fetch tachiyomi image headers", e);
                }
            }

            this.images = rawImages.map(
                (img: string | { url: string; headers?: Record<string, string> }): ImageEntry => {
                    const rawUrl = typeof img === "string" ? img : img.url;
                    const headers = {
                        ...tachiyomiHeaders,
                        ...globalHeaders,
                        ...(typeof img !== "string" && img.headers ? img.headers : {}),
                    };
                    const proxyParams = { url: rawUrl, ...this.extractHeaders(headers) };
                    return {
                        url: isTauri() ? "" : buildTauriProxyUrl(proxyParams),
                        id: rawUrl,
                        proxyParams,
                    };
                }
            );

            if (this.images.length === 0) {
                throw { key: "reader.no_images" } as import("@/api/client").CoreError;
            }

            progressApi
                .updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));
        } catch (e: any) {
            this.error = e;
        } finally {
            this.isLoading = false;
        }
    }

    async updateMangaConfig(patch: Partial<MangaConfig>) {
        if (!appConfig.data?.manga) return;
        // Optimistically apply the patch locally so the UI reacts immediately
        // without waiting for the API round-trip. The store write will reconcile.
        const merged = { ...appConfig.data.manga, ...patch };
        // Mutate the store's data in place so $derived values re-derive right away.
        appConfig.data = { ...appConfig.data, manga: merged };

        try {
            await appConfig.update({ manga: merged });
        } catch (err) {
            console.error("Error updating config:", err);
            // On failure, reload config to restore truth
            appConfig.load().catch(() => {});
        }
    }

    setImgStatus(id: string, status: "loading" | "loaded" | "error") {
        this.imageStatus[id] = status;
    }

    handleImgMount(node: HTMLImageElement, id: string) {
        if (node.complete) this.setImgStatus(id, "loaded");
        return {};
    }

    async resolveImageUrl(img: ImageEntry): Promise<string> {
        if (!isTauri() || !img.proxyParams) return img.url;
        const key = img.proxyParams.url;
        if (this.blobCache.has(key)) return this.blobCache.get(key)!;
        const blob = await proxyApi.fetch(img.proxyParams);
        const blobUrl = URL.createObjectURL(blob);
        this.blobCache.set(key, blobUrl);
        return blobUrl;
    }

    resolveBlobSrc(node: HTMLImageElement, img: ImageEntry) {
        if (isTauri() && img.proxyParams) {
            this.resolveImageUrl(img).then(url => { node.src = url; });
        }
        return {
            update: (newImg: ImageEntry) => {
                if (isTauri() && newImg.proxyParams) {
                    this.resolveImageUrl(newImg).then(url => { node.src = url; });
                }
            },
        };
    }

    private revokeBlobs() {
        this.blobCache.forEach(url => URL.revokeObjectURL(url));
        this.blobCache.clear();
    }

    private preloadNextImages(currentIndex: number) {
        for (let i = currentIndex + 1; i <= currentIndex + 2; i++) {
            const group = this.groupedImages[i];
            if (!group) continue;
            group.forEach(img => {
                if (!img) return;
                if (isTauri() && img.proxyParams) {
                    this.resolveImageUrl(img).catch(() => {});
                } else {
                    const imgEl = new Image();
                    imgEl.src = img.url;
                }
            });
        }
    }

    handleTouchStart(e: TouchEvent) {
        this.touchStartX = e.changedTouches[0].screenX;
        this.touchStartY = e.changedTouches[0].screenY;
    }

    handleTouchEnd(e: TouchEvent) {
        if (this.layout !== "paged") return;
        const touchEndX = e.changedTouches[0].screenX;
        const touchEndY = e.changedTouches[0].screenY;
        const diffX = this.touchStartX - touchEndX;
        const diffY = this.touchStartY - touchEndY;

        if (Math.abs(diffX) > 50 && Math.abs(diffX) > Math.abs(diffY)) {
            this.isSwiping = true;
            setTimeout(() => (this.isSwiping = false), 300);
            this.turnPage(diffX > 0
                ? (this.direction === "rtl" ? "prev" : "next")
                : (this.direction === "rtl" ? "next" : "prev")
            );
        }
    }

    handleZoneClick(e: MouseEvent) {
        if (this.layout === "scroll" || this.isSwiping) return;
        const readerEl = document.getElementById("reader-main-container");
        if (!readerEl) return;
        const rect = readerEl.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        const margin = rect.width * 0.3;

        if (clickX < margin) {
            this.turnPage(this.direction === "rtl" ? "next" : "prev");
        } else if (clickX > rect.width - margin) {
            this.turnPage(this.direction === "rtl" ? "prev" : "next");
        } else {
            e.preventDefault();
            e.stopPropagation();
            this.showOverlay = !this.showOverlay;
        }
    }

    updatePageWithDir(newIndex: number) {
        if (newIndex === this.currentGroupIndex) return;
        const isForward = newIndex > this.currentGroupIndex;
        const fromRight = this.direction === "rtl" ? !isForward : isForward;
        this.animDir = fromRight ? 1 : -1;
        this.currentGroupIndex = newIndex;
    }

    turnPage(dir: "next" | "prev") {
        if (this.layout === "scroll") return;
        this.skipAnimation = false;

        if (dir === "next") {
            if (this.currentGroupIndex < this.groupedImages.length - 1) {
                this.updatePageWithDir(this.currentGroupIndex + 1);
            } else {
                this.skipAnimation = true;
                if (this.nextChapterNum !== null)
                    goto(`/read/${this.cid}/${this.extension}/${this.nextChapterNum}`);
            }
        } else {
            if (this.currentGroupIndex > 0) {
                this.updatePageWithDir(this.currentGroupIndex - 1);
            } else {
                this.skipAnimation = true;
                if (this.prevChapterNum !== null)
                    goto(`/read/${this.cid}/${this.extension}/${this.prevChapterNum}`);
            }
        }
        document.getElementById("reader-main-container")?.scrollTo(0, 0);
    }

    private extractHeaders(headers: Record<string, string>) {
        return {
            referer:   headers["referer"]    ?? headers["Referer"]    ?? undefined,
            origin:    headers["origin"]     ?? headers["Origin"]     ?? undefined,
            userAgent: headers["user-agent"] ?? headers["User-Agent"] ?? undefined,
        };
    }
}