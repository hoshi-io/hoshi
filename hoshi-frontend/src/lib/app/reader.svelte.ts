import { page } from "$app/state";
import { untrack } from "svelte";

import { contentApi } from "@/api/content/content";
import { primaryMetadata } from "@/api/content/types";
import { appConfig } from "@/stores/config.svelte.js";
import { progressApi } from "@/api/progress/progress";
import { listApi } from "@/api/list/list";
import type { CoreError } from "@/api/client";
import {listStore} from "@/app/list.svelte";
import {invoke} from "@tauri-apps/api/core";

export type ChapterItem = {
    number?: string | number;
    unitNumber?: string | number;
    title?: string;
    id?: string;
    [key: string]: unknown;
};

export abstract class BaseReaderState {

    cid           = $derived((page.params as Record<string, string>).cid           ?? "");
    extension     = $derived((page.params as Record<string, string>).extension     ?? "");
    chapterNumber = $derived(Number((page.params as Record<string, string>).number ?? 0));

    isLoading    = $state(true);
    error        = $state<CoreError | null>(null);
    title        = $state("");
    chapterTitle = $state("");
    coverImage   = $state<string | null>(null);
    isNsfw       = $state(false);
    allChapters  = $state<ChapterItem[]>([]);

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
            const currentId = `${this.cid}-${this.extension}-${this.chapterNumber}`;
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

    protected async fetchShared(currentCid: string, currentExt: string, currentChapterNum: number) {
        const [contentRes, itemsRes] = await Promise.all([
            contentApi.get_by_cid(currentCid),
            contentApi.getItems(currentCid, currentExt),
        ]);

        this.isNsfw = contentRes.content.nsfw;

        const meta = primaryMetadata(contentRes);
        this.title      = meta?.title ?? "";
        this.coverImage = meta?.coverImage ?? null;

        const rawItems: any[] = Array.isArray(itemsRes) ? itemsRes : (itemsRes as any)?.data ?? [];
        this.allChapters = rawItems.sort(
            (a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber)
        );

        const currentUnit = this.allChapters.find(
            u => Number(u.number ?? u.unitNumber) === currentChapterNum
        );
        this.chapterTitle = currentUnit?.title || "";

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
            listApi
                .upsert({ cid: this.cid, status: "CURRENT", progress: this.chapterNumber })
                .catch(e => console.error("List sync failed", e));

            listStore.updateEntryProgressLocal(this.cid, this.chapterNumber, "CURRENT");
        }
    }

    retry() {
        this.loadedId = "";
        this.loadChapter(this.cid, this.extension, this.chapterNumber);
    }
}