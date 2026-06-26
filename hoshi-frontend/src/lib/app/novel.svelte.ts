import { goto } from "$app/navigation";
import { appConfig } from "@/stores/config.svelte";
import { progressApi } from "@/api/progress/progress";
import { contentApi } from "@/api/content/content";
import type { NovelConfig } from "@/api/config/types";
import type { CoreError } from "@/api/client";
import { untrack } from "svelte";

import { BaseReaderState } from "./reader.svelte";

export const NOVEL_THEMES = {
    light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
    dark:  { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
    sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
    oled:  { bg: "#000000", text: "#d1d5db", border: "#171717" },
} as const;

export class NovelReaderState extends BaseReaderState {
    contentHtml = $state("");

    private novelConfig = $derived(appConfig.data?.novel);
    theme        = $derived(this.novelConfig?.theme      ?? "dark");
    fontFamily   = $derived(this.novelConfig?.fontFamily ?? "sans");
    textAlign    = $derived(this.novelConfig?.textAlign  ?? "left");

    fontSize         = $state(appConfig.data?.novel?.fontSize         ?? 18);
    lineHeight       = $state(appConfig.data?.novel?.lineHeight       ?? 1.6);
    maxWidth         = $state(appConfig.data?.novel?.maxWidth         ?? 800);
    paragraphSpacing = $state(appConfig.data?.novel?.paragraphSpacing ?? 1.5);

    // Derived from allChapters (inherited $state from BaseReaderState)
    private currentChapterIndex = $derived(
        this.allChapters.findIndex(c => Number(c.number ?? c.unitNumber) === this.chapterNumber)
    );
    private nextChapterNum = $derived(
        this.currentChapterIndex >= 0 && this.currentChapterIndex < this.allChapters.length - 1
            ? (this.allChapters[this.currentChapterIndex + 1].unitNumber ?? this.allChapters[this.currentChapterIndex + 1].number)
            : null
    );
    private prevChapterNum = $derived(
        this.currentChapterIndex > 0
            ? (this.allChapters[this.currentChapterIndex - 1].unitNumber ?? this.allChapters[this.currentChapterIndex - 1].number)
            : null
    );

    get isEmpty(): boolean {
        return !this.isLoading && !this.error && !this.contentHtml;
    }

    private debounceTimer: ReturnType<typeof setTimeout> | undefined;

    scrollProgress = $state(0);

    scrollToPercentage(percent: number) {
        const el = document.getElementById('novel-main-container');
        if (el) {
            const scrollHeight = el.scrollHeight - el.clientHeight;
            el.scrollTop = (percent / 100) * scrollHeight;
        }
    }

    constructor() {
        super();

        $effect(() => {
            const cfg = this.novelConfig;
            if (!cfg) return;
            untrack(() => {
                this.fontSize         = cfg.fontSize         ?? 18;
                this.lineHeight       = cfg.lineHeight       ?? 1.6;
                this.maxWidth         = cfg.maxWidth         ?? 800;
                this.paragraphSpacing = cfg.paragraphSpacing ?? 1.5;
            });
        });

        $effect(() => {
            const size    = this.fontSize;
            const line    = this.lineHeight;
            const width   = this.maxWidth;
            const spacing = this.paragraphSpacing;

            clearTimeout(this.debounceTimer);
            this.debounceTimer = setTimeout(() => {
                untrack(() => this.updateNovelConfig({
                    fontSize:         size,
                    lineHeight:       line,
                    maxWidth:         width,
                    paragraphSpacing: spacing,
                }));
            }, 500);
        });

        $effect(() => {
            const handler = this.handleKeyDown.bind(this);
            window.addEventListener("keydown", handler);
            return () => window.removeEventListener("keydown", handler);
        });
    }

    private handleKeyDown(e: KeyboardEvent) {
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

        const container = document.getElementById("novel-main-container");
        const SCROLL_STEP = 200;

        switch (e.key) {
            case "ArrowDown":
                e.preventDefault();
                container?.scrollBy({ top: SCROLL_STEP, behavior: "smooth" });
                break;
            case "ArrowUp":
                e.preventDefault();
                container?.scrollBy({ top: -SCROLL_STEP, behavior: "smooth" });
                break;
            case " ":
                e.preventDefault();
                container?.scrollBy({
                    top: e.shiftKey ? -window.innerHeight * 0.85 : window.innerHeight * 0.85,
                    behavior: "smooth",
                });
                break;
            case "=":
            case "+":
                e.preventDefault();
                this.fontSize = Math.min(this.fontSize + 1, 36);
                break;
            case "-":
                e.preventDefault();
                this.fontSize = Math.max(this.fontSize - 1, 10);
                break;
        }
    }

    protected async loadChapter(currentCid: string, currentExt: string, currentChapterNum: number) {
        this.isLoading = true;
        this.error = null;

        const container = document.getElementById("novel-main-container");
        if (container) container.scrollTop = 0;

        try {
            const [, playRes] = await Promise.all([
                this.fetchShared(currentCid, currentExt, currentChapterNum),
                contentApi.play(currentCid, currentExt, currentChapterNum),
            ]);
            if (playRes.type.toLowerCase() !== "novel" || !playRes.data) {
                throw { key: "reader.no_data" } as CoreError;
            }

            const data: any = playRes.data;
            this.contentHtml = data.html || data.text || data.content || data;

            if (!this.contentHtml) {
                throw { key: "reader.no_content" } as CoreError;
            }

            progressApi
                .updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));
        } catch (e: any) {
            this.error = e.key ? e : { key: "errors.unknown_error" };
        } finally {
            this.isLoading = false;
        }
    }

    async updateNovelConfig(patch: Partial<NovelConfig>) {
        if (!appConfig.data?.novel) return;
        try {
            await appConfig.update({ novel: { ...appConfig.data.novel, ...patch } });
        } catch (err) {
            console.error("Error updating novel config:", err);
        }
    }

    onScroll(e: Event) {
        const target = e.currentTarget as HTMLElement;
        const scrollHeight = target.scrollHeight - target.clientHeight;
        this.scrollProgress = (target.scrollTop / scrollHeight) * 100;
        if (target.scrollHeight > 0) {
            const ratio = (target.scrollTop + target.clientHeight) / target.scrollHeight;
            this.handleProgress(ratio);
        }
    }

    get themeColors() {
        return NOVEL_THEMES[this.theme as keyof typeof NOVEL_THEMES] ?? NOVEL_THEMES.dark;
    }
}