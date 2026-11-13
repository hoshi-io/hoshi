import { page } from "$app/state";
import { untrack } from "svelte";

import { contentApi } from "@/api/content/content";
import { primaryMetadata } from "@/api/content/types";
import { appConfig } from "@/stores/config.svelte.js";
import { progressApi } from "@/api/progress/progress";
import type { CoreError } from "@/api/client";
import {listStore} from "@/app/list.svelte";
import {invoke} from "@tauri-apps/api/core";
import { chapterHref, chapterNum, collapseByNumber, type ChapterLike } from "@/app/chapterVariants";

export type ChapterItem = ChapterLike;

export abstract class BaseReaderState {

    cid           = $derived((page.params as Record<string, string>).cid           ?? "");
    extension     = $derived((page.params as Record<string, string>).extension     ?? "");
    chapterNumber = $derived(Number((page.params as Record<string, string>).number ?? 0));
    /** Pinned version (language/scanlator) of this chapter number; empty = let the source pick. */
    variantId     = $derived(page.url.searchParams.get("v") ?? "");

    isLoading    = $state(true);
    error        = $state<CoreError | null>(null);
    title        = $state("");
    chapterTitle = $state("");
    coverImage   = $state<string | null>(null);
    isNsfw       = $state(false);
    /** Every version the extension returned, sorted by number. */
    rawChapters  = $state<ChapterItem[]>([]);

    /** Same fallback as the backend: the pinned version, else the first one with this number. */
    currentChapter = $derived(
        this.rawChapters.find(c => chapterNum(c) === this.chapterNumber && c.id === this.variantId)
        ?? this.rawChapters.find(c => chapterNum(c) === this.chapterNumber)
    );
    /** One chapter per number, preferring the current chapter's language/scanlator. */
    allChapters = $derived(collapseByNumber(this.rawChapters, this.currentChapter));

    private currentChapterIndex = $derived(
        this.allChapters.findIndex(c => chapterNum(c) === this.chapterNumber)
    );
    nextChapter = $derived(
        this.currentChapterIndex >= 0 ? this.allChapters[this.currentChapterIndex + 1] ?? null : null
    );
    prevChapter = $derived(
        this.currentChapterIndex > 0 ? this.allChapters[this.currentChapterIndex - 1] : null
    );

    protected abstract readonly baseRoute: string;

    chapterHref(ch: ChapterItem) {
        return chapterHref(this.baseRoute, this.cid, this.extension, ch, this.rawChapters);
    }

    private hasUpdatedList     = $state(false);
    private hasMarkedCompleted = $state(false);

    showSettings = $state(false);
    abstract get isEmpty(): boolean;

    constructor() {
        $effect(() => {
            invoke('enter_fullscreen').catch((e) => console.error("Failed to enter fullscreen", e));

            return () => {
                invoke('exit_fullscreen').catch(() => {});
            };
        });

        $effect(() => {
            this.chapterNumber;
            untrack(() => {
                this.hasUpdatedList = false;
                this.hasMarkedCompleted = false;
            });
        });

        // Trigger load when route params change
        $effect(() => {
            const currentId = `${this.cid}-${this.extension}-${this.chapterNumber}-${this.variantId}`;
            if (this.cid && this.extension && !isNaN(this.chapterNumber)) {
                untrack(() => this.load(currentId));
            }
        });
    }

    // Tracks last-loaded ID to avoid duplicate fetches
    private loadedId = $state("");

    private async load(currentId: string) {
        if (this.loadedId === currentId) return;
        this.loadedId = currentId;
        await this.loadChapter(this.cid, this.extension, this.chapterNumber);
    }

    /** `stale` lets the caller's newer load win: nothing is applied once it returns true. */
    protected async fetchShared(
        currentCid: string, currentExt: string, currentChapterNum: number,
        stale: () => boolean = () => false,
    ) {
        const [contentRes, itemsRes] = await Promise.all([
            contentApi.get_by_cid(currentCid),
            contentApi.getItems(currentCid, currentExt),
        ]);
        if (stale()) return contentRes;

        this.isNsfw = contentRes.content.nsfw;

        const meta = primaryMetadata(contentRes);
        this.title      = meta?.title ?? "";
        this.coverImage = meta?.coverImage ?? null;

        const rawItems: any[] = Array.isArray(itemsRes) ? itemsRes : (itemsRes as any)?.data ?? [];
        this.rawChapters = rawItems.sort((a, b) => chapterNum(a) - chapterNum(b));
        this.chapterTitle = this.currentChapter?.title || "";

        return contentRes;
    }
    protected abstract loadChapter(cid: string, ext: string, chapterNum: number): Promise<void>;

    handleProgress(ratio: number) {
        if (!this.hasMarkedCompleted && ratio >= 0.9) {
            this.hasMarkedCompleted = true;
            progressApi
                .updateChapterProgress({ cid: this.cid, chapter: this.chapterNumber, completed: true })
                .catch(e => console.error("History completion sync failed", e));
        }

        if (!this.hasUpdatedList && ratio >= 0.8 && appConfig.data?.content.autoUpdateProgress) {
            this.hasUpdatedList = true;
            // Trackers count whole chapters: reading 10.5 means 10 are done.
            // (The chapter history above keeps the exact 10.5.)
            listStore
                .autoUpdateProgress(this.cid, Math.floor(this.chapterNumber))
                .catch(e => console.error("List sync failed", e));
        }
    }

    retry() {
        this.loadedId = "";
        this.loadChapter(this.cid, this.extension, this.chapterNumber);
    }
}