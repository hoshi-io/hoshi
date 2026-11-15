import { appConfig } from "@/stores/config.svelte";
import { progressApi } from "@/api/progress/progress";
import { contentApi } from "@/api/content/content";
import type { NovelConfig } from "@/api/config/types";
import type { CoreError } from "@/api/client";
import { untrack } from "svelte";
import { goto } from "$app/navigation";

import { BaseReaderState } from "./reader.svelte";
import { sanitizeHtml } from "@/utils/sanitizeHtml";

export const NOVEL_THEMES = {
    light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
    dark:  { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
    sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
    oled:  { bg: "#000000", text: "#d1d5db", border: "#171717" },
} as const;

export function novelThemeColors(cfg: Partial<NovelConfig> | undefined) {
    if (cfg?.theme === "custom") {
        const bg = cfg.customBg || "#1e1e24";
        const text = cfg.customText || "#e4e4e7";
        return { bg, text, border: `color-mix(in srgb, ${text} 15%, ${bg})` };
    }
    return NOVEL_THEMES[cfg?.theme as keyof typeof NOVEL_THEMES] ?? NOVEL_THEMES.dark;
}

export class NovelReaderState extends BaseReaderState {
    contentHtml = $state("");

    private novelConfig = $derived(appConfig.data?.novel);
    theme        = $derived(this.novelConfig?.theme        ?? "dark");
    fontFamily   = $derived(this.novelConfig?.fontFamily   ?? "sans");
    textAlign    = $derived(this.novelConfig?.textAlign    ?? "left");
    layout       = $derived(this.novelConfig?.layout       ?? "scroll");
    pagesPerView = $derived(this.novelConfig?.pagesPerView ?? 2);
    paged        = $derived(this.layout === "paged");

    // Slider-driven values: local first, saved debounced.
    fontSize         = $state(appConfig.data?.novel?.fontSize         ?? 18);
    lineHeight       = $state(appConfig.data?.novel?.lineHeight       ?? 1.6);
    maxWidth         = $state(appConfig.data?.novel?.maxWidth         ?? 800);
    paragraphSpacing = $state(appConfig.data?.novel?.paragraphSpacing ?? 1.5);
    margin           = $state(appConfig.data?.novel?.margin           ?? 24);
    letterSpacing    = $state(appConfig.data?.novel?.letterSpacing    ?? 0);
    wordSpacing      = $state(appConfig.data?.novel?.wordSpacing      ?? 0);

    protected readonly baseRoute = "/read-novel";

    get isEmpty(): boolean {
        return !this.isLoading && !this.error && !this.contentHtml;
    }

    private debounceTimer: ReturnType<typeof setTimeout> | undefined;
    /** Bumped per chapter load so a slower, older load can't overwrite a newer one. */
    private loadToken = 0;

    scrollProgress = $state(0);

    // ---- paged mode ----
    page      = $state(0);
    pageCount = $state(1);
    /** Width of one spread plus the gutter: how far one page turn moves. */
    private pageStep = 0;

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
                this.margin           = cfg.margin           ?? 24;
                this.letterSpacing    = cfg.letterSpacing    ?? 0;
                this.wordSpacing      = cfg.wordSpacing      ?? 0;
            });
        });

        $effect(() => {
            const patch = {
                fontSize:         this.fontSize,
                lineHeight:       this.lineHeight,
                maxWidth:         this.maxWidth,
                paragraphSpacing: this.paragraphSpacing,
                margin:           this.margin,
                letterSpacing:    this.letterSpacing,
                wordSpacing:      this.wordSpacing,
            };

            clearTimeout(this.debounceTimer);
            this.debounceTimer = setTimeout(() => {
                untrack(() => this.updateNovelConfig(patch));
            }, 500);
        });

        $effect(() => {
            const handler = this.handleKeyDown.bind(this);
            window.addEventListener("keydown", handler);
            return () => window.removeEventListener("keydown", handler);
        });
    }

    get themeColors() {
        return novelThemeColors({
            theme: this.theme,
            customBg: this.novelConfig?.customBg,
            customText: this.novelConfig?.customText,
        });
    }

    private get container() {
        return document.getElementById("novel-main-container");
    }

    private get pager() {
        return document.getElementById("novel-pager");
    }

    scrollToPercentage(percent: number) {
        if (this.paged) {
            this.goToPage(Math.round((percent / 100) * (this.pageCount - 1)));
            return;
        }
        const el = this.container;
        if (el) el.scrollTop = (percent / 100) * (el.scrollHeight - el.clientHeight);
    }

    /** Re-measures pages after anything that reflows the text, keeping the reading position. */
    relayout(spreadWidth: number, gap: number, columns: number) {
        const pager = this.pager;
        if (!pager || spreadWidth <= 0) return;

        const fraction = this.pageCount > 1 ? this.page / (this.pageCount - 1) : 0;
        const columnWidth = (spreadWidth - (columns - 1) * gap) / columns;
        const totalColumns = Math.max(1, Math.round((pager.scrollWidth + gap) / (columnWidth + gap)));

        this.pageStep = spreadWidth + gap;
        this.pageCount = Math.max(1, Math.ceil(totalColumns / columns));
        this.goToPage(Math.round(fraction * (this.pageCount - 1)), false);
    }

    goToPage(target: number, smooth = true) {
        const page = Math.min(Math.max(0, target), this.pageCount - 1);
        this.page = page;
        const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
        this.pager?.scrollTo({ left: page * this.pageStep, behavior: smooth && !reduce ? "smooth" : "auto" });

        // An empty or still-loading chapter measures as one page: that isn't progress.
        if (this.isLoading || !this.contentHtml) return;
        this.scrollProgress = this.pageCount > 1 ? (page / (this.pageCount - 1)) * 100 : 100;
        this.handleProgress((page + 1) / this.pageCount);
    }

    nextPage() {
        if (this.page < this.pageCount - 1) this.goToPage(this.page + 1);
        else if (this.nextChapter) goto(this.chapterHref(this.nextChapter));
    }

    prevPage() {
        if (this.page > 0) this.goToPage(this.page - 1);
        else if (this.prevChapter) goto(this.chapterHref(this.prevChapter));
    }

    private handleKeyDown(e: KeyboardEvent) {
        if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
        if (e.ctrlKey || e.metaKey || e.altKey) return;

        if (e.key === "=" || e.key === "+") {
            e.preventDefault();
            this.fontSize = Math.min(this.fontSize + 1, 36);
            return;
        }
        if (e.key === "-") {
            e.preventDefault();
            this.fontSize = Math.max(this.fontSize - 1, 10);
            return;
        }

        if (this.paged) {
            const forward = ["ArrowRight", "ArrowDown", "PageDown"].includes(e.key) || (e.key === " " && !e.shiftKey);
            const back = ["ArrowLeft", "ArrowUp", "PageUp"].includes(e.key) || (e.key === " " && e.shiftKey);
            if (forward || back) {
                e.preventDefault();
                forward ? this.nextPage() : this.prevPage();
            }
            return;
        }

        const container = this.container;
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
        }
    }

    protected async loadChapter(currentCid: string, currentExt: string, currentChapterNum: number) {
        const token = ++this.loadToken;
        const stale = () => token !== this.loadToken;

        this.isLoading = true;
        this.error = null;
        this.contentHtml = "";
        this.page = 0;
        this.pageCount = 1;
        this.scrollProgress = 0;

        const container = this.container;
        if (container) container.scrollTop = 0;
        this.pager?.scrollTo({ left: 0 });

        try {
            const [, playRes] = await Promise.all([
                this.fetchShared(currentCid, currentExt, currentChapterNum, stale),
                contentApi.play(currentCid, currentExt, currentChapterNum, { itemId: this.variantId || undefined }),
            ]);
            if (stale()) return;
            if (playRes.type.toLowerCase() !== "novel" || !playRes.data) {
                throw { key: "reader.no_data" } as CoreError;
            }

            const data: any = playRes.data;
            const raw = typeof data === "string"
                ? data
                : [data.html, data.text, data.content].find(v => typeof v === "string") ?? "";
            // Rendered with {@html}: must be sanitized (see sanitizeHtml.ts).
            this.contentHtml = sanitizeHtml(raw);

            if (!this.contentHtml) {
                throw { key: "reader.no_content" } as CoreError;
            }

            progressApi
                .updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));
        } catch (e: any) {
            if (!stale()) this.error = e?.key ? e : { key: "error.system.unknown" };
        } finally {
            if (!stale()) this.isLoading = false;
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
        if (this.paged) return;
        const target = e.currentTarget as HTMLElement;
        const scrollHeight = target.scrollHeight - target.clientHeight;
        this.scrollProgress = scrollHeight > 0 ? (target.scrollTop / scrollHeight) * 100 : 100;
        if (target.scrollHeight > 0) {
            const ratio = (target.scrollTop + target.clientHeight) / target.scrollHeight;
            this.handleProgress(ratio);
        }
    }
}
